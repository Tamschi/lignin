#![doc(html_root_url = "https://docs.rs/lignin/0.0.5")]
#![no_std]
#![warn(clippy::pedantic)]

#[cfg(doctest)]
pub mod readme {
	doc_comment::doctest!("../README.md");
}

pub mod callback_registry;
pub mod remnants;
pub mod web;

pub use callback_registry::{CallbackRef, CallbackRegistration};
pub use web::Materialize;

use remnants::RemnantSite;

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Node<'a> {
	Comment {
		comment: &'a str,
		dom_binding: Option<CallbackRef<Option<web::Comment>>>,
	},
	Element {
		element: &'a Element<'a>,
		dom_binding: Option<CallbackRef<Option<web::HtmlElement>>>,
	},
	Ref(&'a Node<'a>),
	Multi(&'a [Node<'a>]),
	Text {
		text: &'a str,
		dom_binding: Option<CallbackRef<Option<web::Text>>>,
	},
	RemnantSite(&'a RemnantSite),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Element<'a> {
	pub name: &'a str,
	pub attributes: &'a [Attribute<'a>],
	pub content: &'a [Node<'a>],
	#[cfg_attr(feature = "debug", derivative(Debug = "ignore"))]
	pub event_bindings: &'a [EventBinding<'a>],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventBinding<'a> {
	pub name: &'a str,
	pub callback: CallbackRef<web::Event>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
