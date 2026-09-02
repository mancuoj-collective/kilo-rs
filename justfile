default:
    @just --list

c:
    cc kilo.c -o kilo -Wall -Wextra -pedantic -std=c99

fmt:
    cargo fmt --all

lint:
    cargo clippy --all-targets --all-features -- -D warnings
