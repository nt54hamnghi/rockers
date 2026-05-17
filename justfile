[private]
default:
    @just --list --justfile {{ justfile() }} --unsorted

# lint code with clippy and rustfmt
lint:
    cargo clippy --all-targets -- -D clippy::all -W clippy::pedantic
    cargo +nightly fmt --check

alias l := lint

# format code with rustfmt
fmt:
    cargo +nightly fmt

alias f := fmt

run:
    cargo build --bin=main
    sudo ./target/debug/main run /bin/sh

alias r := run
