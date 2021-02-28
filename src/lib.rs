#![doc(html_root_url = "https://docs.rs/lignin/0.0.5")]
#![no_std]
#![warn(clippy::pedantic)]
// #![warn(missing_docs)] //TODO

//! # Implementation Contract
//!
//! > **This is not a soundness contract**. Code using this crate must not rely on it for soundness. However, it is free to panic when encountering an incorrect implementation.
//!
//! The DOM may contain extra siblings past the nodes mentioned in the VDOM. Renderers must ignore them.
//!
//! Similarly, the DOM may contain extra attributes and event bindings. Renderers must ignore them unless attributes collide. Components must clean up the ones they have created on teardown.
//!
//! See also the implementation contract on [`DomRef`].

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

/// [`Vdom`]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
	use core::{fmt::Debug, hash::Hash};

	//TODO: Move these bounds to `Vdom`.
	pub trait Sealed: Sized + Debug + Clone + Copy + PartialEq + Eq + Hash {}
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
pub trait ThreadSafety: Sealed {}

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
pub trait Vdom: Sealed {
	type ThreadSafety: ThreadSafety;
}

impl<'a> Vdom for Attribute<'a> {
	type ThreadSafety = ThreadSafe;
}

macro_rules! vdom_impls {
	($($name:ident),*$(,)?) => {$(
		impl<'a, S: ThreadSafety> Vdom for $name<'a, S> {
			type ThreadSafety = S;
		}
	)*};
}
vdom_impls!(Element, EventBinding, Node);

impl<S: ThreadSafety, T> Vdom for CallbackRef<S, T> {
	type ThreadSafety = S;
}
