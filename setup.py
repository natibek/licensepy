import setuptools

from src._version import __version__

description = "licencepy is a Python dependency license check library with recursive dependency handling for pip."

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
    long_description=open("README.md").read(),
    long_description_content_type="text/markdown",
    packages=["src"],
    entry_points={
        "console_scripts": [
            "licensepy=src.licensepy:run_licensepy",
        ],
    },
    classifiers=[
        "Development Status :: 4 - Beta",
        "License :: OSI Approved :: Apache Software License",
        "Programming Language :: Python :: 3",
        "Operating System :: OS Independent",
    ],
)
