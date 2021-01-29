# lignin Changelog

<!-- markdownlint-disable no-trailing-punctuation -->

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
