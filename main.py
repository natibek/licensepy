import re
import subprocess
from importlib.metadata import distribution


class Package:
    def __init__(self, name: str, license: str) -> None:
        self.name = name
        self.license = license

    @property
    def requirements(self) -> list[str]:
        return self._requirements

    @requirements.setter
    def requirements(self, reqs: list[str]) -> None:
        self._requirements = reqs


class ProjectLicenses:
    def __init__(self, depth: int) -> None:
        self._depth = depth
        self._package_libraries: dict[str, Package] = {}  # map package name to object
        self._procjet_dependencies: list[str] = (
            []
        )  # direct dependencies names of project

    def find_project_dependencies(self) -> list[str]:
        dependencies = subprocess.check_output(["python", "-m" "pip", "freeze"]).decode(
            "utf-8"
        )
        dependencies = [dep.split("==")[0] for dep in dependencies.split("\n") if dep]
        return dependencies

    def get_package_requirements(self, package_name: str) -> list[str]:
        package_requirements = []
        if req_info := distribution(package_name).metadata.get_all("Requires-Dist"):
            package_requirements = [
                re.split(r"[<>=~\(;!]", req)[0].strip()
                for req in req_info
                if ";" not in req or "; python_version" in req
                # requirements for only certain python versions are noted with ; python_version
            ]
        print(package_requirements)
        return package_requirements

    def get_license(self, package: str) -> str:
        if package in self._package_libraries:
            return self._package_libraries[package].license

        if (
            not (license := distribution(package).metadata["License"])
            or len(license) > 10
        ):
            classifier = distribution(package).metadata.get_all("Classifier")
            if not classifier:
                # edge case when package does not have classifier information
                return "?"

            for value in classifier:
                if "License" in value:
                    license = value.split(" :: ")[-1]
                    break
        return license.replace("License", "").strip() if license else "?"

    def get_project_dependencies_and_licenses(self):
        dependencies = self.find_project_dependencies()
        for package in dependencies:
            print(f"{package}: {self.get_license(package)}")
            cur_package = Package(package, self.get_license(package))
            cur_package.requirements = self.get_package_requirements(package)

            self._package_libraries[package] = cur_package
            self._procjet_dependencies.append(package)

    def fetch_recursive_dependencies(self):
        queue = self._procjet_dependencies[:]

        while queue:
            package = queue.pop()

            if not package in self._package_libraries:
                cur_package = Package(package, self.get_license(package))
                cur_package.requirements = self.get_package_requirements(package)
                self._package_libraries[package] = cur_package

            for req in self._package_libraries[package].requirements:
                if req not in self._package_libraries:
                    queue.append(req)

    def pretty_print(self):
        pass


# depth of recursion with -r flag
