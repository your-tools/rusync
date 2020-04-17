# rusync

<a href="https://crates.io/crates/rusync"><img src="https://img.shields.io/crates/v/rusync.svg"/></a>
<a href="https://travis-ci.org/dmerejkowsky/rusync"><img src="https://api.travis-ci.org/dmerejkowsky/rusync.svg?branch=master"/></a>

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

* Easy to remember command line syntax

* Print progress on one line, and erase it when done, thus avoiding flooding your terminal
  with useless noise.
  
* Displays a reliable ETA, without sacrificing speed.

* Un-surprising behavior: missing directories are created
  on the fly, files are only copied if:

  * Destination is missing
  * Older than the source
  * Or size is different


# State of the project

I consider this project *done* - I don't intend on adding new features. The goal was to learn more about Rust and I've learned plenty of things already. If there's a feature present in `rsync` that is not available in `rusync`, just use `rsync`  - or try and implement the feature yourself, I'll be happy to review and merge your changes :)
