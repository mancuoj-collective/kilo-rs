default:
    @just --list

fmt:
    cargo fmt --all

lint:
    cargo clippy --all-targets --all-features -- -D warnings

c:
    cc kilo.c -o kilo -Wall -Wextra -pedantic -std=c99
