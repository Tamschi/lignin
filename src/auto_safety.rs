//! Transitive (across function boundaries) [`ThreadSafety`] inference, mainly for use by frameworks.
//!
//! All methods in this module are always-inlined no-ops, meaning that there is zero runtime cost to them.
#![allow(clippy::inline_always)]
//!
//! > This feature relies on opaque return types (`-> impl Trait`) leaking [`Send`] and [`Sync`], so the theoretical limit here, even after specialization lands, are four distinct 'real' types with restrictions on conversion incompatibilities.
//! > Fortunately, `lignin` only needs two of these slots with straightforward compatibility, the `!Send + !Sync` and the `Send + Sync` one.
//! >
//! > Please refer to the item documentation for implementation details.
//!
//! # Examples / Usage
//!
//! > All examples share the following definitions:
//! >
//! > ```rust
//! > use lignin::{
//! >   auto_safety::{AutoSafe, Deanonymize as _}, // <-- Important!
//! >   Node, ThreadBound, ThreadSafe,
//! > };
//! >
//! > fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! > fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! > fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! > fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! >
//! > fn allocate<'a, T>(value: T) -> &'a T { unreachable!() }
//! > #
//! > # fn assert_safe<'a>(value: Node<'a, ThreadSafe>) { }
//! > # fn assert_bound<'a>(value: Node<'a, ThreadBound>) { }
//! > ```
//! >
//! > I recommend using [`bumpalo`](https://github.com/fitzgen/bumpalo) as VDOM allocator since it is fast and versatile, but `lignin` itself has no preference in this regard.
//!
//! > In all examples and the above, [`Node`] can be replaced by any other `ThreadBindable` type.
//!
//! ## Basic Forwarding
//!
//! To mark the thread-safety of a function as inferred, return [`AutoSafe`] wrapping the [`ThreadBound`] version of the VDOM node you want to return.
//!
//! This works with manually-defined sources…:
//!
//! ```rust
//! # use lignin::{
//! #   auto_safety::{AutoSafe, Deanonymize as _},
//! #   Node, ThreadBound, ThreadSafe,
//! # };
//! #
//! # fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! # fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! # fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! # fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! #
//! # fn allocate<'a, T>(value: T) -> &'a T { unreachable!() }
//! #
//! # fn assert_safe<'a>(value: Node<'a, ThreadSafe>) { }
//! # fn assert_bound<'a>(value: Node<'a, ThreadBound>) { }
//! #
//! fn safe_1<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! fn bound_1<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! #
//! # fn assert() {
//! #   assert_safe(safe_1().deanonymize());
//! #   assert_bound(bound_1().deanonymize());
//! # }
//! ```
//!
//! …as well as ones where the original return type is inferred (opaque):
//!
//! ```rust
//! # use lignin::{
//! #   auto_safety::{AutoSafe, Deanonymize as _},
//! #   Node, ThreadBound, ThreadSafe,
//! # };
//! #
//! # fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! # fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! # fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! # fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! #
//! # fn allocate<'a, T>(value: T) -> &'a T { unreachable!() }
//! #
//! # fn assert_safe<'a>(value: Node<'a, ThreadSafe>) { }
//! # fn assert_bound<'a>(value: Node<'a, ThreadBound>) { }
//! #
//! fn safe_2<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { inferred_safe() }
//! fn bound_2<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { inferred_bound() }
//! #
//! # fn assert() {
//! #   assert_safe(safe_2().deanonymize());
//! #   assert_bound(bound_2().deanonymize());
//! # }
//! ```
//!
//! ## Deanonymization
//!
//! Rust doesn't allow consumption of the inferred concrete return type of a function directly, so while the following works fine…:
//!
//! ```rust
//! # use lignin::{
//! #   auto_safety::{AutoSafe, Deanonymize as _},
//! #   Node, ThreadBound, ThreadSafe,
//! # };
//! #
//! # fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! # fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! # fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! # fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! #
//! # fn allocate<'a, T>(value: T) -> &'a T { unreachable!() }
//! #
//! # fn assert_safe<'a>(value: Node<'a, ThreadSafe>) { }
//! # fn assert_bound<'a>(value: Node<'a, ThreadBound>) { }
//! #
//! fn safe_1<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> {
//!   Node::Ref(allocate(safe()))
//! }
//!
//! fn bound_1<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> {
//!   Node::Ref(allocate(bound()))
//! }
//! #
//! # fn assert() {
//! #   assert_safe(safe_1().deanonymize());
//! #   assert_bound(bound_1().deanonymize());
//! # }
//! ```
//!
//! each of these fails to compile:
//!
//! ```compile_fail
//! # use lignin::{
//! #   auto_safety::{AutoSafe, Deanonymize as _},
//! #   Node, ThreadBound, ThreadSafe,
//! # };
//! #
//! # fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! # fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! # fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! # fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! #
//! # fn allocate<'a, T>(value: T) -> &'a T { unreachable!() }
//! #
//! # fn assert_safe<'a>(value: Node<'a, ThreadSafe>) { }
//! # fn assert_bound<'a>(value: Node<'a, ThreadBound>) { }
//! #
//! fn safe_2<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> {
//!   Node::Ref(allocate(inferred_safe()))
//!   //                 ^^^^^^^^^^^^^^^ expected enum `Node`, found opaque type
//! }
//! ```
//!
//! ```compile_fail
//! # use lignin::{
//! #   auto_safety::{AutoSafe, Deanonymize as _},
//! #   Node, ThreadBound, ThreadSafe,
//! # };
//! #
//! # fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! # fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! # fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! # fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! #
//! # fn allocate<'a, T>(value: T) -> &'a T { unreachable!() }
//! #
//! # fn assert_safe<'a>(value: Node<'a, ThreadSafe>) { }
//! # fn assert_bound<'a>(value: Node<'a, ThreadBound>) { }
//! #
//! fn bound_2<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> {
//!   Node::Ref(allocate(inferred_bound()))
//!   //                 ^^^^^^^^^^^^^^^^ expected enum `Node`, found opaque type
//! }
//! ```
//!
//! ### `.deanonymize()`
//!
//! Call `.deanonymize()` _without qualification_ on an opaquely-typed value to cast it to the underlying named type.
//!
//! This method resolves either through [`AutoSafe`] or [`Deanonymize`], so it's important for both traits to be in scope at the call site!
//!
//! ```
//! # use lignin::{
//! #   auto_safety::{AutoSafe, Deanonymize as _},
//! #   Node, ThreadBound, ThreadSafe,
//! # };
//! #
//! # fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! # fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! # fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! # fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! #
//! # fn allocate<'a, T>(value: T) -> &'a T { unreachable!() }
//! #
//! # fn assert_safe<'a>(value: Node<'a, ThreadSafe>) { }
//! # fn assert_bound<'a>(value: Node<'a, ThreadBound>) { }
//! #
//! fn safe_2<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> {
//!   Node::Ref(allocate(inferred_safe().deanonymize()))
//! }
//!
//! fn bound_2<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> {
//!   Node::Ref(allocate(inferred_bound().deanonymize()))
//! }
//! #
//! # fn assert() {
//! #   assert_safe(safe_2().deanonymize());
//! #   assert_bound(bound_2().deanonymize());
//! # }
//! ```
//!
//! > You also have to do this to annotate the type of local variables…:
//! >
//! > ```
//! > # use lignin::{
//! > #   auto_safety::{AutoSafe, Deanonymize as _},
//! > #   Node, ThreadBound, ThreadSafe,
//! > # };
//! > #
//! > # fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! > # fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! > # fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! > # fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! > #
//! > # fn allocate<'a, T>(value: T) -> &'a T { unreachable!() }
//! > #
//! > # fn assert_safe<'a>(value: Node<'a, ThreadSafe>) { }
//! > # fn assert_bound<'a>(value: Node<'a, ThreadBound>) { }
//! > #
//! > let safe_node: Node<_> = inferred_safe().deanonymize();
//! > let bound_node: Node<_> = inferred_bound().deanonymize();
//! > ```
//! >
//! > …or to specify a [`ThreadSafety`] in the return type:
//! >
//! > ```
//! > # use lignin::{
//! > #   auto_safety::{AutoSafe, Deanonymize as _},
//! > #   Node, ThreadBound, ThreadSafe,
//! > # };
//! > #
//! > # fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! > # fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! > # fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! > # fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! > #
//! > # fn allocate<'a, T>(value: T) -> &'a T { unreachable!() }
//! > #
//! > # fn assert_safe<'a>(value: Node<'a, ThreadSafe>) { }
//! > # fn assert_bound<'a>(value: Node<'a, ThreadBound>) { }
//! > #
//! > fn strictly_safe<'a>() -> Node::<'a, ThreadSafe> {
//! >   inferred_safe().deanonymize()
//! > }
//! >
//! > fn strictly_bound<'a>() -> Node::<'a, ThreadBound> {
//! >   inferred_bound().deanonymize()
//! > }
//! > ```
//!
//! #### Identity Cast
//!
//! Calling `.deanonymize()` on named types is valid but ultimately useless:
//!
//! ```
//! # use lignin::{
//! #   auto_safety::{AutoSafe, Deanonymize as _},
//! #   Node, ThreadBound, ThreadSafe,
//! # };
//! #
//! # fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! # fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! # fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! # fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! #
//! # fn allocate<'a, T>(value: T) -> &'a T { unreachable!() }
//! #
//! # fn assert_safe<'a>(value: Node<'a, ThreadSafe>) { }
//! # fn assert_bound<'a>(value: Node<'a, ThreadBound>) { }
//! #
//! // warning:
//! //   use of deprecated associated function `lignin::auto_safety::<impl lignin::Node<'a, S>>::deanonymize`:
//! //   Call of `.deanonymize()` on named type.
//! let safe_node: Node<ThreadSafe> = safe().deanonymize();
//! //                                       ^^^^^^^^^^^
//! let bound_node: Node<ThreadBound> = bound().deanonymize();
//! //                                          ^^^^^^^^^^^
//! ```
//!
//! Macros can suppress this warning by emitting the method call with [`Span::mixed_site()`](https://doc.rust-lang.org/stable/proc_macro/struct.Span.html#method.mixed_site) hygiene.
//!
//! <!-- TODO: Make sure that's actually the case. -->
//!
//! #### No Coercion
//!
//! Calls to `.deanonymize()` can't be coerced, so each of the following fails to compile:
//!
//! ```compile_fail
//! # use lignin::{
//! #   auto_safety::{AutoSafe, Deanonymize as _},
//! #   Node, ThreadBound, ThreadSafe,
//! # };
//! #
//! # fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! # fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! # fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! # fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! #
//! # fn allocate<'a, T>(value: T) -> &'a T { unreachable!() }
//! #
//! # fn assert_safe<'a>(value: Node<'a, ThreadSafe>) { }
//! # fn assert_bound<'a>(value: Node<'a, ThreadBound>) { }
//! #
//! fn attempt_safe<'a>() -> Node::<'a, ThreadSafe> {
//! //                       ----------------------
//! //                       expected `Node<'a, ThreadSafe>` because of return type
//!   inferred_bound().deanonymize()
//! //^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected struct `ThreadSafe`, found struct `ThreadBound`
//! //
//! // note: expected enum `Node<'a, ThreadSafe>`
//! //          found enum `Node<'_, ThreadBound>`
//! }
//! ```
//!
//! ```compile_fail
//! # use lignin::{
//! #   auto_safety::{AutoSafe, Deanonymize as _},
//! #   Node, ThreadBound, ThreadSafe,
//! # };
//! #
//! # fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! # fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! # fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! # fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! #
//! # fn allocate<'a, T>(value: T) -> &'a T { unreachable!() }
//! #
//! # fn assert_safe<'a>(value: Node<'a, ThreadSafe>) { }
//! # fn assert_bound<'a>(value: Node<'a, ThreadBound>) { }
//! #
//! fn attempt_bound<'a>() -> Node::<'a, ThreadBound> {
//! //                        -----------------------
//! //                        expected `Node<'a, ThreadSafe>` because of return type
//!   inferred_safe().deanonymize()
//! //^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected struct `ThreadBound`, found struct `ThreadSafe`
//! //
//! // note: expected enum `Node<'a, ThreadBound>`
//! //          found enum `Node<'_, ThreadSafe>`
//! }
//! ```
//!
//! # Alignment
//!
//! TODO
//!
//! # [`ThreadSafe`] Preference
//!
//! TODO

