# rusync

<a href="https://crates.io/crates/rusync"><img src="https://img.shields.io/crates/v/rusync.svg"/></a>
<a href="https://travis-ci.org/dmerejkowsky/rusync"><img src="https://api.travis-ci.org/dmerejkowsky/rusync.svg?branch=master"/></a>

Minimalist `rsync` clone implemented in Rust.

# Caveat

We do everything we can to make sure data loss is impossible, but despite our best efforts, it may still happen.

Please make sure your files files are backed up if necessary before using `rusync` on sensitive data.

Thank you for your understanding!

# Usage

```
$ cargo install rusync
$ rusync test/src test/dest
:: Syncing from test/src to test/dest …
 50% 24/50 Downloads/archlinux.iso   00:01:30
```

# Features

* Easy to remember command line syntax

* Print progress on one line, and erase it when done, thus avoiding flooding your terminal
  with useless noise.

* Un-surprising behavior: missing directories are created
  on the fly, files are only copied if:

  * Destination is missing
  * Older than the source
  * Or size is different

* Minimalistic implementation

# Missing

There are *tons* of stuff in `rsync` we don't implement.

But for me, the goal was to learn more about Rust and I've learned plenty of things already.

The big missing feature is an option to delete extraneous files. Maybe I'll start working on it one day.

Some people have asked for transfer over `ssh`. Not sure if that can be easily done. Maybe try using `rusync` normally on top of `sshfs`?

For the rest, well, patches are welcome!
