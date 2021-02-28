#![doc(html_root_url = "https://docs.rs/lignin/0.0.5")]
#![no_std]
#![warn(clippy::pedantic)]
// #![warn(missing_docs)] //TODO

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
//! > `lignin`'s types don't contain enough information to establish unique identity.
//! > Additionally, this allows reuse of DOM nodes between components and as such reduces the amount of DOM API calls necessary.
//!
//! See also the implementation contract on [`DomRef`].
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
//! For shallow comparisons, access and compare fields directly.
//!
//! # Notes on Performance
//!
//! ## Clone
//!
//! [`Clone`] is always implemented via [`Copy`] in this crate, since none of the types provide heap storage.
//!
//! ## Hashing
//!
//! As shallow hashes would easily collide for most applications where VDOM hashing comes up,
//! [`Hash`](`core::hash::Hash`) is implemented recursively in this crate and is potentially expensive.
//! The same applies to [`PartialEq`] and [`Eq`].
//!
//! **`lignin` does not implement hash caching by itself**, so users of a [`HashMap`](https://doc.rust-lang.org/stable/std/collections/struct.HashMap.html) or similar container should wrap node graphs in a "`HashCached<T>`" type first.
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

//TODO: The derives emit bounds on S here, which aren't necessary but appear in the documentation.
// It would be cleaner to explicitly implement all of these traits.

/// [`Vdom`] A single VDOM node.
#[non_exhaustive]
pub enum Node<'a, S: ThreadSafety> {
	Comment {
		comment: &'a str,
		dom_binding: Option<CallbackRef<S, DomRef<web::Comment>>>,
	},
	Element {
		element: &'a Element<'a, S>,
		dom_binding: Option<CallbackRef<S, DomRef<web::HtmlElement>>>,
	},
	Ref(&'a Node<'a, S>),
	Multi(&'a [Node<'a, S>]),
	Text {
		text: &'a str,
		dom_binding: Option<CallbackRef<S, DomRef<web::Text>>>,
	},
	#[doc(hidden)]
	RemnantSite(&'a RemnantSite),
}

/// [`Vdom`] A single HTML element with `name`, `attributes`, `content` and `event_bindings`.
pub struct Element<'a, S: ThreadSafety> {
	pub name: &'a str,
	pub attributes: &'a [Attribute<'a>],
	pub content: &'a [Node<'a, S>],
	pub event_bindings: &'a [EventBinding<'a, S>],
}

/// [`Vdom`] A single DOM event binding with `name` and `callback`.
pub struct EventBinding<'a, S: ThreadSafety> {
	pub name: &'a str,
	pub callback: CallbackRef<S, web::Event>,
}

/// [`Vdom`] A single HTML attribute with `name` and `value`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct Attribute<'a> {
	pub name: &'a str,
	pub value: &'a str,
}

mod sealed {
	use super::{ThreadBound, ThreadSafe};
	use crate::{
		remnants::RemnantSite, Attribute, CallbackRef, Element, EventBinding, Node, ThreadSafety,
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
vdom_impls!(Element, EventBinding, Node);

impl<S: ThreadSafety, T> Vdom for CallbackRef<S, T> {
	type ThreadSafety = S;
}
