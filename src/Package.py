from dataclasses import dataclass
from importlib.metadata import PackageMetadata


@dataclass
class Package:
    """Used to store a package's information including name, license, and packages it requires."""

    name: str  # name of the package
    license: str # license of the package
    requirements: list[str] 

    def __str__(self) -> str:
        return f"{self.name} ({self.license})"

