# Contributing

Keep this a pure calculation library. Supply time, coordinates and model inputs
explicitly. Add numerical fixtures with their public source, units, time scale
and tolerance; use synthetic inputs, never customer birth details.

Read [the export boundary](docs/export-boundary.md) before adding a file or
dependency. Changes to `export-manifest.json` need the same review as code.
No automatic synchronization from private repositories is permitted.

```sh
python scripts/check_public_boundary.py
python -m unittest discover -s scripts -p "test_*.py"
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo check -p astro-engine --target wasm32-unknown-unknown --locked
cargo doc --workspace --no-deps --locked
```

Rust is pinned by rust-toolchain.toml; use Python 3.11 or later for the boundary
check. Contributions to original code are submitted under Apache-2.0. Retain
inherited notices, and identify the license/provenance of any adapted code.

Submit only work you are authorised to license, including any employer-owned
contributions. Describe the public numerical reference and the changes made
when adapting an implementation. See [licensing](docs/licensing.md).

Before release, fetch the repository's branches and tags and run
`python scripts/check_public_boundary.py --history` in a full clone. A current
tree check alone cannot find sensitive content deleted in an earlier commit.
Review pull-request discussions and other GitHub surfaces separately before
making a previously private repository public.
