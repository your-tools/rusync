# rusync

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
* Preserving permissions
* Option to delete extraneous files
* Progress bar when files are big (easy)
* Global progress bar (hard)

For the rest, well, patches are welcome!
