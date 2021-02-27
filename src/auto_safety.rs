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
//! **The following is a long explanation that you probably don't have to read.**  
//! In hand-written code, you can always use [`From`] or [`Into`] to cast a [`…<ThreadSafe>`](`ThreadSafe`) type to the matching [`…<ThreadBound>`](`ThreadBound`) type where necessary.  
//! If you receive an opaque type, `use lignin::auto_safety::{AutoSafe as _, Deanonymize as _};` and call `.deanonymize()` on it, then **politely** ask the author to consider being more specific.
//!
//! If you do intend to use this module, please still declare [`ThreadSafe`] explicitly at crate boundaries, or encourage developers using your library to do so.
//! [You can find more information on this near the end of this page.](#limiting-autosafe-exposure)
//!
//! # Examples / Usage
//!
//! > All examples share the following definitions:
//! >
//! > ```rust
//! > use lignin::{
//! >   auto_safety::{Align as _, AutoSafe, Deanonymize as _}, // <-- Important!
//! >   Node, ThreadBound, ThreadSafe,
//! > };
//! >
//! > fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! > fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! > fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! > fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! >
//! > fn allocate<'a, T>(value: T) -> &'a T {
//! >   // …
//! > #   Box::leak(Box::new(value))
//! > }
//! > #
//! > # fn assert_safe<'a>(value: Node<'a, ThreadSafe>) { }
//! > # fn assert_bound<'a>(value: Node<'a, ThreadBound>) { }
//! > ```
//! >
//! > I recommend using [`bumpalo`](https://github.com/fitzgen/bumpalo) as VDOM allocator since it is fast and versatile, but `lignin` itself has no preference in this regard.
//!
//! > In all examples and the above, [`Node`] can be replaced by any other [`Vdom`] type.
//!
//! ## Basic Forwarding
//!
//! To mark the [`ThreadSafety`] of a function as inferred, return [`AutoSafe`] wrapping the [`ThreadBound`] version of the VDOM node you want to return.
//!
//! This works with manually-defined sources…:
//!
//! ```rust
//! # use lignin::{
//! #   auto_safety::{Align as _, AutoSafe, Deanonymize as _},
//! #   Node, ThreadBound, ThreadSafe,
//! # };
//! #
//! # fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! # fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! # fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! # fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! #
//! # fn allocate<'a, T>(value: T) -> &'a T { Box::leak(Box::new(value)) }
//! #
//! # fn assert_safe<'a>(value: Node<'a, ThreadSafe>) { }
//! # fn assert_bound<'a>(value: Node<'a, ThreadBound>) { }
//! #
//! fn safe_1<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! fn bound_1<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! #
//! # assert_safe(safe_1().deanonymize());
//! # assert_bound(bound_1().deanonymize());
//! ```
//!
//! …as well as ones where the original return type is inferred (opaque):
//!
//! ```rust
//! # use lignin::{
//! #   auto_safety::{Align as _, AutoSafe, Deanonymize as _},
//! #   Node, ThreadBound, ThreadSafe,
//! # };
//! #
//! # fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! # fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! # fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! # fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! #
//! # fn allocate<'a, T>(value: T) -> &'a T { Box::leak(Box::new(value)) }
//! #
//! # fn assert_safe<'a>(value: Node<'a, ThreadSafe>) { }
//! # fn assert_bound<'a>(value: Node<'a, ThreadBound>) { }
//! #
//! fn safe_2<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { inferred_safe() }
//! fn bound_2<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { inferred_bound() }
//! #
//! # assert_safe(safe_2().deanonymize());
//! # assert_bound(bound_2().deanonymize());
//! ```
//!
//! ## Deanonymization
//!
//! Rust doesn't allow consumption of the inferred concrete return type of a function directly, so while the following works fine…:
//!
//! ```rust
//! # use lignin::{
//! #   auto_safety::{Align as _, AutoSafe, Deanonymize as _},
//! #   Node, ThreadBound, ThreadSafe,
//! # };
//! #
//! # fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! # fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! # fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! # fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! #
//! # fn allocate<'a, T>(value: T) -> &'a T { Box::leak(Box::new(value)) }
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
//! # assert_safe(safe_1().deanonymize());
//! # assert_bound(bound_1().deanonymize());
//! ```
//!
//! …each of these fails to compile:
//!
//! ```compile_fail
//! # use lignin::{
//! #   auto_safety::{Align as _, AutoSafe, Deanonymize as _},
//! #   Node, ThreadBound, ThreadSafe,
//! # };
//! #
//! # fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! # fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! # fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! # fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! #
//! # fn allocate<'a, T>(value: T) -> &'a T { Box::leak(Box::new(value)) }
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
//! #   auto_safety::{Align as _, AutoSafe, Deanonymize as _},
//! #   Node, ThreadBound, ThreadSafe,
//! # };
//! #
//! # fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! # fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! # fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! # fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! #
//! # fn allocate<'a, T>(value: T) -> &'a T { Box::leak(Box::new(value)) }
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
//! #   auto_safety::{Align as _, AutoSafe, Deanonymize as _},
//! #   Node, ThreadBound, ThreadSafe,
//! # };
//! #
//! # fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! # fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! # fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! # fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! #
//! # fn allocate<'a, T>(value: T) -> &'a T { Box::leak(Box::new(value)) }
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
//! # assert_safe(safe_2().deanonymize());
//! # assert_bound(bound_2().deanonymize());
//! ```
//!
//! > You also have to do this to annotate the type of local variables…:
//! >
//! > ```
//! > # use lignin::{
//! > #   auto_safety::{Align as _, AutoSafe, Deanonymize as _},
//! > #   Node, ThreadBound, ThreadSafe,
//! > # };
//! > #
//! > # fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! > # fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! > # fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! > # fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! > #
//! > # fn allocate<'a, T>(value: T) -> &'a T { Box::leak(Box::new(value)) }
//! > #
//! > # fn assert_safe<'a>(value: Node<'a, ThreadSafe>) { }
//! > # fn assert_bound<'a>(value: Node<'a, ThreadBound>) { }
//! > #
//! > let safe_node: Node<_> = inferred_safe().deanonymize();
//! > let bound_node: Node<_> = inferred_bound().deanonymize();
//! > #
//! > # // No assert here! This test should fail if some some reason this fails to compile without further coercion.
//! > ```
//! >
//! > …or to specify a [`ThreadSafety`] in the return type:
//! >
//! > ```
//! > # use lignin::{
//! > #   auto_safety::{Align as _, AutoSafe, Deanonymize as _},
//! > #   Node, ThreadBound, ThreadSafe,
//! > # };
//! > #
//! > # fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! > # fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! > # fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! > # fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! > #
//! > # fn allocate<'a, T>(value: T) -> &'a T { Box::leak(Box::new(value)) }
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
//! #   auto_safety::{Align as _, AutoSafe, Deanonymize as _},
//! #   Node, ThreadBound, ThreadSafe,
//! # };
//! #
//! # fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! # fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! # fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! # fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! #
//! # fn allocate<'a, T>(value: T) -> &'a T { Box::leak(Box::new(value)) }
//! #
//! # fn assert_safe<'a>(value: Node<'a, ThreadSafe>) { }
//! # fn assert_bound<'a>(value: Node<'a, ThreadBound>) { }
//! #
//! let safe_node: Node<ThreadSafe> = safe().deanonymize();
//! //                                       ^^^^^^^^^^^
//! let bound_node: Node<ThreadBound> = bound().deanonymize();
//! //                                          ^^^^^^^^^^^
//! //
//! // warning:
//! //   use of deprecated associated function `lignin::auto_safety::<impl lignin::Node<'a, S>>::deanonymize`:
//! //   Call of `.deanonymize()` on named type.
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
//! #   auto_safety::{Align as _, AutoSafe, Deanonymize as _},
//! #   Node, ThreadBound, ThreadSafe,
//! # };
//! #
//! # fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! # fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! # fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! # fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! #
//! # fn allocate<'a, T>(value: T) -> &'a T { Box::leak(Box::new(value)) }
//! #
//! # fn assert_safe<'a>(value: Node<'a, ThreadSafe>) { }
//! # fn assert_bound<'a>(value: Node<'a, ThreadBound>) { }
//! #
//! let safe_node: Node::<ThreadSafe> = inferred_bound().deanonymize();
//! //             ------------------   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//! //             |                    expected struct `ThreadSafe`, found struct `ThreadBound`
//! //             expected due to this
//! //
//! // note: expected enum `Node<'_, ThreadSafe>`
//! //          found enum `Node<'_, ThreadBound>`
//! ```
//!
//! ```compile_fail
//! # use lignin::{
//! #   auto_safety::{Align as _, AutoSafe, Deanonymize as _},
//! #   Node, ThreadBound, ThreadSafe,
//! # };
//! #
//! # fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! # fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! # fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! # fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! #
//! # fn allocate<'a, T>(value: T) -> &'a T { Box::leak(Box::new(value)) }
//! #
//! # fn assert_safe<'a>(value: Node<'a, ThreadSafe>) { }
//! # fn assert_bound<'a>(value: Node<'a, ThreadBound>) { }
//! #
//! let bound_node: Node::<ThreadBound> = inferred_safe().deanonymize();
//! //              -------------------   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//! //              |                     expected struct `ThreadBound`, found struct `ThreadSafe`
//! //              expected due to this
//! //
//! // note: expected enum `Node<'_, ThreadBound>`
//! //          found enum `Node<'_, ThreadSafe>`
//! ```
//!
//! # Alignment
//!
//! TODO
//!
//! ## More lenient conversion with [`Into`]
//!
//! TODO
//!
//! # [`ThreadSafe`] Preference
//!
//! The Rust compiler can usually infer the correct [`ThreadSafety`] without annotations if valid choices are in any way limited in this regard.
//!
//! However, this isn't the case for most [`Vdom`] expressions without inputs…:
//!
//! ```compile_fail
//! # use lignin::{
//! #   auto_safety::{Align as _, AutoSafe, Deanonymize as _},
//! #   Node, ThreadBound, ThreadSafe,
//! # };
//! #
//! # fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! # fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! # fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! # fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! #
//! # fn allocate<'a, T>(value: T) -> &'a T { Box::leak(Box::new(value)) }
//! #
//! # fn assert_safe<'a>(value: Node<'a, ThreadSafe>) { }
//! # fn assert_bound<'a>(value: Node<'a, ThreadBound>) { }
//! #
//! let attempt_1 = Node::Multi(&[]);
//! //  ---------   ^^^^^^^^^^^       // See below.
//! ```
//!
//! …or if all inputs are thread-safe and [`.align()`](`Align::align`) is called on each of them:
//!
//! ```compile_fail
//! # use lignin::{
//! #   auto_safety::{Align as _, AutoSafe, Deanonymize as _},
//! #   Node, ThreadBound, ThreadSafe,
//! # };
//! #
//! # fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! # fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! # fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! # fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! #
//! # fn allocate<'a, T>(value: T) -> &'a T { Box::leak(Box::new(value)) }
//! #
//! # fn assert_safe<'a>(value: Node<'a, ThreadSafe>) { }
//! # fn assert_bound<'a>(value: Node<'a, ThreadBound>) { }
//! #
//! let attempt_2 = Node::Multi(allocate([safe().align(), inferred_safe().deanonymize().align()]));
//! //  ---------   ^^^^^^^^^^^ cannot infer type for type parameter `S` declared on the enum `Node`
//! //  consider giving `attempt_2` the explicit type `Node<'_, S>`, where the type parameter `S` is specified
//! //
//! // note: cannot satisfy `_: ThreadSafety`
//! // note: required by `Multi`
//! ```
//!
//! In these cases, you can call `.prefer_thread_safe()` on the indeterminate expression to nudge the compiler in the right direction.
//!
//! ```rust
//! # use lignin::{
//! #   auto_safety::{Align as _, AutoSafe, Deanonymize as _},
//! #   Node, ThreadBound, ThreadSafe,
//! # };
//! #
//! # fn safe<'a>() -> Node::<'a, ThreadSafe> { Node::Multi(&[]) }
//! # fn bound<'a>() -> Node::<'a, ThreadBound> { Node::Multi(&[]) }
//! # fn inferred_safe<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { safe() }
//! # fn inferred_bound<'a>() -> impl AutoSafe<Node::<'a, ThreadBound>> { bound() }
//! #
//! # fn allocate<'a, T>(value: T) -> &'a T { Box::leak(Box::new(value)) }
//! #
//! # fn assert_safe<'a>(value: Node<'a, ThreadSafe>) { }
//! # fn assert_bound<'a>(value: Node<'a, ThreadBound>) { }
//! #
//! let safe_1 = Node::Multi(&[]).prefer_thread_safe();
//!
//! let safe_2 = Node::Multi(allocate([
//!   safe().align(),
//!   inferred_safe().deanonymize().align(),
//! ])).prefer_thread_safe();
//! #
//! # assert_safe(safe_1);
//! # assert_safe(safe_2);
//! ```
//!
//! > This is implemented directly on the individual [`Vdom`] type variants, so no additional trait imports are necessary to use it.
//!
//! # Limiting [`AutoSafe`] Exposure
//!
//! Thread-safety inference is powerful, but also dangerous: A change deep in a library could cause a public function return type to shift, breaking compatibility with downstream crates.
//! For this reason, and because of its worse ergonomics, `-> impl AutoSafe<…>` should not be exposed in a crate's public API.
//!
//! A front-end template language or framework author may still want to avoid requiring explicit threading annotations in most cases.
//! Even in that case, it's possible to limit this feature to functions not externally visible, by aliasing it with a generated less visible trait:
//!
//! TODO

