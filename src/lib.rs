#![doc(html_root_url = "https://docs.rs/lignin/0.0.2")]
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

use core::any::Any;
use std::{pin::Pin, rc::Rc};

#[cfg(doctest)]
pub mod readme {
	doc_comment::doctest!("../README.md");
}

pub use bumpalo;

#[cfg(feature = "debug")]
use {core::fmt::Debug, derivative::Derivative};

pub mod remnants;

use remnants::RemnantSite;

#[non_exhaustive]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Node<'a> {
	Comment(&'a str),
	Element(&'a Element<'a>),
	Ref(&'a Node<'a>),
	Multi(&'a [Node<'a>]),
	Text(&'a str),
	RemnantSite(&'a RemnantSite<'a>),
}

#[cfg_attr(feature = "debug", derive(Derivative))]
#[cfg_attr(feature = "debug", derivative(Debug))]
pub struct Element<'a> {
	pub name: &'a str,
	pub attributes: &'a [Attribute<'a>],
	pub content: &'a [Node<'a>],
	#[cfg_attr(feature = "debug", derivative(Debug = "ignore"))]
	pub event_bindings: &'a [EventBinding<'a>],
}

pub struct EventBinding<'a> {
	pub name: &'a str,
	pub context: &'a dyn Any,
	pub handler: Pin<Rc<dyn Fn(&dyn Any) + 'a>>,
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Attribute<'a> {
	pub name: &'a str,
	pub value: &'a str,
}

impl<'a: 'b, 'b> From<&'a Element<'a>> for Node<'b> {
	fn from(element: &'a Element<'a>) -> Self {
		Self::Element(element)
	}
}

impl<'a: 'b, 'b> From<&'a mut Element<'a>> for Node<'b> {
	fn from(element: &'a mut Element<'a>) -> Self {
		Self::Element(element)
	}
}
