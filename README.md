# Introduction

This is a simple app to detect missing type hints in function definitions for Python
file(s).

It was primarily done for education purposes to learn more about [tree-sitter](https://tree-sitter.github.io/tree-sitter/).

## Usage

To run it, you can cargo install the package and then run the following command:

```
pythcheck {FILE_PATH}
```

This will check for missing type hints on functions of the given file.

If you pass a directory, it will check for all Python files in that
directory recursively. For more information, run `pythcheck -h`.

```
$ pythcheck -h
Checks Python files for missing type hints in function parameters and return values.

Usage: pythcheck [OPTIONS] <PATH>

Arguments:
  <PATH>  File or directory to check

Options:
      --ignore-hidden  Ignores hidden subdirectories and files
      --ignore-tests   Ignores tests subdirectories and files
      --ignore-return  Ignores absence of return type hints
  -h, --help           Print help
  -V, --version        Print version
```
