//! Erasable web type stand-ins used as callback parameters.
#![allow(clippy::inline_always)]

macro_rules! web_types {
	{$(
		$(#[$($attrs:tt)*])*
		($container:ident$(<$($generics:ident),*$(,)?>)?, $container_str:literal) => $contents:ty
	),*$(,)?} => {$(
		// It's unfortunately not possible to puzzle the first line together like below, since it ends up cut off in the overview.
		$(#[$($attrs)*])*
		///
		/// Use [`Materialize::materialize`] to convert it to the actual value.
		#[cfg_attr(feature = "callbacks", repr(transparent))]
		#[derive(Debug, Clone)]
		pub struct $container$(<$($generics),*>)?(
			#[cfg(feature = "callbacks")] $contents,
			#[cfg(not(feature = "callbacks"))] FeatureNeeded,
			$(#[cfg(not(feature = "callbacks"))] core::marker::PhantomData::<($($generics,)*)>,)?
		);
		impl$(<$($generics),*>)? $container$(<$($generics),*>)? {
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
				Self(value, $(#[cfg(not(feature = "callbacks"))] core::marker::PhantomData::<($($generics,)*)>)?)
			}
		}
	)?};
}

web_types! {
	/// Erasable stand-in for [`Option<T>`](`Option`) used as callback parameter.
	///
	/// This type is used instead of [`Option<T>`] to also make the [`None`] variant erasable.
	(DomRef<T>, "DomRef<T>") => Option<T>,

	/// Erasable stand-in for [`web_sys::Comment`](https://docs.rs/web-sys/0.3/web_sys/struct.Comment.html) used as callback parameter.
	(Comment, "Comment") => web_sys::Comment,

	/// Erasable stand-in for [`web_sys::Event`](https://docs.rs/web-sys/0.3/web_sys/struct.Event.html) used as callback parameter.
	(Event, "Event") => web_sys::Event,

	/// Erasable stand-in for [`web_sys::HtmlElement`](https://docs.rs/web-sys/0.3/web_sys/struct.HtmlElement.html) used as callback parameter.
	(HtmlElement, "HtmlElement") => web_sys::HtmlElement,

	/// Erasable stand-in for [`web_sys::Text`](https://docs.rs/web-sys/0.3/web_sys/struct.Text.html) used as callback parameter.
	(Text, "Text") => web_sys::Text,
}

macro_rules! conversions {
	{$(
		$container:ty => $contents:ty
	),*$(,)?} => {$(
		#[cfg(feature = "callbacks")]
		impl Materialize<$contents> for $container {
			fn materialize(self) -> $contents {
				self.0
			}
		}

		#[cfg(not(feature = "callbacks"))]
		impl<AnyType> Materialize<AnyType> for $container {
			fn materialize(self) -> AnyType {
				unreachable!()
			}
		}
	)*};
}

impl<T: Materialize<U>, U> Materialize<Option<U>> for DomRef<T> {
	#[inline(always)]
	fn materialize(self) -> Option<U> {
		self.0.map(<T as Materialize<U>>::materialize)
	}
}

conversions! {
	Comment => web_sys::Comment,
	Event => web_sys::Event,
	HtmlElement => web_sys::HtmlElement,
	Text => web_sys::Text,
}

/// Replaces erasable values in this module if the `"callbacks"` feature is not active.
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
pub trait Materialize<T> {
	/// Convert a DOM stand-in to its web type value. This is a no-op with the `"callbacks"` feature and unreachable otherwise.
	fn materialize(self) -> T;
}