use crate::{Node, ThreadBound, ThreadSafe, ThreadSafety};

mod sealed {
	/// It's probably good to be a bit more specific in [`Align`](`super::Align`)'s signature, among others.
	/// The bounds are necessary the default implementations in derived traits and also prevent their object-safety, which is good because that would at best only add useless dynamic dispatch overhead.
	pub trait ThreadBindable: Copy + Sized {}
}

impl<'a, S: ThreadSafety> Node<'a, S> {
	#[deprecated = "Call of `.deanonymize()` on named type."]
	#[must_use]
	pub fn deanonymize(self) -> Self {
		self
	}
}

/// Deanonymize towards the general ([`ThreadBound`]) case. Used as e.g. `-> impl<AutoSafe<…>>`.
///
/// See module documentation for usage.
pub trait AutoSafe<ThreadBound>
where
	Self: sealed::ThreadBindable,
	ThreadBound: sealed::ThreadBindable,
{
	/// Deanonymize towards a compatible concrete type.
	///
	/// This method is by reference, so it will resolve with lower priority than the by-value method on [`Deanonymize`].  
	/// Note that not all tooling will show the correct overload here, but the compiler knows which to pick.
	#[must_use]
	#[inline(always)] // No-op.
	fn deanonymize(&self) -> ThreadBound {
		unsafe {
			// SAFETY:
			// Under normal circumstances, this trait or method would have to be `unsafe`.
			// However, we're ensuring only sound implementations exist by sealing it and carefully implementing it only across layout-compatible types.
			*(self as *const Self).cast()
		}
	}
}

