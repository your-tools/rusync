#!/bin/bash

set -e

src=tests/data
dest=/tmp/dest

reset_test_data_folder() {
  git checkout -- $src
}

cleanup_dest() {
  rm -fr $dest
}

run_rusync() {
  cargo run --quiet $src $dest
}

change_link_dest() {
  (
    cd ${src}/a_dir
    ln -sf two.txt link_to_one
  )
}

main() {
  cleanup_dest
  run_rusync
  change_link_dest
  run_rusync
  reset_test_data_folder
}

main
