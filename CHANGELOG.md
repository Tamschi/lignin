# lignin Changelog

<!-- markdownlint-disable no-trailing-punctuation -->

## next

TODO: Date

* **Breaking:**
  * Removed `bumpalo` dependency and re-export
    > the necessary state management for Remnants will happen out of band, somehow.  
    > Probably with mutable tree inside `lignin-dom` that's iterated in parallel to diffs.
  * Stubbed out remnant implementation
    > See above.
  * Removed "debug" feature
    > `Debug` is now always implemented, without requiring a proc macro dependency.
  * Added `"callbacks"` feature (default)
    > `lignin` now guarantees that no memory leaks happen if a node graph is forgotten, but the tradeoff for this is that callbacks (e.g. to element reference setters or event handlers) go through a global registry with incrementing keys.
    >
    > This key space (for now `u64`) can be exhausted after a long time of heavy use, at which point any new registration will panic. Disabling the `"callbacks"` feature instead voids out this registry, so that
    >
    > 1. This crate becomes no_std and does not have usage limits and
    >
    > 2. all callback invocations silently do nothing.
    >
  * Event bindings are now both leak-safe and sound
  * VDOM producers can now subscribe to DOM reference updates

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
