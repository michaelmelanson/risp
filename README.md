# Risp

Risp is a programming environment. It's implemented in Rust and compiles to x86-64 at runtime to execute it.

This is SUPER hacky and will probably fail to run all but the simplest programs. It may even eat your homework.

<img src="images/screenshot.png">

## Features

- [x] Basic arithmetic expressions, e.g. `1 + 2 + 3 * 4` evaluates to `15`.
- [x] Function definitions and evaluation, e.g. `def add_one (x) { 1 + x } add_one(41)` evaluates to `42`.
- [x] Variable definitions, e.g. `let x = 3`.
- [x] Early return statements, e.g. `return 42`.
- [x] Conditional statements, e.g. `if x { return 1 }`.
- [ ] Assignment statements, e.g. `x = x + 1`.
- [ ] Boolean expressions, e.g. `x < 3` or `x == y`.
- [ ] While loops, e.g. `while x < 10 { x = x + 1 }`.

## How to use it

1. Install Rust via Rustup.
2. Build the project using `cargo build`.
3. Run the Risp environment with `cargo run`.
