# rusync

<a href="https://crates.io/crates/rusync"><img src="https://img.shields.io/crates/v/rusync.svg"/></a>
<a href="https://github.com/dmerejkowsky/rusync/actions"><img src="https://github.com/dmerejkowsky/rusync/workflows/Run%20tests/badge.svg"/></a>
<a href="https://github.com/dmerejkowsky/rusync/actions"><img src="https://github.com/dmerejkowsky/rusync/workflows/Run%20linters/badge.svg"/></a>
<a href="https://github.com/dmerejkowsky/rusync/actions"><img src="https://github.com/dmerejkowsky/rusync/workflows/Audit%20dependencies/badge.svg"/></a>


Minimalist `rsync` implementation in Rust.

# Usage

```
$ cargo install rusync
$ rusync test/src test/dest
:: Syncing from test/src to test/dest …
 50% 24/50 Downloads/archlinux.iso   00:01:30
```

# Caveat

We do everything we can to make sure data loss is impossible, but despite our best efforts, it may still happen.

Please make sure your files files are backed up if necessary before using `rusync` on sensitive data.

Thank you for your understanding!

# Features

* Easy to remember command line syntax.

* Print progress on one line, and erase it when done, thus avoiding flooding your terminal
  with useless noise.

* Displays a reliable ETA, without sacrificing speed.

* Unsurprising behavior: missing directories are created
  on the fly, files are only copied if:

  * destination is missing
  * destination exists but is older than the source
  * or source and destination have different sizes

# Command line options

Just two at the moment:

* `--no-perms`: prevents`rusync` from trying to preserve file permissions (useful if you copy data from a Linux partition to NTFS for instance).
* `--err-list FILE`: write name of entries that caused errors in the given file, separated by `\n`


# State of the project

I consider this project *done* - I don't intend on adding new features. The goal was to learn more about Rust and I've learned plenty of things already. If there's a feature present in `rsync` that is not available in `rusync`, just use `rsync`  - or try and implement the feature yourself, I'll be happy to review and merge your changes :)
