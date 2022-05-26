# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
- \[Rust\]: Add support for strings in unions
- \[Rust\]: Add support for structs in unions

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
