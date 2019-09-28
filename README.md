<a href="https://github.com/open-flash/open-flash">
    <img src="https://raw.githubusercontent.com/open-flash/open-flash/master/logo.png"
    alt="Open Flash logo" title="Open Flash" align="right" width="64" height="64" />
</a>

# AVM1 Parser

[![npm](https://img.shields.io/npm/v/avm1-parser.svg)](https://www.npmjs.com/package/avm1-parser)
[![crates.io](https://img.shields.io/crates/v/avm1-parser.svg)](https://crates.io/crates/avm1-parser)
[![GitHub repository](https://img.shields.io/badge/Github-open--flash%2Favm1--parser-blue.svg)](https://github.com/open-flash/avm1-parser)
[![Build status](https://img.shields.io/travis/com/open-flash/avm1-parser/master.svg)](https://travis-ci.com/open-flash/avm1-parser)

AVM1 parser implemented in Rust and Typescript (Node and browser).
Converts bytes to [`avm1-types` control flow graphs][avm1-types].

- [Rust implementation](./rs/README.md)
- [Typescript implementation](./ts/README.md)

This library is part of the [Open Flash][ofl] project.

## Usage

- [Rust](./rs/README.md#usage)
- [Typescript](./ts/README.md#usage)

## Status

The raw-action parser is complete.
The CFG parser still needs some work and feedback.

## Contributing

Each implementation lives in its own directory (`rs` or `ts`). The commands
must be executed from these "project roots", not from the "repo root".

Check the implementation-specific guides:

- [Rust](./rs/README.md#contributing)
- [Typescript](./ts/README.md#contributing)

You can also use the library and report any issues you encounter on the Github
issues page.

[ofl]: https://github.com/open-flash/open-flash
[avm1-types]: https://github.com/open-flash/avm1-types
