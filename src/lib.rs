#![doc(html_root_url = "https://docs.rs/lignin/0.0.5")]
#![no_std]
#![warn(clippy::pedantic)]

//! # Implementation Contract
//!
//! > **This is not a soundness contract**. Code using this crate must not rely on it for soundness. However, it is free to panic when encountering an incorrect implementation.
//!
//! The DOM may contain extra siblings past the nodes mentioned in the VDOM. Renderers must ignore them.
//!
//! Similarly, the DOM may contain extra attributes and event bindings. Renderers must ignore them unless attributes collide. Components must clean up the ones they have created on teardown.

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

use core::{fmt::Debug, hash::Hash, marker::PhantomData};
use remnants::RemnantSite;
use sealed::Sealed;

//TODO: The derives emit bounds on S here, which aren't necessary but appear in the documentation.
// It would be cleaner to explicitly implement all of these traits.

/// `ThreadBindable`
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Node<'a, S: ThreadSafety> {
	Comment {
		comment: &'a str,
		dom_binding: Option<CallbackRef<DomRef<web::Comment>, S>>,
	},
	Element {
		element: &'a Element<'a, S>,
		dom_binding: Option<CallbackRef<DomRef<web::HtmlElement>, S>>,
	},
	Ref(&'a Node<'a, S>),
	Multi(&'a [Node<'a, S>]),
	Text {
		text: &'a str,
		dom_binding: Option<CallbackRef<DomRef<web::Text>, S>>,
	},
	#[doc(hidden)]
	RemnantSite(&'a RemnantSite),
}

/// `ThreadBindable`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Element<'a, S: ThreadSafety> {
	pub name: &'a str,
	pub attributes: &'a [Attribute<'a>],
	pub content: &'a [Node<'a, S>],
	pub event_bindings: &'a [EventBinding<'a, S>],
}

/// `ThreadBindable`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventBinding<'a, S: ThreadSafety> {
	pub name: &'a str,
	pub callback: CallbackRef<web::Event, S>,
}

/// `ThreadBindable`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Attribute<'a> {
	pub name: &'a str,
	pub value: &'a str,
}

impl<'a: 'b, 'b, S: ThreadSafety> From<&'a Element<'a, S>> for Node<'b, S> {
	fn from(element: &'a Element<'a, S>) -> Self {
		Self::Element {
			element,
			dom_binding: None,
		}
	}
}

impl<'a: 'b, 'b, S: ThreadSafety> From<&'a mut Element<'a, S>> for Node<'b, S> {
	fn from(element: &'a mut Element<'a, S>) -> Self {
		Self::Element {
			element,
			dom_binding: None,
		}
	}
}

mod sealed {
	use super::{ThreadBound, ThreadSafe};

	pub trait Sealed {}
	impl Sealed for ThreadBound {}
	impl Sealed for ThreadSafe {}
}

pub trait ThreadSafety:
	Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash + Sealed
{
}

/// [`ThreadSafety`] marker for `!Send + !Sync`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ThreadBound(PhantomData<*const ()>);
/// [`ThreadSafety`] marker for `Send + Sync`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ThreadSafe(
	/// The type here doesn't matter especially (as long as there's a private field to prevent construction),
	/// but since the [`ThreadSafety`] types are stand-ins for references, I went with one that resembles that.
	PhantomData<&'static ()>,
);
impl ThreadSafety for ThreadBound {}
impl ThreadSafety for ThreadSafe {}
