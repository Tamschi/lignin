#![doc(html_root_url = "https://docs.rs/lignin/0.0.5")]
#![no_std]
#![warn(clippy::pedantic)]
// #![warn(missing_docs)] //TODO

//! `lignin`, named after the structural polymer found in plants, is a lightweight but comprehensive VDOM data type library for use in a wider web context.
//!
//! # Implementation Contract
//!
//! > **This is not a soundness contract**. Code using this crate must not rely on it for soundness.
//! > However, it is free to panic when encountering an incorrect implementation.
//!
//! ## Correctness
//!
//! The DOM may contain extra siblings past the nodes mentioned in the VDOM. Renderers must ignore them.
//!
//! Similarly, the DOM may contain extra attributes and event bindings. Renderers must ignore them unless attributes collide.  
//! Components must clean up extra attributes and event listeners they have previous added to the DOM via the DOM API on teardown.
//!
//! > This simplifies renderers and allows reuse of DOM nodes between components, which in turn reduces the amount of DOM API calls necessary.
//!
//! See also the implementation contracts on [`DomRef`] and [`Node::Keyed`].
//!
//! ## Performance
//!
//! While the order of [attributes](https://developer.mozilla.org/en-US/docs/Web/API/Element/attributes) reported by the DOM API in browsers isn't specified and event listeners can't be examined this way,
//! components *should* stick to a relatively consistent order here and place conditional attributes and event bindings past always present ones in the respective slices.
//!
//! When adding or removing [`Node`]s dynamically between updates, components should wrap lists in [`Node::Multi`] and otherwise insert an empty [`Node::Multi([&[])`](`Node::Multi`) as placeholder for an absent element.
//!
//! > This both together allows better and easier diff optimization in renderers, but otherwise mustn't be a strict requirement for compatibility.
//!
//! # Deep Comparisons
//!
//! All [`core`] comparison traits ([`PartialEq`], [`Eq`], [`PartialOrd`] and [`Ord`]) operate on the entire reachable VDOM graph and are implemented recursively where applicable.
//!
//! Note that [`CallbackRef`]s derived from separate instances of [`CallbackRegistration`] are still considered distinct,
//! regardless of the `receiver` and `handler` used to make them.
//! However, without the `"callbacks"` feature, all [`CallbackRef`] instances are inactive **and indistinct**.
//!
//! For shallow comparisons, access and compare fields directly or [memoize](`Node::Memoized`) parts of the GUI.
//!
//! # Notes on Performance
//!
//! ## Clone
//!
//! [`Clone`] is always implemented via [`Copy`] in this crate, since none of the instances provide heap storage.
//!
//! ## Hashing
//!
//! As shallow hashes would easily collide for most applications where VDOM hashing comes up,
//! [`Hash`](`core::hash::Hash`) is implemented recursively in this crate and is potentially expensive.
//! The same applies to [`PartialEq`], [`Eq`], [`PartialOrd`] and [`Ord`].
//!
//! As an exception, [`Memoized` nodes](`Node::Memoized`) are compared only by their [`state_key`](`Node::Memoized::state_key`).
//! Their [`content`](`Node::Memoized::content`) is ignored for comparisons and does not factor into their [hash](`core::hash`).
//!
//! **`lignin` does not implement hash caching by itself**, so users of [`HashMap`](https://doc.rust-lang.org/stable/std/collections/struct.HashMap.html) or similar containers should wrap node graphs in a "`HashCached<T>`" type first.
//!
//! # Limitations
//!
//! As `lignin` targets HTML and DOM rather than XML, it does not support [processing instructions](https://developer.mozilla.org/en-US/docs/Web/API/ProcessingInstruction).
//!
//! For the same reason, there is formally no information about VDOM identity, which could be used to render self-referential XML documents.
//!
//! > In practice, it **may** be possible to determine identity by comparing pointers, but there are caveats around pointer comparisons.  
//! > For example, `lignin` lets apps freely mix instances allocated on the stack and heap in the same VDOM graph, but [pointer equality between those isn't necessarily meaningful](TODO).
//! >
//! > The implementation itself is also quite error-prone on types that are [`Copy`] due to implicit by-value copies there.
//! >
//! > Proceed with extreme caution and architecture assertions if you must!
//!
//! Element and attribute names are always plain `&str`s, which isn't ideal for software that renders its GUI more directly than through a web browser.
//! I'm open to maintaining a generic fork if there's interest in this regard.
//!
//! While the `"callbacks"` feature is disabled, all callback management is erased.
//! This makes `lignin` faster and removes usage limits, but removes unique identities from [`CallbackRegistration`] and [`CallbackRef`], which affects comparisons and hashing.
#[cfg(doctest)]
pub mod readme {
	doc_comment::doctest!("../README.md");
}

pub mod auto_safety;
pub mod callback_registry;
#[doc(hidden)]
pub mod remnants;
pub mod web;

pub use callback_registry::{CallbackRef, CallbackRegistration};
pub use web::{DomRef, Materialize};

mod ergonomics;

use core::{convert::Infallible, fmt::Debug, hash::Hash, marker::PhantomData};
use remnants::RemnantSite;
use sealed::Sealed;

//TODO: Should `Vdom` types implement `PartialOrd` and `Ord`?

