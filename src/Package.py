from dataclasses import dataclass


@dataclass
class Package:
    """Used to store a package's information including name, license, and packages it requires."""

    name: str  # name of the package
    license: str  # license of the package

    def __str__(self) -> str:
        return f"{self.name} ({self.license})"

    @property
    def requirements(self) -> list[str]:
        """Returns the list of packages the package requires."""
        return self._requirements

    @requirements.setter
    def requirements(self, reqs: list[str]) -> None:
        self._requirements = reqs
