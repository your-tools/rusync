# v0.2.1

This contains several bug fixes regarding symlinks.

Here the algorithm when now use:

* If the destination does not exists:
  * Create a new symlink with the same target as the previous one

* If the destination exists:

  * If it's not a symlink:
      * Abort!

  * Otherwise:

    * If the destination symlink already has the correct target, consider it up to date.
    * If the destination symlink is broken, remove it and re-create it
    * If the destination symlink does not point to the correct location, remove it and re-create it.

# v0.2.0

* Try and preserve permissions after files are copied

# v0.1.2

* Add missing call to `stdout().flush()`

# v0.1.1

* Display a progress bar for each file

# v0.1.0

Initial release
