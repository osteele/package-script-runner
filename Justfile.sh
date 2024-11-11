create-test-projects:
    ./scripts/create-test-projects.sh

fmt:
    cargo fmt --all

clippy:
    cargo clippy --allow-dirty

test:
    cargo test

fix: fmt clippy test
