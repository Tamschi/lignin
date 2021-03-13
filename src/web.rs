//! Erasable web type stand-ins used as callback parameters.
//!
//! `struct`s in this module are only inhabited with the `"callbacks"` feature enabled.  
//! Without it, they become [uninhabited](https://doc.rust-lang.org/nomicon/exotic-sizes.html#empty-types) and are erased entirely at compile-time, so any code paths that depend on them can in turn be removed too.
#![allow(clippy::inline_always)]

use crate::sealed::Sealed;

/// Used as DOM reference callback parameter. (Expand for implementation contract!)
///
/// When you receive a [`DomRef`] containing a stand-in type, use [`Materialize::materialize`] to convert it to the actual value.
///
/// # Implementation Contract
///
/// > **This is not a soundness contract**. Code using this type must not rely on it for soundness. However, it is free to panic when encountering an incorrect implementation.
///
/// ## For VDOM-to-DOM renderers:
///
/// If a renderer invoked a callback with the [`Added`](`DomRef::Added`) variant, it **must** invoke it with the [`Removing`](`DomRef::Removing`) variant before destroying or reusing the relevant part of the DOM.
///
/// This includes cases where the identity of the `CallbackRef` or DOM node changes, in which case the new reference is [`Added`](`DomRef::Added`) after, in this order, [`Removing`](`DomRef::Removing`) the old reference and updating the relevant part(s) of the DOM.
///
/// ## For apps/VDOM renderers:
///
/// Tearing down and reconstructing the child DOM according to the current child VDOM must be possible at any time.
///
/// <!-- The above is a fairly strict constraint. It's here so that renderers aren't forced to (partially) double-buffer the VDOM, even if the current "default" renderer `lignin-dom` does so. -->
///
/// Please refer to the variant documentation for more information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DomRef<T> {
	/// When constructing the DOM, this variant is passed **after** all child elements have been processed and, if applicable, the element has been added to the document tree.
	///
	/// In particular, this means:
	///
	/// - Manipulating child elements is possible (but this can cause panics to occur later on if an incompatible child diff occurs).
	/// - Traversing ancestors and their attributes should work.
	/// - Scrolling the node into view and grabbing focus should work.
	/// - **Any siblings and ancestor siblings may be in an indeterminate state at this point!**
	Added(T),
	/// When tearing down the DOM, this variant is passed **before** any child elements are processed and, if applicable, the element is removed from to the document tree.
	///
	/// In particular, this means:
	///
	/// - **Child elements must be restored to a clean state compatible with their VDOM here!**
	/// - Traversing ancestors and their attributes should still work.
	/// - **Any siblings and ancestor siblings may be in an indeterminate state at this point!**
	Removing(T),
}
impl<T> Sealed for DomRef<T> {}

macro_rules! web_types {
	{$(
		$(#[$($attrs:tt)*])*
		($container:ident, $container_str:literal) => $contents:ty
	),*$(,)?} => {$(
		// It's unfortunately not possible to puzzle the first line together like below, since it ends up cut off in the overview.
		$(#[$($attrs)*])*
		///
		/// Use [`Materialize::materialize`] to convert it to the actual value.
		#[cfg_attr(feature = "callbacks", repr(transparent))]
		#[derive(Debug, Clone)]
		pub struct $container(
			#[cfg(feature = "callbacks")] $contents,
			#[cfg(not(feature = "callbacks"))] FeatureNeeded,
		);
		impl Sealed for $container {}
		impl<'a> Sealed for &'a $container {}
		impl $container {
			/// Creates a new [`
			#[doc = $container_str]
			/// `] instance. The `"callbacks"` feature is required to use this function.
			#[cfg_attr(
				not(feature = "callbacks"),
				deprecated = "The `\"callbacks\"` feature is required to use this function."
			)]
			#[inline(always)]
			#[must_use]
			pub fn new(
				#[cfg(feature = "callbacks")] value: $contents,
				#[cfg(not(feature = "callbacks"))] value: FeatureNeeded,
			) -> Self {
				Self(value)
			}
		}
	)?};
}

