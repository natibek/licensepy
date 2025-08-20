# licensepy
## Python Depedency License Check and License Header Formating.

**_licensepy_** is a Python dependency license check and license header check/format library written in Rust. This package has recursive dependency checks that are not offered by many existing license check libraries. By default, the output will group packages by their licenses.

<!--![](https://raw.githubusercontent.com/natibek/licensepy/main/imgs/licensepy_output.png)-->

Configure the tool with a _pyproject.toml_ file at the root directory of the project with a list of licenses to avoid. If dependencies of the project are found to use these flagged licenses, **licensepy** will exit with the count of these projects. Otherwise, it will exit with code 0. For the license header formatter, the exit code will be the count of the source code files with incorrect header.

## Installing

Use pip to install **licensepy** in your project.

```bash
$ pip3 install licensepy
```

## Command Line Arguments

### Dependency license check
For the dependency license check, the following are the command line arguments.

```bash
$ licensepy check
```
1. -r, --recursive: Recursively find all the dependencies of the project and their licences.
   <!--![](https://raw.githubusercontent.com/natibek/licensepy/main/imgs/licensepy_recursive.png)-->
   - Recursive dependencies will have the color red if they have licenses that have been flagged to avoid and green otherwise.
     <!--![](https://raw.githubusercontent.com/natibek/licensepy/main/imgs/licensepy_recursive_avoid_mit.png)-->
1. -by-package: Group output by packages in alphabetical order.
   <!--![](https://raw.githubusercontent.com/natibek/licensepy/main/imgs/licensepy_output_by_package.png)-->
1. -s, silent: Silence all outputs.
1. -f, print-fails: Only print the packages whose licenses are flagged to be avoided.
1. -j, --num-threads: Number of threads to use. Default is 1. Max is 32 [default: 1].

### License header checker and formatter

For the license
```bash
$ licensepy format
```

1. files: Positional arguments are the Python files to run license header checker/formatter.
1. -l, --licensee: Licensee. Has precedence over value from config.
1. -y, --license-year: License year. Has precedence over value from config.
1. -s, --silent: Don't print any outputs. Default if false.
1. -d, --dry-run: Don't run formatter. Only print outputs. Default if false.
1. -j, --num-threads: Number of threads to use. Default is 1. Max is 32 [default: 1].

## Configuration

Licenses can be flagged to avoid in a pyproject.toml files saved in the root of the project directory. Licenses should be stored in a list.

```toml
# In the pyproject.toml file

[tool.licensepy]
# List of licenses to avoid.
avoid = ["MIT"]

# header template: The template for the license header. {year}, {licensee}
# are placeholders that will be populated with values from command line
# or the config. The template can have the # at the beginning or not.
license_header_template = "# Copyright {year} {licensee}"

# license_year: The value to replace the {year} in the license_header_template.
# Default value is the current year.
license_year = 2025

# licensee: The value to replace the {licensee} in the license_header_template.
# If the {licensee} placeholder is found in the template and the licensee field
# or the command line argument are
licensee = "Nati"
```

This is the output when the above configuration is used for:

```bash
$ licensepy check
```

![](https://raw.githubusercontent.com/natibek/licensepy/main/imgs/licensepy_avoid_mit.png)

```bash
$ licensepy check --by-package
```

![](https://raw.githubusercontent.com/natibek/licensepy/main/imgs/licensepy_by_package_avoid_MIT.png)
