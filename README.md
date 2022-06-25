<a href="https://github.com/open-flash/open-flash">
    <img src="https://raw.githubusercontent.com/open-flash/open-flash/master/logo.png"
    alt="Open Flash logo" title="Open Flash" align="right" width="64" height="64" />
</a>

# AVM1 Parser

AVM1 parser implemented in Rust and Typescript (Node and browser).
Converts bytes to [`avm1-types` control flow graphs][avm1-types].

<table>
<thead>
  <tr>
    <th>Implementation</th>
    <th>Package</th>
    <th>Checks</th>
    <th>Documentation</th>
  </tr>
</thead>
<tbody>
  <tr>
    <td>
      <a href="./rs/README.md">Rust</a>
    </td>
    <td>
      <a href="https://crates.io/crates/avm1-parser"><img src="https://img.shields.io/crates/v/avm1-parser" alt="crates.io crate"/></a>
    </td>
    <td>
      <a href="https://github.com/open-flash/avm1-parser/actions/workflows/check-rs.yml"><img src="https://img.shields.io/github/workflow/status/open-flash/avm1-parser/check-rs/main"  alt="Rust checks status"/></a>
    </td>
    <td>
      <a href="https://docs.rs/avm1-parser"><img src="https://img.shields.io/badge/docs.rs-avm1--parser-informational" alt="docs.rs/avm1-parser"></a>
    </td>
  </tr>
  <tr>
    <td>
      <a href="./ts/README.md">TypeScript</a>
    </td>
    <td>
      <a href="https://www.npmjs.com/package/avm1-parser"><img src="https://img.shields.io/npm/v/avm1-parser" alt="npm package"/></a>
    </td>
    <td>
      <a href="https://github.com/open-flash/avm1-parser/actions/workflows/check-ts.yml"><img src="https://img.shields.io/github/workflow/status/open-flash/avm1-parser/check-ts/main"  alt="TypeScript checks status"/></a>
    </td>
    <td>
      <a href="./ts/src/lib">Source Code ¯\_(ツ)_/¯</a>
    </td>
  </tr>
</tbody>
</table>

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
