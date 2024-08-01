import argparse
import platform
import re
import subprocess
from collections import Counter
from importlib.metadata import distribution
from pathlib import Path
from typing import Any

import tomlkit


class Package:
    """Used to store a package's information including name, license, and packages it requires."""

    def __init__(self, name: str, license: str) -> None:
        self.name = name  # name of the package
        self.license = license  # license of the package

    def __str__(self) -> str:
        return f"{self.name} ({self.license})"

    @property
    def requirements(self) -> list[str]:
        """Returns the list of packages the package requires."""
        return self._requirements

    @requirements.setter
    def requirements(self, reqs: list[str]) -> None:
        self._requirements = reqs


class ProjectLicenses:
    """Used to store project's dependencies and licenses of said dependencies."""

    def __init__(
        self,
        recursive: bool,
        by_package: bool,
        print_fails: bool,
        to_avoid: list[str] | None,
    ) -> None:
        self._recursive: bool = (
            recursive  # cli argument for recursive dependencies fetchign
        )
        self.by_package = by_package  # print by package
        self.print_fails = (
            print_fails  # print only packages whose licenses want to be avoided
        )
        self.to_avoid = to_avoid if to_avoid else ["MIT"]  # list of licenses to avoid

        self._python_version: tuple[str, str, str] = platform.python_version_tuple()
        self._packages: dict[str, Package] = {}  # map package name to object
        self._project_dependencies: list[str] = []
        # direct dependencies names of project

    def find_project_dependencies(self) -> list[str]:
        """Get all the direct dependencies of the project.

        Returns a list of the names of the packages the project depends on.
        """

        dependencies = subprocess.check_output(
            ["python", "-m" "pip", "freeze"], text=True
        )
        dependencies = [dep.split("==")[0] for dep in dependencies.split("\n") if dep]
        return dependencies

    def _matches_python_version(self, req_info: str) -> bool:
        """Check if the python version for which requirement of a package is needed matches the
        projects python version. `distribution` does not have metadata for packages that are not
        install and package requirements with specified python version for the requirement are not
        installed if the version is not matched.

        Args:
            req_info: The package requirement info.
                Formatted '<package_name> <expression> <version> ; python_version <expression> <python_version>

        Returns whether the projects version matches the packages specified version.
        """

        expression = req_info.split(";")[1]

        for char in ["'", '"', " "]:
            expression = expression.replace(char, "")

        version = re.split(r"==|<=|>=|!=|<|>", expression)[1].strip().split(".")
        if len(version) == 2:
            version.append(self._python_version[2])

        diff = [
            int(ver) - int(p_ver) for ver, p_ver in zip(version, self._python_version)
        ]

        # print(req_info, expression, version, self._python_version, diff, end=" -> ")
        if "<=" in expression:
            return diff[0] > 0 or (diff[0] == 0 and diff[1] >= 0)
        elif "<" in expression:
            return diff[0] > 0 or (diff[0] == 0 and diff[1] > 0)
        elif ">=" in expression:
            return diff[0] < 0 or (diff[0] == 0 and diff[1] <= 0)
        elif ">" in expression:
            return diff[0] < 0 or (diff[0] == 0 and diff[1] > 0)
        elif "==" in expression:
            return diff[0] == 0 and diff[1] == 0 and diff[2] == 0
        elif "!=" in expression:
            return diff[0] != 0 or diff[1] != 0 or diff[2] != 0
        return True

    def get_package_requirements(self, package_name: str) -> list[str]:
        """Get the packages that a package requires.

        Args:
            package_name: The package whose requirements are being checked.

        Returns a list of the packages requirements.
        """

        package_requirements = []
        if req_info := distribution(package_name).metadata.get_all("Requires-Dist"):
            for req in req_info:
                if (
                    ";" not in req
                    or ("; python_version" in req)
                    and self._matches_python_version(req)
                ):
                    package_req = re.split(r"[<>=~\(;!]", req)[0].strip()
                    package_requirements.append(package_req)

        return package_requirements

    def get_license(self, package_name: str) -> str:
        """Get the license of the package from the cache, License metadata, or Classifier metadata.

        Args:
            package_name: The package's name.

        Returns the packages license.
        """

        if package_name in self._packages:
            return self._packages[package_name].license

        if (
            not (license := distribution(package_name).metadata["License"])
            or len(license) > 10
        ):
            # really long license strings are likely to be the entire licensing doc

            classifier = distribution(package_name).metadata.get_all("Classifier")
            if not classifier:
                # edge case when package_name does not have classifier information
                return "?"

            for value in classifier:
                if "License" in value:
                    license = value.split(" :: ")[-1]
                    return license.replace("License", "").strip()

        return license.replace("License", "").strip()

    def get_project_dependencies_and_licenses(self) -> None:
        """Get the direct dependencies of the project and their licenses."""

        dependencies = self.find_project_dependencies()
        for package in dependencies:
            cur_package = Package(package, self.get_license(package))
            self._packages[package] = cur_package
            cur_package.requirements = self.get_package_requirements(package)
            self._project_dependencies.append(package)

    def fetch_recursive_dependencies(self):
        """Recursively find all the packages each of the direct dependencies of the project require."""

        queue = self._project_dependencies[:]
        while queue:
            package = queue.pop()

            if package not in self._packages:
                cur_package = Package(package, self.get_license(package))
                cur_package.requirements = self.get_package_requirements(package)
                self._packages[package] = cur_package

            for req in self._packages[package].requirements:
                if req not in self._packages:
                    queue.append(req)

    def pretty_print(self):
        """Pretty print the licenses of all the dependencies of the project."""

        packages = (
            sorted(list(self._packages.values()), key=lambda x: x.license)
            if not self.by_package
            else sorted(self._packages.values(), key=lambda x: x.name.lower())
        )
        license_count = Counter([package.license for package in packages])

        last_license = None
        print_license = False
        for pack in packages:
            pack_text = ""
            if self.by_package:
                mark = "\N{check mark}" if pack.license not in self.to_avoid else "x"

                print_license = (
                    self.print_fails and pack.license in self.to_avoid
                ) or not self.print_fails
                if print_license:
                    pack_text += f"{mark}  {pack}"
            else:
                if last_license != pack.license:
                    # u'/u2713'
                    mark = (
                        "\N{check mark}" if pack.license not in self.to_avoid else "x"
                    )

                    print_license = (
                        self.print_fails and pack.license in self.to_avoid
                    ) or not self.print_fails
                    if print_license:
                        pack_text += f"\n---{pack.license} [{license_count[pack.license]}]---  {mark}\n"

                last_license = pack.license

                if (
                    self.print_fails and pack.license in self.to_avoid
                ) or not self.print_fails:
                    pack_text += f"\t{pack.name}"

            if self._recursive and pack.requirements and print_license:
                pack_text += f"\t-> {pack.requirements}"

            if print_license:
                print(pack_text)
        print()

    def check_for_bad_license(self) -> int:
        """Tests if any of the user provided licences to avoid where found used by dependencies.

        Returns an exit code. (0 if none of the licenses to avoid were found in the projects
            dependencies and 1 otherwise)
        """

        unique_licenses = set(package.license for package in self._packages.values())

        for license in self.to_avoid:
            if license in unique_licenses:
                return 1

        return 0


