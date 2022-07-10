# Notes

## Documentation

### Running documentation locally

- [Article I used](https://dev.to/deciduously/prepare-your-rust-api-docs-for-github-pages-2n5i)
- [HTTP server used](https://developer.mozilla.org/en-US/docs/Learn/Common_questions/set_up_a_local_testing_server)

Because just navigating through the raw `.html` files using the `file://` protocol
doesn't keep the zoom level and I'm visually impaired and need to zoom in at least 150%...

1. Create `index.html` with `<meta http-equiv="refresh" content="0; url=rusty_endsong_parser">` in `target/doc`
2. Run `python3 -m http.server` in `target/doc`
3. Open `http://0.0.0.0:8000/` in browser and enjoy having a constant zoom level!

### Lints

```rust
#![forbid(missing_docs)]
#![forbid(clippy::missing_docs_in_private_items)]
#![forbid(rustdoc::broken_intra_doc_links)]
#![forbid(rustdoc::private_intra_doc_links)]
#![forbid(rustdoc::missing_crate_level_docs)]
#![forbid(rustdoc::invalid_codeblock_attributes)]
#![forbid(rustdoc::invalid_rust_codeblocks)]
#![forbid(rustdoc::bare_urls)]
```

in `main.rs`

and in VSC:
[`"rust-analyzer.checkOnSave.command": "clippy"`](https://users.rust-lang.org/t/how-to-use-clippy-in-vs-code-with-rust-analyzer/41881/2)

## idk

### JSON parsing

[https://youtu.be/hIi_UlyIPMg](https://youtu.be/hIi_UlyIPMg)

### Benchmarking

[With Criterion crate](https://youtu.be/eIB3Pd5LBkc)
