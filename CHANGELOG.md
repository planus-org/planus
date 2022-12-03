# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- \[Rust\]: Added more `impl`s for union and struct references
- \[Rust\]: Add an `ENUM_VALUES` const to enums
- \[Rust\]: Make `Vector` more similar to rust slices by adding more methods
- \[Rust\]: Vectors of `uint8`/`int8` now deserialize directly to rust slices
- \[Rust\]: Implement caching of vtables, byte-slices and strings, hidden
  behind the `vtable-cache`, `bytes-cache` and `string-cache` feature flags
  (they are enabled by default)
- \[Rust\]: Bump the Minimum Support Rust Version (MSRV) to 1.61.0.
- \[Rust\]: Add license files to crates
- \[Rust\]: Implement a builder pattern for serializing tables and unions
- Add support for docstrings, and add them to the Rust output.
- Update the `README` with information about our Discord server.

### Removed
- \[Rust\]: The old ways of serializing tables and unions using `create`-functions have been removed.

## [0.3.1] - 2022-06-15
### Added
- \[Rust\]: Made planus crate `#[deny(missing_docs)]`
- \[Rust\]: Add support for strings in unions
- \[Rust\]: Add support for structs in unions

### Fixed
- \[Rust\]: Fix a few codegen bugs when using certain field names
- \[Rust\]: Fix panic when accessing union from invalid input
- \[Rust\]: Fix a bug where struct attributes were used instead of struct field attributes
- \[Rust\]: Implement support for the `force_align` attribute on structs

## [0.3.0] - 2022-02-06
### Added
- Improved documentation
- \[Rust\]: The `Builder` now has impls for `Send` and and `Sync`
- \[Rust\]: We now derive `Ord`, `Eq`, `Hash` and `Default` in some cases
- \[Rust\]: Added an MSRV policy
- Added a type-check for name overlap between namespaces and declarations
- Added `generate-completions` subcommand to `planus-cli` to generate shell completions for popular shells
- Added a DOT backend
- Added a version check to guard against using out-of-date code

### Fixed
- Fixed some typos by [@OliverEvans96](https://github.com/OliverEvans96)
- Various clippy lints


## [0.2.0] - 2022-01-12
### Rust
- Add null terminators to strings
- Make generated code work in `no_std`
- Add support for the `id` attribute
- (Breaking) Removed the `ToOwned` trait in favor of `TryInto`
- (Breaking) The Vector deserialization API changed slightly, see #59 for details


## [0.1.0] - 2021-12-30
- Initial release

[Unreleased]: https://github.com/planus-org/planus/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/planus-org/planus/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/planus-org/planus/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/planus-org/planus/releases/tag/v0.1.0
