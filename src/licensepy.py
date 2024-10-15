#!/usr/bin/env python3

import argparse
from pathlib import Path
from typing import Any

import tomlkit

from .ProjectLicense import ProjectLicenses

# conda search --info numpy==1.26.4=py312hc5e2394_0
# c conda-forge
# conda list numpy to get the infor


def run_licensepy():
    """Run the licensepy algorithm."""

    to_avoid = None
    if Path("pyproject.toml").is_file():
        data: dict[str, Any] = tomlkit.parse(Path("pyproject.toml").read_text())
        if "licensepy" in data and "avoid" in data["licensepy"]:
            to_avoid = data["licensepy"]["avoid"]
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
        help="Group output by alphabetical order of package names.",
    )
    parser.add_argument(
        "--print-fails",
        "-f",
        action="store_true",
        default=False,
        help="Only print the packages whose licenses are flagged to be avoided.",
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
# pyproject finding
# testing

if __name__ == "__main__":
    exit(run_licensepy())
