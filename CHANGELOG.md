# lignin Changelog

<!-- markdownlint-disable no-trailing-punctuation -->

## 0.0.5

2021-01-30

* **Breaking:**
  * Upgraded `bumpalo` dependency from ~3.4.0 to ~3.6.0
    > to use fallible allocation downstream in Asteracea.
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