/// [`Vdom`] A single generic VDOM node.
pub enum Node<'a, S: ThreadSafety> {
	/// Represents a DOM [Comment](https://developer.mozilla.org/en-US/docs/Web/API/Comment) node.
	Comment {
		comment: &'a str,
		dom_binding: Option<CallbackRef<S, DomRef<web::Comment>>>,
	},
	/// Represents a single [HTMLElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement).
	Element {
		element: &'a Element<'a, S>,
		dom_binding: Option<CallbackRef<S, DomRef<web::HtmlElement>>>,
	},
	/// Uses shallow comparison and hashing based on its `state_key` only.
	///
	/// A (good enough) `content` [hash](`core::hash`) makes for a good `state_key`, but this isn't the only possible scheme and may not be the optimal one for your use case.
	///
	/// TODO?: Explain a bit more what's specified and what isn't.
	Memoized {
		state_key: u128,
		content: &'a Node<'a, S>,
	},
	/// A DOM-transparent sequence of VDOM nodes. Used to hint diffs in case of additions and removals.
	Multi(&'a [Node<'a, S>]),
	/// A sequence of VDOM nodes that's transparent at rest, but encodes information on how to reuse and reorder elements when diffing.
	///
	/// **List indices are bad [`ReorderableFragment::dom_key`] values!**
	/// Use the [`Multi`](`Node::Multi`) variant instead if you don't track component identity.
	///
	/// # Implementation Contract
	///
	/// > **This is not a soundness contract**. Code using this crate must not rely on it for soundness.
	/// > However, it is free to panic when encountering an incorrect implementation.
	///
	/// The [`ReorderableFragment::dom_key`] values must be unique within a slice reference by a [`Node::Keyed`] instance.
	///
	/// > This rule does not apply between distinct [`ReorderableFragment`] slices, even if they overlap in memory or one is reachable from the other.
	///TODO
	Keyed(&'a [ReorderableFragment<'a, S>]),
	//TODO: Map that allows shuffling of elements!
	/// Represents a DOM [Text](https://developer.mozilla.org/en-US/docs/Web/API/Text) node.
	Text {
		text: &'a str,
		dom_binding: Option<CallbackRef<S, DomRef<web::Text>>>,
	},
	RemnantSite(&'a RemnantSite),
}

/// [`Vdom`] A VDOM node that has its DOM identity preserved during DOM updates even after being repositioned within a (path-)matching [`Node::Keyed`].
pub struct ReorderableFragment<'a, S: ThreadSafety> {
	pub dom_key: usize,
	pub content: &'a Node<'a, S>,
}

#[allow(clippy::doc_markdown)]
/// [`Vdom`] Represents a single [*HTMLElement*](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement) as `name`, `attributes`, `content` and `event_bindings`.
pub struct Element<'a, S: ThreadSafety> {
	pub name: &'a str,
	pub attributes: &'a [Attribute<'a>],
	pub content: Node<'a, S>,
	pub event_bindings: &'a [EventBinding<'a, S>],
}

/// [`Vdom`] Represents a single DOM event binding with `name` and `callback`.
pub struct EventBinding<'a, S: ThreadSafety> {
	pub name: &'a str,
	pub callback: CallbackRef<S, web::Event>,
}

/// [`Vdom`] Represents a single HTML [*Attr*](https://developer.mozilla.org/en-US/docs/Web/API/Attr) with `name` and `value`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct Attribute<'a> {
	pub name: &'a str,
	pub value: &'a str,
}

mod sealed {
	use super::{ThreadBound, ThreadSafe};
	use crate::{
		remnants::RemnantSite, Attribute, CallbackRef, Element, EventBinding, Node,
		ReorderableFragment, ThreadSafety,
	};

	//TODO: Move these bounds to `Vdom`.
	pub trait Sealed {}
	impl Sealed for ThreadBound {}
	impl Sealed for ThreadSafe {}
	impl<'a> Sealed for Attribute<'a> {}
	impl<S: ThreadSafety, T> Sealed for CallbackRef<S, T> {}
	impl<'a, S: ThreadSafety> Sealed for Element<'a, S> {}
	impl<'a, S: ThreadSafety> Sealed for EventBinding<'a, S> {}
	impl<'a, S: ThreadSafety> Sealed for Node<'a, S> {}
	impl<'a, S: ThreadSafety> Sealed for ReorderableFragment<'a, S> {}
	impl Sealed for RemnantSite {}
}

/// Marker trait for thread-safety tokens.
pub trait ThreadSafety:
	Sealed + Sized + Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash
{
}

/// [`ThreadSafety`] marker for `!Send + !Sync`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ThreadBound(
	/// Neither [`Send`] nor [`Sync`].
	pub PhantomData<*const ()>,
	/// [Uninhabited.](https://doc.rust-lang.org/nomicon/exotic-sizes.html#empty-types)
	pub Infallible,
);
/// [`ThreadSafety`] marker for `Send + Sync`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ThreadSafe(
	/// This field here technically doesn't matter, but I went with something to match [`ThreadBound`].
	pub PhantomData<&'static ()>,
	/// [Uninhabited.](https://doc.rust-lang.org/nomicon/exotic-sizes.html#empty-types)
	pub Infallible,
);
impl ThreadSafety for ThreadBound {}
impl ThreadSafety for ThreadSafe {}

/// Marker trait for VDOM data types, which (almost) all vary by [`ThreadSafety`].
///
/// Somewhat uselessly implemented on [`Attribute`], which is always [`ThreadSafe`].
pub trait Vdom: Sealed + Sized + Debug + Clone + Copy + PartialEq + Eq + Hash {
	type ThreadSafety: ThreadSafety;
}

impl<'a> Vdom for Attribute<'a> {
	type ThreadSafety = ThreadSafe;
}

macro_rules! vdom_impls {
	($($name:ident),*$(,)?) => {$(
		impl<'a, S> Vdom for $name<'a, S> where
			S: ThreadSafety,
		{
			type ThreadSafety = S;
		}
	)*};
}
vdom_impls!(Element, EventBinding, Node, ReorderableFragment);

impl<S: ThreadSafety, T> Vdom for CallbackRef<S, T> {
	type ThreadSafety = S;
}
