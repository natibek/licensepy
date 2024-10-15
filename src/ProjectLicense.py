import platform
import re
import subprocess
from collections import Counter
from importlib.metadata import distribution

from .Package import Package
from .pretty_string import *


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
        self.to_avoid = to_avoid if to_avoid else []  # list of licenses to avoid

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
        dependencies = [
            re.split(r"==|@", dep)[0].strip() for dep in dependencies.split("\n") if dep
        ]
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

        version = re.search(r"python_version(==|<=|>=|!=|<|>)\d\.\d(\.\d)?", expression)
        assert version
        version = version.group(0)
        version = re.split(r"(==|<=|>=|!=|<|>)", version)[2].strip().split(".")

        if len(version) == 2:
            version.append(self._python_version[2])

        diff = [
            int(ver) - int(p_ver) for ver, p_ver in zip(version, self._python_version)
        ]

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
            not (license := distribution(package_name).metadata.get("License"))
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

        return license.replace("License", "").strip() if license else "?"

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

    def _requirements_to_str(self, requirements: list[str]) -> str:
        """Generate the string representation of the requirements for a package. Colors them
        green if they are not in the package's license is not in the list to avoid and red if
        it is.

        Args:
            requirements: the package requirements for a package.

        Returns the string representation.
        """

        req_text = [
            (
                failure(req)
                if self._packages[req].license in self.to_avoid
                else success(req)
            )
            for req in requirements
        ]
        return " [" + ", ".join(req_text) + "]"

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
                mark = (
                    success("\N{check mark}")
                    if pack.license not in self.to_avoid
                    else failure("x")
                )

                print_license = (
                    self.print_fails and pack.license in self.to_avoid
                ) or not self.print_fails
                if print_license:
                    pack_text += f"{mark}  {pack}"
            else:
                if last_license != pack.license:
                    # u'/u2713'
                    mark = (
                        success("\N{check mark}")
                        if pack.license not in self.to_avoid
                        else failure("x")
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
                pack_text += self._requirements_to_str(pack.requirements)

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
