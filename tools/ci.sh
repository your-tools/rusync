set -x
set -e
cargo build
cargo test
if [[ "${TRAVIS_RUST_VERSION}" == "nightly" ]]; then
  cargo clippy
fi
