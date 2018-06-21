# v0.4.1

* Improve error handling: display more details about the file operation that failed
  instead of just the raw io::Error

# v0.4.0

* Display an ETA at the right of the progress bar.

# v0.3.1

* Exit early if the source given on the command line is not an argument. We used to display a weird
  "0 files copied" in this case.

# v0.3.0

* Change output to be like a Ninja. Print all progress on one line, and erase it when done.

The line looks like:

```
 50% 24/50 Downloads/archlinux.iso
```

It contains the percentage of the current file that has been transfered, the index of the current transfered,
the total number of files to copy, and the name of the current file.

Note that the number of files to copy may increase while rusync is running: this is because the contents
of the source folder are read *while the copy is done*.


# v0.2.3

* Add a `--no-perms` flag to disable preservation of permissions. Useful when
  you *know* this will fail and don't want to be flooded with warning messages.

# v0.2.1

This contains several bug fixes regarding symlinks.

Here the algorithm when now use:

* If the destination does not exists:
  * Create a new symlink with the same target as the previous one.

* If the destination exists:

  * If it's not a symlink:
      * Abort!

  * Otherwise:

    * If the destination symlink already has the correct target, consider it up to date.
    * If the destination symlink is broken, remove it and re-create it.
    * If the destination symlink does not point to the correct location, remove it and re-create it.

# v0.2.0

* Try and preserve permissions after files are copied

# v0.1.2

* Add missing call to `stdout().flush()`

# v0.1.1

* Display a progress bar for each file

# v0.1.0

Initial release
