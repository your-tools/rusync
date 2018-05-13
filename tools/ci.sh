set -x
set -e
cargo build
cargo test
if [[ "${TRAVIS_RUST_VERSION}" == "nightly" ]]; then
  cargo install clippy
  cargo clippy
fi
