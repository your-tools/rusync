# rusync

<a href="https://crates.io/crates/rusync"><img src="https://img.shields.io/crates/v/rusync.svg"/></a>
<a href="https://travis-ci.org/dmerejkowsky/rusync"><img src="https://api.travis-ci.org/dmerejkowsky/rusync.svg?branch=master"/></a>

`rsync` implemented in rust.

# Usage

```
$ cargo install rusync
$ rusync test/src test/dest
:: Syncing from test/src to test/dest …
-> foo/baz.txt
-> foo/bar.txt
 ✓ Synced 2 files (1 up to date)
```

# Features

* Easy to remember command line syntax
* Colorful and *useful* output
* Un-surprising behavior: missing directories are created
  on the fly, files are only copied if destination is missing or older than
  the source
* Minimalistic implementation

# Missing

There are *tons* of stuff in `rsync` we don't implement. Here's what's missing and I think `rusync` should have:

* Symlinks handling
* Option to delete extraneous files
* Global progress bar (hard)

For the rest, well, patches are welcome!
