# Linux source querying tool

[![github](https://img.shields.io/badge/github-katyo/kquery-8da0cb.svg?style=for-the-badge&logo=github)](https://github.com/katyo/kquery)
[![crates](https://img.shields.io/crates/v/kquery.svg?style=for-the-badge&color=fc8d62&logo=rust)](https://crates.io/crates/kquery)
[![docs](https://img.shields.io/badge/docs.rs-kquery-66c2a5?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K)](https://docs.rs/kquery)
[![status](https://img.shields.io/github/workflow/status/katyo/kquery/Rust?style=for-the-badge&logo=github-actions&logoColor=white)](https://github.com/katyo/kquery/actions?query=workflow%3ARust)

This is blazing fast querying tool for deep dive into Linux source code.
Development in early stage so things may work wrong or does not work at all.

Currently it consists of querying library and simple command-line tool.

## Command-line usage

Create or update index first:

```shell
$ cd path/to/linux/sources
$ kquery index
```

Query source info which has compatible string:

```shell
$ kquery compat compat-string
```

Query sources info related to config options:

```shell
$ kquery config CONFIG_OPTION
```

Query source info by path:

```shell
$ kquery source path/to/source.c
```

## Library usage

```no_run
use kquery::{FileMgr, MetaData};

// Create file manager instance using path to sources
let filemgr = FileMgr::new("path/to/sources").await?;

// Create index from Linux source tree
let metadata = MetaData::from_kbuild(&filemgr).await?;

// Store metadata into cache file in source tree
metadata.save_cache(&filemgr).await?;

// Load metadata from cache file in source tree
let metadata = MetaData::from_cache(&filemgr).await?;
```