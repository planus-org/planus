* Update version numbers in:
  * `planus-cli/Cargo.toml`
  * `planus/Cargo.toml`
  * `planus/src/lib.rs`
  * `planus-cli/src/codegen/templates/rust/namespace.template`
  * `examples/rust/Cargo.toml`
* Update CHANGELOG.md, including links at the bottom
* Commit changes to a branch and make a PR
* Run `cargo publish --dry-run` on PR branch
* Wait for CI and merge PR to main branch
* Run `cargo publish` on main branch
* Push tag and make a github release
* Make an announcement on discord
