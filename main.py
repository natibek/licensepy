import platform
import re
import subprocess
from importlib.metadata import PackageNotFoundError, distribution


class Package:
    def __init__(self, name: str, license: str) -> None:
        self.name = name
        self.license = license

    def __str__(self) -> str:
        return f"{self.name} -> {self.license}"

    @property
    def requirements(self) -> list[str]:
        return self._requirements

    @requirements.setter
    def requirements(self, reqs: list[str]) -> None:
        self._requirements = reqs


class ProjectLicenses:
    def __init__(self, depth: int) -> None:
        self._python_version = platform.python_version()
        self._depth = depth
        self._package_libraries: dict[str, Package] = {}  # map package name to object
        self._project_dependencies: list[str] = (
            []
        )  # direct dependencies names of project

    def find_project_dependencies(self) -> list[str]:
        dependencies = subprocess.check_output(["python", "-m" "pip", "freeze"]).decode(
            "utf-8"
        )
        dependencies = [dep.split("==")[0] for dep in dependencies.split("\n") if dep]
        return dependencies

    def _matching_version(self, req_info: str) -> bool:
        expression = req_info.split(";")[1]
        version = re.split(r"[<>=]", expression)[1].strip().split(".")
        if len(version) == 2:
            version.append("0")
        
        diff = [int(ver) - int(p_ver) for ver, p_ver in zip(version, self._python_version)]
        print(diff)
        if "<=" in expression:
            return diff[0] > 0 or (diff[0] == 0 and diff[1] >= 0)
        elif "<" in expression:
            return diff[0] > 0 or (diff[0] == 0 and diff[1] > 0)
        elif ">=" in expression:
            return diffqqqqq q
        elif ">" in expression:

        elif "==" in expression:

        elif "!=" in expression:

        else:
            print(expression)
        return True

    def get_package_requirements(self, package_name: str) -> list[str]:
        package_requirements = []
        try:
            if req_info := distribution(package_name).metadata.get_all("Requires-Dist"):
                for req in req_info:
                    if ";" not in req or ("; python_version" in req) and self._matching_version(req):
                        package_requirements.append(re.split(r"[<>=~\(;!]", req)[0].strip())
                # package_requirements = [
                #     re.split(r"[<>=~\(;!]", req)[0].strip()
                #     for req in req_info
                #     if ";" not in req
                #     or "; python_version" in req
                #     and self._matching_version(req)
                #     # check the version and if it matches the current one
                #     # requirements for only certain python versions are noted with ; python_version
                # ]
        except PackageNotFoundError:
            return []

        return package_requirements

    def get_license(self, package: str) -> str:
        license = None
        if package in self._package_libraries:
            return self._package_libraries[package].license
        try:
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
        except:
            try:
                classifier = distribution(package).metadata.get_all("Classifier")
                if not classifier:
                    # edge case when package does not have classifier information
                    return "?"

                for value in classifier:
                    if "License" in value:
                        license = value.split(" :: ")[-1]
                        break
            except:
                pass
        return license.replace("License", "").strip() if license else "?"

    def get_project_dependencies_and_licenses(self):
        dependencies = self.find_project_dependencies()
        for package in dependencies:
            print(package)
            cur_package = Package(package, self.get_license(package))
            cur_package.requirements = self.get_package_requirements(package)

            self._package_libraries[package] = cur_package
            self._project_dependencies.append(package)

    def fetch_recursive_dependencies(self):
        queue = self._project_dependencies[:]

        while queue:
            package = queue.pop()

            if not package in self._package_libraries:
                cur_package = Package(package, self.get_license(package))
                cur_package.requirements = self.get_package_requirements(package)
                self._package_libraries[package] = cur_package

            for req in self._package_libraries[package].requirements:
                if req not in self._package_libraries:
                    queue.append(req)

    def print_lib(self):
        pass

    def pretty_print(self):
        print(self._package_libraries)
        for p, pack in self._package_libraries.items():
            print(pack)
            print(f"\treqs -> {pack.requirements}")

        print(self._project_dependencies)


if __name__ == "__main__":
    project = ProjectLicenses(1)
    project.get_project_dependencies_and_licenses()
    project.fetch_recursive_dependencies()
    project.pretty_print()
# depth of recursion with -r flag
