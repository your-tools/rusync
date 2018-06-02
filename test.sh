#!/bin/bash

set -xe

rm -fr ~/tmp/big2/
cargo run ~/tmp/big ~/tmp/big2
echo "corrupted" > ~/tmp/big2/big.avi
sleep 1
# rust-gdb target/debug/rusync ~/tmp/big ~/tmp/big2
cargo run ~/tmp/big ~/tmp/big2
