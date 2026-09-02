default:
    @just --list

fmt:
    cargo fmt --all

check:
    cargo check

test:
    cargo test

lint:
    cargo clippy --all-targets --all-features -- -D warnings

run *args:
    cargo run -- {{args}}

c:
    cc kilo.c -o kilo -Wall -Wextra -pedantic -std=c99
