# simple-crate conventions

- Always add doc comments (`///`) to public items.
- Use `thiserror` for error types.
- Inline tests (in the same file) are preferred over a separate tests/ directory.
- Run `cargo fmt` and `cargo clippy` before committing.
- Keep modules small — if a file exceeds 300 lines, split it.