def run_pylicense():
    """Run the pylicense algorithm."""

    to_avoid = None
    if Path("pyproject.toml").is_file():
        data: dict[str, Any] = tomlkit.parse(Path("pyproject.toml").read_text())
        if "pylicense" in data and "avoid" in data["pylicense"]:
            to_avoid = data["pylicense"]["avoid"]
            assert isinstance(
                to_avoid, list
            ), f"Expected avoid to have type list[str]. Found {type(to_avoid)}"
            assert all(
                isinstance(item, str) for item in to_avoid
            ), "All items of the list should be strings."

    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--recursive",
        "-r",
        action="store_true",
        default=False,
        help="Recursively find all the dependencies of the project and their licences.",
    )
    parser.add_argument(
        "--silent",
        "-s",
        action="store_true",
        default=False,
        help="Don't print any outputs.",
    )
    parser.add_argument(
        "--by-package",
        action="store_true",
        default=False,
        help="Group by package when printing.",
    )
    parser.add_argument(
        "--print-fails",
        "-f",
        action="store_true",
        default=False,
        help="Only print the packages whose licenses want to be avoided",
    )

    args = parser.parse_args()

    project = ProjectLicenses(
        args.recursive, args.by_package, args.print_fails, to_avoid
    )
    project.get_project_dependencies_and_licenses()

    if args.recursive:
        project.fetch_recursive_dependencies()

    if not args.silent:
        project.pretty_print()

    return project.check_for_bad_license()


# color code the licenses
# testing

if __name__ == "__main__":
    exit(run_pylicense())
