import subprocess
import sys
from importlib.metadata import distribution

# dependencies = subprocess.Popen(["pip", "freeze"], stdout=subprocess.PIPE)
# dependencies = dependencies.stdout.read().decode("utf-8")
common_licenses = [
    "MIT",
    "Apache",
    "BSD",
    "GPL",
    "LGPL",
]


def get_dependencies() -> list[str]:
    dependencies = subprocess.check_output(["pip3", "freeze"]).decode("utf-8")
    dependencies = [dep.split("==")[0] for dep in dependencies.split("\n") if dep]
    return dependencies


def get_package_requirements(package_name: str) -> list[str]:
    package_info = subprocess.check_output(["pip3", "show", package_name]).decode(
        "utf-8"
    )

    for info in package_info.split("\n"):
        if info.startswith("Requires"):
            package_requirements = info.split(": ")[1].split(", ")
            break
    return package_requirements


def get_license(package: str) -> str:
    if not (license := distribution(package).metadata["License"]) or len(license) > 10:
        for value in distribution(package).metadata.get_all("Classifier"):
            if "License" in value:
                license = value.split(" :: ")[-1]
                break
    return license.replace("License", "").strip() if license else "?"


def get_licenses():
    dependencies = get_dependencies()
    licenses = {}

    for package in dependencies:
        print(f"{package}: {get_license(package)}")
        licenses[package] = get_license(package)

        print(f"{package}: {get_package_requirements(package)}")


get_licenses()
