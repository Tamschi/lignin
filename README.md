# lignin

[![Lib.rs](https://img.shields.io/badge/Lib.rs-*-84f)](https://lib.rs/crates/lignin)
[![Crates.io](https://img.shields.io/crates/v/lignin)](https://crates.io/crates/lignin)
[![Docs.rs](https://docs.rs/lignin/badge.svg)](https://docs.rs/lignin)

![Rust 1.54](https://img.shields.io/static/v1?logo=Rust&label=&message=1.54&color=grey)
[![CI](https://github.com/Tamschi/lignin/workflows/CI/badge.svg?branch=develop)](https://github.com/Tamschi/lignin/actions?query=workflow%3ACI+branch%3Adevelop)
![Crates.io - License](https://img.shields.io/crates/l/lignin/0.1.0)

[![GitHub](https://img.shields.io/static/v1?logo=GitHub&label=&message=%20&color=grey)](https://github.com/Tamschi/lignin)
[![open issues](https://img.shields.io/github/issues-raw/Tamschi/lignin)](https://github.com/Tamschi/lignin/issues)
[![open pull requests](https://img.shields.io/github/issues-pr-raw/Tamschi/lignin)](https://github.com/Tamschi/lignin/pulls)
[![good first issues](https://img.shields.io/github/issues-raw/Tamschi/lignin/good%20first%20issue?label=good+first+issues)](https://github.com/Tamschi/lignin/contribute)

[![crev reviews](https://web.crev.dev/rust-reviews/badge/crev_count/lignin.svg)](https://web.crev.dev/rust-reviews/crate/lignin/)
[![Zulip Chat](https://img.shields.io/endpoint?label=chat&url=https%3A%2F%2Fiteration-square-automation.schichler.dev%2F.netlify%2Ffunctions%2Fstream_subscribers_shield%3Fstream%3Dproject%252Flignin)](https://iteration-square.schichler.dev/#narrow/stream/project.2Flignin)

A lightweight but featureful virtual DOM library, primarily for web use.

`no_std` and no dependencies without the `"callbacks"` feature.

## Installation

Please use [cargo-edit](https://crates.io/crates/cargo-edit) to always add the latest version of this library:

```cmd
cargo add lignin && cargo add -D lignin --features callbacks
```

Some type constraints are more strict with the `"callbacks"` feature enabled, so make sure to always check this way!

**When writing a renderer that supports callbacks**, instead use

```cmd
cargo add lignin --features callbacks
```

to always enable the feature.

## Features

### `"callbacks"`

Enables DOM callback support. Off by default. Requires `std`.

Apps or components can be written against the callback API without enabling this feature, in which case those code paths can be erased at compile-time.

## Example

```rust
use lignin::{Node, Element, ElementCreationOptions};

// Please bring your own allocator where necessary.
let _ = &Node::HtmlElement {
  element: &Element {
    name: "DIV", // Use all-caps for more efficient DOM interactions.¹
    creation_options: ElementCreationOptions::new(), // `const fn` builder and getter/setter patterns for extensible interfaces.
    attributes: &[],
    content: Node::Multi(&[
      "Hello! ".into(), // Some convenience included.
      Node::Comment {
        comment: "--> Be mindful of HTML pitfalls. <!--", // Renderers must validate.
        dom_binding: None,
      }
    ]),
    event_bindings: &[], // Strongly typed using `web-sys`.
  },
  dom_binding: None, // For JS interop.
}
.prefer_thread_safe(); // Thread-safety can be inferred from bindings!
```

¹ See [Element.tagName (MDN)](https://developer.mozilla.org/en-US/docs/Web/API/Element/tagName). This avoids case-insensitive comparisons.

## Implementation Contract

There are a few ecosystem compatibility rules that aren't covered by Rust's type system or the `unsafe` keyword. Please see [the main module documentation](https://docs.rs/lignin/0.1.0/lignin/#implementation-contract) for more information.

## Prior Art

- Dodrio, which didn't quite satisfy my requirements but generally inspired this approach.

## Thanks

To [Dronaroid](https://twitter.com/artdron) for finding a great name for this library, [@platy](https://github.com/platy) for criticism and pushing me towards a better implementation, and everyone on the Rust Programming Language Community Server discord who answered my questions that came up during the rewrite.

## License

Licensed under either of

- Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING](CONTRIBUTING.md) for more information.

## [Code of Conduct](CODE_OF_CONDUCT.md)

## [Changelog](CHANGELOG.md)

## Versioning

`lignin` strictly follows [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html) with the following exceptions:

- The minor version will not reset to 0 on major version changes (except for v1).  
Consider it the global feature level.
- The patch version will not reset to 0 on major or minor version changes (except for v0.1 and v1).  
Consider it the global patch level.

This includes the Rust version requirement specified above.  
Earlier Rust versions may be compatible, but this can change with minor or patch releases.

Which versions are affected by features and patches can be determined from the respective headings in [CHANGELOG.md](CHANGELOG.md).

Note that dependencies of this crate may have a more lenient MSRV policy!
Please use `cargo +nightly update -Z minimal-versions` in your automation if you don't generate Cargo.lock manually (or as necessary) and require support for a compiler older than current stable.