use crate::{
	Attribute, CallbackRef, Element, EventBinding, Node, ThreadBound, ThreadSafe, ThreadSafety,
	Vdom,
};

/// Deanonymize towards the general ([`ThreadBound`]) case. Used as `-> impl AutoSafe<…>`.
///
/// See module documentation for usage.
pub trait AutoSafe<BoundVariant>
where
	Self: Vdom + Align<BoundVariant>,
	BoundVariant: Vdom<ThreadSafety = ThreadBound>,
{
	/// Deanonymize towards a compatible concrete type.
	///
	/// This method is by reference, so it will resolve with lower priority than the by-value method on [`Deanonymize`].  
	/// Note that not all tooling will show the correct overload here, but the compiler knows which to pick.
	#[must_use]
	#[inline(always)] // Plain deref.
	fn deanonymize(&self) -> BoundVariant {
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
pub trait Deanonymize<SafeVariant>
where
	Self: Vdom + Send + Sync,
	SafeVariant: Vdom<ThreadSafety = ThreadSafe>,
{
	/// Deanonymize towards a compatible concrete type.
	///
	/// This method is by value, so it will resolve with higher priority than the by-reference method on [`AutoSafe`].  
	/// Note that not all tooling will show the correct overload here, but the compiler knows which to pick.
	#[must_use]
	#[inline(always)] // No-op.
	fn deanonymize(self) -> SafeVariant {
		unsafe {
			// SAFETY:
			// Under normal circumstances, this trait or method would have to be `unsafe`.
			// However, we're ensuring only sound implementations exist by sealing it and carefully implementing it only across layout-compatible types.
			*(&self as *const Self).cast()
		}
	}
}

impl<'a, S, T> AutoSafe<T> for S
where
	S: Vdom + Align<T>,
	T: Vdom<ThreadSafety = ThreadBound>,
{
}

/// Contextually thread-binds an instance, or not. Use only without qualification.
///
/// This trait acts as [`Into`] on and between variants of the same [`Vdom`] type, but without raising `useless_conversion` warnings.
///
/// See module documentation for when to use this trait and when it's unnecessary.
pub trait Align<T: Vdom>: Vdom {
	/// Contextually thread-binds an instance, or not. Use only without qualification.
	#[allow(clippy::inline_always)]
	#[inline(always)] // No-op.
	fn align(self) -> T {
		unsafe {
			// SAFETY: This trait is sealed and only implemented on and across compatible types.
			*(&self as *const Self).cast()
		}
	}
}

/// Not derived from the [`Into`] constraints since those are too broad.
impl<T> Align<T> for T where T: Vdom {}

macro_rules! deanonymize_on_named {
	() => {
		/// When called on an opaque type, deanonymizes it into the underlying named type.
		///
		/// **Both** [`AutoSafe`] and [`Deanonymize`] must be in scope and the method must be called *without qualification* for this to work.
		///
		/// > Calling this method on a named type returns the value and type unchanged and produces a deprecation warning.
		#[deprecated = "Call of `.deanonymize()` on named type."]
		#[must_use]
		#[inline(always)] // No-op.
		pub fn deanonymize(self) -> Self {
			self
		}
	};
}

macro_rules! prefer_thread_safe_safe {
	{
		$(#[$($attrs:tt)*])*
	} => {
		/// Gently nudges the compiler to choose the [`ThreadSafe`] version of a value if both are possible.
		///
		/// This method is by value, so it will resolve with higher priority than the by-reference method on the [`ThreadBound`] type.
		///
		/// Note that not all tooling will show the correct overload here, but the compiler knows which to pick.
		$(#[$($attrs)*])*
		#[must_use]
		#[inline(always)] // No-op.
		pub fn prefer_thread_safe(self) -> Self {
			self
		}
	};
}

macro_rules! prefer_thread_safe_bound {
	() => {
		/// Gently nudges the compiler to choose the [`ThreadSafe`] version of a value if both are is possible.
		///
		/// This method is by reference, so it will resolve with lower priority than the by-value method on the [`ThreadSafe`] type.
		///
		/// Note that not all tooling will show the correct overload here, but the compiler knows which to pick.
		#[must_use]
		#[inline(always)] // Plain deref.
		pub fn prefer_thread_safe(&self) -> Self {
			*self
		}
	};
}

impl<'a> Attribute<'a> {
	deanonymize_on_named!();
	prefer_thread_safe_safe! {
		///
		/// > Calling this method on [`Attribute`] produces a deprecation warning since the type is always [`ThreadSafe`].
		#[deprecated = "Call of `.prefer_thread_safe()` on `Attribute`."]
	}
}

macro_rules! impl_auto_safety {
	($($Name:ident),*$(,)?) => {$(
		impl<'a, S: ThreadSafety> $Name<'a, S> {
			deanonymize_on_named!();
		}
		impl<'a> $Name<'a, ThreadSafe> {
			prefer_thread_safe_safe!();
		}
		impl<'a> $Name<'a, ThreadBound> {
			prefer_thread_safe_bound!();
		}
		impl<'a, O> Deanonymize<$Name<'a, ThreadSafe>> for O where
			O: Send + Sync + AutoSafe<$Name<'a, ThreadBound>>,
		{}
		impl<'a> Align<$Name<'a, ThreadBound>> for $Name<'a, ThreadSafe> {}
	)*};
}

impl_auto_safety!(Element, EventBinding, Node);

impl<S: ThreadSafety, T> CallbackRef<S, T> {
	deanonymize_on_named!();
}
impl<T> CallbackRef<ThreadSafe, T> {
	prefer_thread_safe_safe!();
}
impl<T> CallbackRef<ThreadBound, T> {
	prefer_thread_safe_bound!();
}
impl<T, O> Deanonymize<CallbackRef<ThreadSafe, T>> for O where
	O: Send + Sync + AutoSafe<CallbackRef<ThreadBound, T>>
{
}
impl<T> Align<CallbackRef<ThreadBound, T>> for CallbackRef<ThreadSafe, T> {}