import io

import setuptools

__version__ = "0.0.1"

description = "pylicense is a license check tool for project package dependencies."

long_description = io.open("README.md", encoding="utf-8").read()

requirements = open("requirements.txt").readlines()
requirements = [req.strip() for req in requirements]

setuptools.setup(
    name="pylicense",
    version=__version__,
    url="https://github.com/natibek/pylicense/tree/main",
    author="Nathnael Bekele",
    author_email="nwtbekele@gmail.com",
    python_requires=(">=3.11.0"),
    install_requires=requirements,
    license="Apache 2.0",
    description=description,
    long_description=long_description,
    packages=["src"],
    scripts=["scr/pylicense"],
    classifiers=[
        "License :: OSI Approved :: Apache 2.0",
        "Programming Language :: Python",
        "Operating System :: OS Independent",
    ],
)