/// Deanonymize towards the special ([`ThreadSafe`]) case. **This trait must be in scope for correct inference!**
///
/// See module documentation for usage.
pub trait Deanonymize<'a>: sealed::ThreadBindable + Send + Sync {
	type ThreadSafe: sealed::ThreadBindable;
	/// Deanonymize towards a compatible concrete type.
	///
	/// This method is by value, so it will resolve with higher priority than the by-reference method on [`AutoSafe`].  
	/// Note that not all tooling will show the correct overload here, but the compiler knows which to pick.
	#[must_use]
	#[inline(always)] // No-op.
	fn deanonymize(self) -> Self::ThreadSafe {
		unsafe {
			// SAFETY:
			// Under normal circumstances, this trait or method would have to be `unsafe`.
			// However, we're ensuring only sound implementations exist by sealing it and carefully implementing it only across layout-compatible types.
			*(&self as *const Self).cast()
		}
	}
}

impl<'a, S: ThreadSafety> sealed::ThreadBindable for Node<'a, S> {}

impl<'a, S: ThreadSafety> AutoSafe<Node<'a, ThreadBound>> for Node<'a, S> {}
impl<'a, T: Send + Sync + AutoSafe<Node<'a, ThreadBound>>> Deanonymize<'a> for T {
	type ThreadSafe = Node<'a, ThreadSafe>;
}

