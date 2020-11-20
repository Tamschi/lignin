use crate::Node;
use bumpalo::Bump;
use std::{error::Error, sync::Arc};

#[cfg(feature = "debug")]
use {core::fmt::Debug, derivative::Derivative};

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct RemnantSite<'a> {
	pub key: Arc<()>,
	pub content: &'a Node<'a>,
	pub remnant_callback: RemnantRenderCallback,
}

#[cfg_attr(feature = "debug", derive(Derivative))]
#[cfg_attr(feature = "debug", derivative(Debug))]
pub struct RemnantRenderCallback(
	#[cfg_attr(feature = "debug", derivative(Debug = "ignore"))]
	#[allow(clippy::type_complexity)]
	pub Box<dyn FnOnce(&'_ Bump) -> Result<RemnantState<'_>, Box<dyn Error>>>,
);

pub enum RemnantState<'a> {
	Bound(&'a Node<'a>, Option<RemnantRenderCallback>),
	Vanished,
}
