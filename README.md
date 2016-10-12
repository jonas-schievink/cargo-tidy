# Tidy up your project! [![Build Status](https://travis-ci.org/jonas-schievink/cargo-tidy.svg?branch=master)](https://travis-ci.org/jonas-schievink/cargo-tidy)

This is a small Cargo subcommand, `cargo tidy`, which allows you to configure
various code style checks to be executed. You can then put it in CI to make sure
that all PRs adhere to the rules.

An example configuration can be found in
[tidy-config.rs.toml](./tidy-config.rs.toml).

For example, `tidy` allows you to check:

* That a line doesn't exceed a maximum line length
* That a file only uses `\n` as line endings
* That only spaces and no tabs are used for indentation

## Installation

Tidy can be installed like any other crate:
```
cargo install cargo-tidy
```

## CI Integration

The `cargo tidy` project itself uses `cargo tidy` to check its own code style!

An example `.travis.yml` might look like this:

```yml
language: rust
sudo: false
before_script:
  - export PATH=$HOME/.cargo/bin:$PATH && cargo install cargo-tidy
script:
  - cargo test
  - cargo tidy -c my-tidy-config.toml
```

## Different languages, different rules

When working on a large-scale project which combines multiple programming
languages, you might want to run multiple sets of style checks. Or maybe you
just want different rules for `.md` files than for `.rs` files.

Since `cargo tidy` allows specifying custom file globs, you can write multiple
configuration files for different file types (or folders).
