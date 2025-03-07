# licensepy: Python License Check Module.

**_licensepy_** is a Python dependency license check library written in Rust. This package has recursive dependency checks that are not offered by many existing license check libraries. By default, the output will group packages by their licenses.

![](https://raw.githubusercontent.com/natibek/licensepy/main/imgs/licensepy_output.png)

Command line arguments can be used to recursively find all the dependencies of the project (-r, --recursive), group output by packages in alphabetical order (--by-package), silence all outputs (-s, --silent), or only print fails (-f, --print-fails).

Configure the tool with a _pyproject.toml_ file at the root directory of the project with a list of licenses to avoid. If any one the dependencies of the project use these flagged licenses, **licensepy** will exit with code 1. Otherwise, it will exit with code 0.

## Installing

Use pip to install **licensepy** in your project.

```bash
pip3 install licensepy

```

## Command Line Arguments

1. -r, --recursive: Recursively find all the dependencies of the project and their licences.
   ![](https://raw.githubusercontent.com/natibek/licensepy/main/imgs/licensepy_recursive.png)
   - Recursive dependencies will have the color red if they have licenses that have been flagged to avoid and green otherwise.
     ![](https://raw.githubusercontent.com/natibek/licensepy/main/imgs/licensepy_recursive_avoid_mit.png)
1. -by-package: Group output by packages in alphabetical order.
   ![](https://raw.githubusercontent.com/natibek/licensepy/main/imgs/licensepy_output_by_package.png)
1. -s, silent: Silence all outputs.
1. -f, print-fails: Only print the packages whose licenses are flagged to be avoided.

## Configuration

Licenses can be flagged to avoid in a pyproject.toml files saved in the root of the project directory. Licenses should be stored in a list.

```toml
# In the pyproject.toml file

[licensepy]
avoid = ["MIT"]
```

This is the output when the above configuration is used for:

1.  ```bash
    licensepy
    ```

    ![](https://raw.githubusercontent.com/natibek/licensepy/main/imgs/licensepy_avoid_mit.png)

1.  ```bash
    licensepy --by-package
    ```

    ![](https://raw.githubusercontent.com/natibek/licensepy/main/imgs/licensepy_by_package_avoid_MIT.png)
