import io

import setuptools

__version__ = "0.0.1"

description = "licensepy is a license check tool for project package dependencies."

long_description = io.open("README.md", encoding="utf-8").read()

setuptools.setup(
    name="licensepy",
    version=__version__,
    url="https://github.com/natibek/licensepy/tree/main",
    author="Nathnael Bekele",
    author_email="nwtbekele@gmail.com",
    python_requires=(">=3.11.0"),
    install_requires=[
        "tomlkit==0.13.0",
    ],
    license="Apache 2.0",
    description=description,
    long_description=long_description,
    packages=["src"],
    entry_points={
        "console_scripts": [
            "licensepy=src.licensepy:run_licensepy",
        ],
    },
    classifiers=[
        "Development Status :: 4 - Beta",
        "License :: OSI Approved :: Apache 2.0",
        "Programming Language :: Python",
        "Operating System :: OS Independent",
    ],
)