web_types! {
	/// Erasable stand-in for [`web_sys::Comment`](https://docs.rs/web-sys/0.3/web_sys/struct.Comment.html) used as callback parameter.
	(Comment, "Comment") => web_sys::Comment,

	/// Erasable stand-in for [`web_sys::Element`](https://docs.rs/web-sys/0.3/web_sys/struct.Element.html) used as callback parameter.
	(Element, "Element") => web_sys::Element,

	/// Erasable stand-in for [`web_sys::Event`](https://docs.rs/web-sys/0.3/web_sys/struct.Event.html) used as callback parameter.
	(Event, "Event") => web_sys::Event,

	/// Erasable stand-in for [`web_sys::HtmlElement`](https://docs.rs/web-sys/0.3/web_sys/struct.HtmlElement.html) used as callback parameter.
	(HtmlElement, "HtmlElement") => web_sys::HtmlElement,

	/// Erasable stand-in for [`web_sys::SvgElement`](https://docs.rs/web-sys/0.3/web_sys/struct.SvgElement.html) used as callback parameter.
	(SvgElement, "HtmlElement") => web_sys::SvgElement,

	/// Erasable stand-in for [`web_sys::Text`](https://docs.rs/web-sys/0.3/web_sys/struct.Text.html) used as callback parameter.
	(Text, "Text") => web_sys::Text,
}

macro_rules! conversions {
	{$(
		$container:ty => $contents:ty
	),*$(,)?} => {$(
		#[cfg(feature = "callbacks")]
		impl Materialize<$contents> for $container {
			#[inline(always)] // No-op.
			fn materialize(self) -> $contents {
				self.0
			}
		}

		#[cfg(feature = "callbacks")]
		impl<'a> Materialize<&'a $contents> for &'a $container {
			#[inline(always)] // No-op.
			fn materialize(self) -> &'a $contents {
				unsafe {&*(self as *const $container).cast() }
			}
		}

		#[cfg(not(feature = "callbacks"))]
		impl<AnyType> Materialize<AnyType> for $container {
			#[inline(always)]
			fn materialize(self) -> AnyType {
				unreachable!()
			}
		}

		#[cfg(not(feature = "callbacks"))]
		impl<'a, AnyType> Materialize<&'a AnyType> for &'a $container {
			#[inline(always)]
			fn materialize(self) -> &'a AnyType {
				unreachable!()
			}
		}

		#[cfg(feature = "callbacks")]
		impl From<$contents> for $container {
			#[inline(always)] // No-op.
			fn from(contents: $contents) -> Self {
				Self(contents)
			}
		}

		#[cfg(feature = "callbacks")]
		impl<'a> From<&'a $contents> for &'a $container {
			#[inline(always)] // No-op.
			fn from(contents: &'a $contents) -> Self {
				unsafe {
					&*(contents as *const $contents).cast()
				}
			}
		}
	)*};
}

impl<T: Materialize<U>, U> Materialize<DomRef<U>> for DomRef<T> {
	#[inline(always)]
	fn materialize(self) -> DomRef<U> {
		match self {
			Self::Added(added) => DomRef::Added(added.materialize()),
			Self::Removing(removing) => DomRef::Removing(removing.materialize()),
		}
	}
}

conversions! {
	Comment => web_sys::Comment,
	Event => web_sys::Event,
	HtmlElement => web_sys::HtmlElement,
	SvgElement => web_sys::SvgElement,
	Text => web_sys::Text,
}

/// Empty. Replaces erasable values in this module if the `"callbacks"` feature is not active.
#[doc(hidden)]
#[allow(clippy::empty_enum)]
#[derive(Debug, Clone)]
pub enum FeatureNeeded {}
impl FeatureNeeded {
	#[allow(dead_code)]
	fn map<T, U>(self, _: T) -> Option<U> {
		let _ = self;
		unreachable!()
	}
}

/// Convert a DOM stand-in to its web type value. This is a no-op with the `"callbacks"` feature and unreachable otherwise.
///
/// The extra trait is necessary because `Into` conflicts on `T: From<T>` and `Option<T>: From<T>`.
///
/// **Warning**:
///
/// Without the `"callbacks"` feature, the stand-ins in this module implement [`Materialize`] for any target type!  
/// Make sure to check if your package compiles with this feature enables, most easily by requiring it in the `[dev-dependencies]` section of your *Cargo.toml*.
pub trait Materialize<T: Sized>: Sized + Sealed {
	/// Convert a DOM stand-in to its web type value. This is a no-op with the `"callbacks"` feature and unreachable otherwise.
	fn materialize(self) -> T;
}
