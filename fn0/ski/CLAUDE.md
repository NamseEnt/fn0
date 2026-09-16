This project is a WinterCG-compatible minimum JS runtime that completely excludes Node.js-like features.
module, resolve, import, etc. are not supported. Instead, it assumes fully bundled files via rolldown as input.

## Important Rules
- Always use `cargo add` to install crates with the latest version
- Never manually edit Cargo.toml for crate installation
- Always use `cargo clippy` for build checks (instead of cargo build)
- Do not add Clippy exception attributes or equivalent lint suppressions. Resolve Clippy findings through code changes.
- Final testing should be verified by running `cargo run`
