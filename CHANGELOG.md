# lignin Changelog

<!-- markdownlint-disable no-trailing-punctuation -->

## next

TODO: Date

* Features
  * Added `callback_registry::ENABLED` constant.
  * Added `callback_registry::if_callbacks!` and `callback_registry::if_not_callbacks!` macros.
    > Prefer using the constant over the macros where possible, as the former won't turn off syntax and type checks for part of your code.
* Revisions
  * Updated Rust project template to version 6
    > This mainly leads to more thorough CI, but also fixes the docs.rs badge link.
  * Added more usage notes and examples to the documentation.

## 0.0.6

2021-04-18

This is nearly a complete rewrite of this crate, with much improved DOM feature coverage, and support for safe threading and callbacks.
All VDOM types can now be [forgotten] without risk of memory leaks.

[forgotten]: https://doc.rust-lang.org/stable/core/mem/fn.forget.html

* **Breaking:**
  * Removed `bumpalo` dependency and re-export
    > Active Remnants will likely be managed by the DOM renderer, once this feature is available.
  * Stubbed out remnant implementation
    > See above.
  * Removed "debug" feature
    > `Debug` is now always implemented, without requiring a proc macro dependency.
  * Added `"callbacks"` feature
    > `lignin` now guarantees that no memory leaks happen if a node graph is forgotten, but the tradeoff for this is that callbacks (e.g. to element reference setters or event handlers) go through a global registry with incrementing keys.
    >
    > This key space (for now `NonZeroU32`) can be exhausted after a long time of heavy use, at which point any new registration will panic. Not requiring the `"callbacks"` feature instead voids out this registry, so that
    >
    > 1. This crate becomes no_std and does not have usage limits and
    >
    > 2. all callback invocations silently do nothing.
    >
    > The feature should only be enabled by renderers that support these callbacks.  
    > Any other consumers of this library should test with the feature, but not require it.
  * Event bindings are now both leak-resistant and sound
  * VDOM producers can now subscribe to DOM Node reference updates
  * Increased minimum Rust version from 1.44 to 1.46
    > in order to use `match` expressions in `const fn` functions.

* Revisions
  * Run CI against Rust 1.46 instead of Rust 1.44.0 specifically.

## 0.0.5

2021-01-30

* **Breaking:**
  * Upgraded `bumpalo` dependency from ~3.4.0 to ~3.6.0
    > to use fallible allocation/bump object initialisation downstream in Asteracea.
  * Increased minimum Rust version from 1.40.0 to 1.44.0
    > to upgrade bumpalo.

## 0.0.4

2021-01-29

* **Breaking:**
  * Fixed `bumpalo` at `"~3.4.0"` due to minor version Rust version requirement bumps.

## 0.0.3

2021-01-01

* **Breaking:**
  * `EventBinding.handler` is a `Pin<Rc<dyn Fn(&dyn Any) + 'a>>` now.

## 0.0.2

2020-11-20

* **Breaking:**
  * Removed "bumpalo" and "remnants" features (always enabled now)
* Revisions:
  * Fixed Travis configuration

## 0.0.1

2020-10-02

Initial unstable release
