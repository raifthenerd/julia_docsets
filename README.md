# julia_docsets

[![Build Status](https://img.shields.io/github/actions/workflow/status/raifthenerd/julia_docsets/CI.yml)](https://github.com/raifthenerd/julia_docsets/actions/workflows/CI.yml?query=branch%3Amain)
[![License](https://img.shields.io/github/license/raifthenerd/julia_docsets)](LICENSE)

## How to Use

```bash
> cargo build --release
> git submodule update --init --recursive
> git submodule foreach git pull origin gh-pages
> TMPDIR=./target ./target/release/build_docsets
```

## For Developers

### How to Setup Development Environment

First, you should install [pre-commit] and [Cocogitto] first.
After installing the prerequisites, execute the following shell commands:

```bash
> pre-commit install
> cog install-hook --all
```

[pre-commit]: https://pre-commit.com/
[Cocogitto]: https://docs.cocogitto.io/
