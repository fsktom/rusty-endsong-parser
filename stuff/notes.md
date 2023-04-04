# Notes

## Documentation

- [Rust By Example article](https://doc.rust-lang.org/rust-by-example/meta/doc.html)
- [The Rust Book: Making Useful Documentation Comments][book]
- [The rustdoc Book][rustdoc-book]
- [The Reference: Doc comments][ref-comments]
- [RFC 1574: API Documentation Conventions][api-conv]
- [RFC 1946: Relative links to other items from doc comments (intra-rustdoc links)][intra-links]
- [Is there any documentation style guide for comments? (reddit)][reddit]

[markdown]: https://en.wikipedia.org/wiki/Markdown
[book]: https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#making-useful-documentation-comments
[ref-comments]: https://doc.rust-lang.org/stable/reference/comments.html#doc-comments
[rustdoc-book]: https://doc.rust-lang.org/rustdoc/index.html
[api-conv]: https://rust-lang.github.io/rfcs/1574-more-api-documentation-conventions.html#appendix-a-full-conventions-text
[intra-links]: https://rust-lang.github.io/rfcs/1946-intra-rustdoc-links.html
[reddit]: https://www.reddit.com/r/rust/comments/ahb50s/is_there_any_documentation_style_guide_for/

### Running documentation locally

- [Article I used](https://dev.to/deciduously/prepare-your-rust-api-docs-for-github-pages-2n5i)
- [HTTP server used](https://developer.mozilla.org/en-US/docs/Learn/Common_questions/set_up_a_local_testing_server)

Because just navigating through the raw `.html` files using the `file://` protocol
doesn't keep the zoom level and I'm visually impaired and need to zoom in at least 150%...

1. Create `index.html` with `<meta http-equiv="refresh" content="0; url=rusty_endsong_parser">` in `target/doc`
2. Run `python3 -m http.server` in `target/doc`
3. Open `http://0.0.0.0:8000/` in browser and enjoy having a constant zoom level!

### Lints

[Useful article](https://medium.com/@Razican/enforcing-documentation-in-a-medium-size-rust-project-7b6a2a47b6d6)

[Rustdoc lints](https://doc.rust-lang.org/rustdoc/lints.html)
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
[use also `cargo criterion`](https://crates.io/crates/cargo-criterion)

### Performance
[very good interview](https://www.youtube.com/watch?v=OtozASk68Os) - a lot applies to this program/my style of programming as well (e.g. the whole memory allocations I removed in [8b0fcdc](https://github.com/fsktom/rusty-endsong-parser/commit/8b0fcdc2d9f0cfbf9faae18de88b47e1a427326c)
