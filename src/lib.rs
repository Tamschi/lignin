#![doc(html_root_url = "https://docs.rs/lignin/0.0.5")]
#![warn(clippy::pedantic)]
//TODO: web-sys, annoyingly, makes this moot and also pulls in a number of proc macro dependencies.
// There should be some way to not depend on it at all when not needed, and to make the `"callbacks"` feature not default.
// (Replace `Option<web_sys::Comment>` with an always-thereto-convertible `DomRef<Comment>` that's an empty enum without `"callbacks"`?)
#![no_std]

#[cfg(doctest)]
pub mod readme {
	doc_comment::doctest!("../README.md");
}

mod callback_registry;
pub use callback_registry::*;

pub mod remnants;

use remnants::RemnantSite;

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Node<'a> {
	Comment {
		comment: &'a str,
		dom_binding: Option<CallbackRef<Option<web_sys::Comment>>>,
	},
	Element {
		element: &'a Element<'a>,
		dom_binding: Option<CallbackRef<Option<web_sys::HtmlElement>>>,
	},
	Ref(&'a Node<'a>),
	Multi(&'a [Node<'a>]),
	Text {
		text: &'a str,
		dom_binding: Option<CallbackRef<Option<web_sys::Text>>>,
	},
	RemnantSite(&'a RemnantSite),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Element<'a> {
	pub name: &'a str,
	pub attributes: &'a [Attribute<'a>],
	pub content: &'a [Node<'a>],
	#[cfg_attr(feature = "debug", derivative(Debug = "ignore"))]
	pub event_bindings: &'a [EventBinding<'a>],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventBinding<'a> {
	pub name: &'a str,
	pub callback: CallbackRef<web_sys::Event>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Attribute<'a> {
	pub name: &'a str,
	pub value: &'a str,
}

impl<'a: 'b, 'b> From<&'a Element<'a>> for Node<'b> {
	fn from(element: &'a Element<'a>) -> Self {
		Self::Element {
			element,
			dom_binding: None,
		}
	}
}

impl<'a: 'b, 'b> From<&'a mut Element<'a>> for Node<'b> {
	fn from(element: &'a mut Element<'a>) -> Self {
		Self::Element {
			element,
			dom_binding: None,
		}
	}
}
