#![doc(html_root_url = "https://docs.rs/lignin/0.0.5")]
#![no_std]
#![warn(clippy::pedantic)]

//! # Implementation Contract
//!
//! > **This is not a soundness contract**. Code using this crate must not rely on it for soundness. However, it is free to panic when encountering an incorrect implementation.
//!
//! The DOM may contain extra sibling past the nodes mentioned in the VDOM. Renderers must ignore them.
//!
//! Similarly, the DOM may contain extra attributes and event bindings. Renderers must ignore them unless attributes collide. Components must clean up the ones they have created on teardown.

#[cfg(doctest)]
pub mod readme {
	doc_comment::doctest!("../README.md");
}

pub mod callback_registry;
pub mod remnants;
pub mod web;

pub use callback_registry::{CallbackRef, CallbackRegistration};
use sealed::Sealed;
pub use web::{DomRef, Materialize};

use remnants::RemnantSite;

mod sealed {
	use crate::{Node, SendSyncness};

	pub trait Sealed {}
	impl Sealed for &() {}
	impl Sealed for *const () {}
	impl<'a, S: SendSyncness> Sealed for Node<'a, S> {}
}

pub trait SendSyncness: Sealed {}
impl SendSyncness for &() {}
impl SendSyncness for *const () {}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Node<'a, S: SendSyncness> {
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
	RemnantSite(&'a RemnantSite),
}
pub trait AutoNode<'a>: Sealed {
	fn deanonymize(&self) -> Node<'a, *const ()>;
}
impl<'a, S: SendSyncness> AutoNode<'a> for Node<'a, S> {
	fn deanonymize(&self) -> Node<'a, *const ()> {
		unsafe { *(self as *const _ as *const _) }
	}
}
pub trait Deanonymize<'a> {
	type Named;
	fn deanonymize(self) -> Self::Named;
}
impl<'a, T> Deanonymize<'a> for T
where
	T: AutoNode<'a> + Send + Sync,
{
	type Named = Node<'a, &'static ()>;
	#[must_use]
	#[allow(clippy::inline_always)]
	#[inline(always)] // No-op.
	fn deanonymize(self) -> Self::Named {
		unsafe { *(&self as *const _ as *const _) }
	}
}
impl<'a> Node<'a, &()> {
	#[must_use]
	#[allow(clippy::inline_always)]
	#[inline(always)] // No-op.
	pub fn prefer_threadsafe(self) -> Self {
		self
	}
}
impl<'a> Node<'a, *const ()> {
	#[must_use]
	#[allow(clippy::inline_always)]
	#[inline(always)] // No-op.
	pub fn prefer_threadsafe(&self) -> Self {
		*self
	}
}
impl<'a> From<Node<'a, &()>> for Node<'a, *const ()> {
	#[allow(clippy::inline_always)]
	#[inline(always)] // No-op.
	fn from(threadsafe: Node<'a, &()>) -> Self {
		unsafe { *(&threadsafe as *const _ as *const _) }
	}
}
pub trait Align<T>: Into<T> {
	#[allow(clippy::inline_always)]
	#[inline(always)]
	fn align(self) -> T {
		self.into()
	}
}
impl<T: Into<U>, U> Align<U> for T {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Element<'a, S: SendSyncness> {
	pub name: &'a str,
	pub attributes: &'a [Attribute<'a>],
	pub content: &'a [Node<'a, S>],
	pub event_bindings: &'a [EventBinding<'a, S>],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventBinding<'a, S: SendSyncness> {
	pub name: &'a str,
	pub callback: CallbackRef<web::Event, S>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Attribute<'a> {
	pub name: &'a str,
	pub value: &'a str,
}

impl<'a: 'b, 'b, S: SendSyncness> From<&'a Element<'a, S>> for Node<'b, S> {
	fn from(element: &'a Element<'a, S>) -> Self {
		Self::Element {
			element,
			dom_binding: None,
		}
	}
}

impl<'a: 'b, 'b, S: SendSyncness> From<&'a mut Element<'a, S>> for Node<'b, S> {
	fn from(element: &'a mut Element<'a, S>) -> Self {
		Self::Element {
			element,
			dom_binding: None,
		}
	}
}