impl<'a> Node<'a, ThreadSafe> {
	/// Gently nudges the compiler to choose the thread-safe version of a value if both are possible.
	///
	/// This method is by value, so it will resolve with higher priority than the by-reference method on the thread-bound type.  
	/// Note that not all tooling will show the correct overload here, but the compiler knows which to pick.
	#[must_use]
	#[inline(always)] // No-op.
	pub fn prefer_thread_safe(self) -> Self {
		self
	}
}
impl<'a> Node<'a, ThreadBound> {
	/// Gently nudges the compiler to choose the thread-safe version of a value if both are is possible.
	///
	/// This method is by reference, so it will resolve with lower priority than the by-reference method on the thread-safe type.  
	/// Note that not all tooling will show the correct overload here, but the compiler knows which to pick.
	#[must_use]
	#[inline(always)] // No-op.
	pub fn prefer_thread_safe(&self) -> Self {
		*self
	}
}
impl<'a> From<Node<'a, ThreadSafe>> for Node<'a, ThreadBound> {
	#[allow(clippy::inline_always)]
	#[inline(always)] // No-op.
	fn from(thread_safe: Node<'a, ThreadSafe>) -> Self {
		unsafe { *(&thread_safe as *const Node<'a, ThreadSafe>).cast() }
	}
}

/// Contextually thread-binds an instance, or not. Use only without qualification.
///
/// This trait acts as (i.e.: _is_) [`Into`] on and between thread-bindable types, but without raising `useless_conversion` warnings.
///
/// See module documentation for when to use this trait and when it's unnecessary.
pub trait Align<T: sealed::ThreadBindable>: sealed::ThreadBindable
where
	Self: Into<T>,
{
	/// Contextually thread-binds an instance, or not. Use only without qualification.
	#[allow(clippy::inline_always)]
	#[inline(always)] // No-op.
	fn align(self) -> T {
		self.into()
	}
}
impl<T: Into<U>, U> Align<U> for T
where
	T: sealed::ThreadBindable,
	U: sealed::ThreadBindable,
{
}
