# Contributing

## Development setup

```bash
git clone https://github.com/sfoerster/mntctl.git
cd mntctl
cargo build
```

## Running checks

All three must pass before merging:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## Adding a backend

1. Create `src/backend/<name>.rs` implementing the `Backend` trait
2. Add `pub mod <name>;` to `src/backend/mod.rs`
3. Register it in `BackendRegistry::new()`:
   ```rust
   registry.register(Box::new(<name>::<Name>Backend));
   ```
4. Add unit generation tests (string assertions against expected output)
5. Add a fixture TOML in `tests/fixtures/`
6. Add an example config in `docs/examples/`

## Testing

- **Unit tests**: config parsing, validation, systemd unit generation (pure string output)
- **CLI tests**: `assert_cmd` binary tests for help, list, status, completions
- **No mount mocking**: actual mount/unmount operations are tested manually, not in CI

## Code style

- `cargo fmt` for formatting
- `cargo clippy -- -D warnings` for lints
- Use `anyhow::Context` at call sites for error context
- Use `MntctlError` variants for domain-specific errors
- No `unwrap()` or `panic!()` in non-test code
