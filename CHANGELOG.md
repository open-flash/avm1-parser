# 0.14.0 (2022-06-25)

- **[Breaking change]** Update to `swf-types@0.14`.
- **[Internal]** Migrate from Travis CI to GitHub Actions.

## Rust

- **[Breaking change]** Require Rust `1.60.0`.
- **[Breaking change]** Update to `nom@7`.
- **[Fix]** Update dependencies.

# Typescript

- **[Breaking change]** Compile to `.mjs`.
- **[Fix]** Update dependencies.
- **[Internal]** Use Yarn's Plug'n'Play linker.

# 0.13.0 (2021-07-24)

## Rust

- **[Breaking change]** Update to `swf-types@0.13`.
- **[Breaking change]** Update to `nom@6`.
- **[Fix]** Update dependencies.
- **[Internal]** Add Clippy support.

## Typescript

- **[Breaking change]** Update to `avm1-types@0.13`.
- **[Breaking change]** Drop `lib` prefix and `.js` extension from deep-imports.
- **[Fix]** Update dependencies.

# 0.12.0 (2021-05-05)

- **[Breaking change]** Update to `avm1-types@0.12.0`.
- **[Feature]** Add support for the `StrictMode` action.
- **[Fix]** Fix support for `float64` push value.

## TypeScript

- **[Internal]** Update to Yarn 2.

# 0.11.0 (2020-09-07)

- **[Breaking change]** Update to `avm1-types@0.11.0`.

## Rust

- **[Internal]** Move binary out of the library crate.

## TypeScript

- **[Breaking change]** Update to native ESM.
- **[Internal]** Switch from `tslint` to `eslint`.

# 0.10.0 (2020-02-20)

- **[Breaking change]** Update to `avm1-types@0.10.0`.
- **[Breaking change]** Use DFS order for layer numbering.

## Rust

- **[Feature]** Implement CFG parser.
- **[Fix]** Update to `nom@5`.

# 0.9.1 (2019-09-28)

## Typescript

- **[Fix]** Add support for fall-through over empty blocks (especially over empty but defined `finally`).

# 0.9.0 (2019-09-27)

- **[Breaking change]** Update to `avm1-types@0.9` (former `avm1-tree`).

# 0.8.0 (2019-09-24)

- **[Breaking change]** Update to `avm1-tree@0.8`.

## Typescript

- **[Fix]** Add initial support for parse errors.

# 0.7.1 (2019-07-17)

## Typescript

- **[Fix]** Add soft-block identifier to labels.

# 0.7.0 (2019-07-16)

- **[Breaking change]** Update to `avm1-tree@0.7`.

# 0.5.0 (2019-07-09)

- **[Breaking change]** Update to `avm1-tree@0.5`.
- **[Internal]** Switch to `travis-ci.com` for CI.

## Rust

- **[Internal]** Add `rustfmt` support.

# 0.4.1 (2019-05-21)

### Typescript

- **[Fix]** Skip overflows while parsing all raw actions.

# 0.4.0 (2019-05-21)

- **[Breaking change]** Update to `avm1-tree@0.4`.
- **[Internal]** Add `CHANGELOG.md`.
