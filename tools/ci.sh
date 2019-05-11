set -x
set -e
cargo build --release
cargo test --release
