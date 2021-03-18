#![doc(html_root_url = "https://docs.rs/lignin/0.0.5")]
#![no_std]
#![warn(clippy::pedantic)]
#![warn(missing_docs)]

//! `lignin`, named after the structural polymer found in plants, is a lightweight but comprehensive VDOM data type library for use in a wider web context.
//!
//! # About the Documentation
//!
//! DOM API terms are ***bold italic*** and linked to the MDN Web Docs.
//! (Please file an issue if this isn't the case somewhere.)
//!
//! # Implementation Contract
//!
//! > **This is not a soundness contract**. Code using this crate must not rely on it for soundness.
//! > However, it is free to panic when encountering an incorrect implementation.
//!
//! ## Security
//!
//! See also the implementation contracts on [`Node::Text::text`], [`Node::Comment::comment`] and [`Attribute::name`].
//!
//! When rendering the VDOM as HTML text, extra care **must** be taken to syntactically validate everything according to [the specification](https://html.spec.whatwg.org/multipage/syntax.html)!
//!
//! HTML renderers should error rather than panic when encountering a VDOM that they can't guarantee will be parsed as intended (assuming any syntax errors potentially cause undefined behavior *somewhere*).  
//! However, renderers are free to be lenient in this regard by adjusting their output to be syntactically valid in a way that's unlikely to cause a changed user experience. (That is: Feel free to substitute illegal character sequences in comments and such.)
//!
//! ## Correctness
//!
//! The DOM may contain extra siblings past the nodes mentioned in the VDOM. Renderers must ignore them.
//!
//! Similarly, the DOM may contain extra attributes and event bindings. Renderers must ignore them unless attributes collide.  
//! Components must clean up extra attributes and event listeners they have previous added to the DOM via the DOM API on teardown.
//!
//! > This simplifies renderers and allows reuse of DOM nodes between components, which in turn reduces the amount of DOM API calls necessary.
//!
//! See also the implementation contracts on [`DomRef`] and [`Node::Keyed`].
//!
//! ## Performance
//!
//! While the order of [attributes](https://developer.mozilla.org/en-US/docs/Web/API/Element/attributes) reported by the DOM API in browsers isn't specified and event listeners can't be examined this way,
//! components *should* stick to a relatively consistent order here and place conditional attributes and event bindings past always present ones in the respective slices.
//!
//! When adding or removing [`Node`]s dynamically between updates, components should wrap lists in [`Node::Multi`] and otherwise insert an empty [`Node::Multi([&[])`](`Node::Multi`) as placeholder for an absent element.
//!
//! > Each of these suggestions allows better and easier diff optimization in renderers, but otherwise mustn't be a strict requirement for compatibility.
//!
//! # Deep Comparisons
//!
//! All [`core`] comparison traits ([`PartialEq`], [`Eq`], [`PartialOrd`] and [`Ord`]) are implemented recursively where applicable.
//!
//! Note that [`CallbackRef`]s derived from separate instances of [`CallbackRegistration`] are still considered distinct,
//! regardless of the `receiver` and `handler` used to make them.
//! However, without the `"callbacks"` feature, all [`CallbackRef`] instances are inactive **and indistinct**.
//!
//! For shallow comparisons, access and compare fields directly or [memoize](`Node::Memoized`) parts of the GUI.
//!
//! # Notes on Performance
//!
//! ## Clone
//!
//! [`Clone`] is always implemented via [`Copy`] in this crate, since none of the instances provide heap storage.
//!
//! ## Comparisons and Hashing
//!
//! As shallow hashes would easily collide for most applications where VDOM hashing comes up,
//! [`Hash`](`core::hash::Hash`) is implemented recursively in this crate and is potentially expensive.
//! The same applies to [`PartialEq`], [`Eq`], [`PartialOrd`] and [`Ord`].
//!
//! As an exception, [`Node::Memoized`] instances are compared only by their [`state_key`](`Node::Memoized::state_key`).
//! Their [`content`](`Node::Memoized::content`) is ignored for comparisons and does not factor into their [hash](`core::hash`).
//!
//! **`lignin` does not implement hash caching by itself**, so users of [`HashMap`](https://doc.rust-lang.org/stable/std/collections/struct.HashMap.html) or similar containers should wrap node graphs in a "`HashCached<T>`" type first.
//!
//! # Limitations
//!
//! As `lignin` targets HTML and DOM rather than XML, it does not support [***processing instructions***](https://developer.mozilla.org/en-US/docs/Web/API/ProcessingInstruction) or [***CDATA sections***](https://developer.mozilla.org/en-US/docs/Web/API/CDATASection).
//!
//! For the same reason, there is formally no information about VDOM identity, which could be used to render self-referential XML documents.
//!
//! > In practice, it **may** be possible to determine identity by comparing pointers, but this would require some workarounds regarding `lignin`'s slices-of-values to be general.
//! >
//! > The implementation itself would be quite error-prone on types that are [`Copy`] due to implicit by-value copies there. Proceed with caution if you must!
//!
//! Element and attribute names are always plain `&str`s, which isn't ideal for software that renders its GUI more directly than through a web browser.
//! I'm open to maintaining a generic fork if there's interest in this regard.
//!
//! While the `"callbacks"` feature is disabled, all callback management is erased.
//! This makes `lignin` faster and removes usage limits, but removes unique identities from [`CallbackRegistration`] and [`CallbackRef`], which affects comparisons and hashing.
//!
//! [MathML](https://developer.mozilla.org/en-US/docs/Web/MathML) support is rudimentary due lack of direct support in web-sys.
#[cfg(doctest)]
pub mod readme {
	doc_comment::doctest!("../README.md");
}

pub mod auto_safety;
pub mod callback_registry;
mod remnants;
pub mod web;

use callback_registry::CallbackSignature;
pub use callback_registry::{CallbackRef, CallbackRegistration};
pub use web::{DomRef, Materialize};

mod ergonomics;

use core::{convert::Infallible, fmt::Debug, hash::Hash, marker::PhantomData};
use remnants::RemnantSite;
use sealed::Sealed;

/// [`Vdom`] A single generic VDOM node.
///
/// This should be relatively small:
///
/// ```rust
/// # use core::mem::size_of;
/// # use lignin::{Node, ThreadSafe};
/// if size_of::<usize>() == 8 {
///   assert!(size_of::<Node<ThreadSafe>>() <= 24);
/// }
///
/// // e.g. current Wasm
/// if size_of::<usize>() == 4 {
///   assert!(size_of::<Node<ThreadSafe>>() <= 16);
/// }
/// ```
#[allow(clippy::clippy::type_complexity)] // `Option<CallbackRef<S, fn(DomRef<&'_ …>)>>` appears to be a little much.
pub enum Node<'a, S: ThreadSafety> {
	/// Represents a [***Comment***](https://developer.mozilla.org/en-US/docs/Web/API/Comment) node.
	Comment {
		/// The comment's body, as unescaped plaintext.
		///
		/// Renderers shouldn't insert padding whitespace around it, except as required by e.g. pretty-printing.
		///
		/// # Implementation Contract
		///
		/// > **This is not a soundness contract**. Code using this crate must not rely on it for soundness.
		/// > However, it is free to panic when encountering an incorrect implementation.
		///
		/// ## **Security**
		///
		/// This field may contain arbitrary character sequences, some of which are illegal in [***Comment***](https://developer.mozilla.org/en-US/docs/Web/API/Comment)s at least when serialized as HTML.
		/// See <https://html.spec.whatwg.org/multipage/syntax.html#comments> for more information.
		///
		/// Renderers **must** either refuse or replace illegal-for-target comments with ones that are inert.
		///
		/// Not doing so opens the door for [XSS](https://developer.mozilla.org/en-US/docs/Glossary/Cross-site_scripting)
		/// and/or format confusion vulnerabilities.
		comment: &'a str,
		/// Registers for [***Comment***](https://developer.mozilla.org/en-US/docs/Web/API/Comment) reference updates.
		///
		/// See [`DomRef`] for more information.
		dom_binding: Option<CallbackRef<S, fn(dom_ref: DomRef<&'_ web::Comment>)>>,
	},
	/// Represents a single [***HTMLElement***](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement).
	HtmlElement {
		/// The [`Element`] to render.
		element: &'a Element<'a, S>,
		/// Registers for [***HTMLElement***](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement) reference updates.
		///
		/// See [`DomRef`] for more information.
		dom_binding: Option<CallbackRef<S, fn(dom_ref: DomRef<&'_ web::HtmlElement>)>>,
	},
	/// Represents a single [***MathMLElement***](https://developer.mozilla.org/en-US/docs/Web/API/MathMLElement).
	///
	/// Note that [distinct browser support for these is really quite bad](https://developer.mozilla.org/en-US/docs/Web/API/MathMLElement#browser_compatibility)
	/// and [correct styling isn't much more available](https://developer.mozilla.org/en-US/docs/Web/MathML#browser_compatibility).
	///
	/// However, [MathML *is* part of the HTML standard](https://html.spec.whatwg.org/multipage/embedded-content-other.html#mathml), so browsers should at least parse it correctly, and styling can be polyfilled.
	MathMlElement {
		/// The [`Element`] to render.
		element: &'a Element<'a, S>,
		/// Registers for [***Element***](https://developer.mozilla.org/en-US/docs/Web/API/Element) reference updates.
		///
		/// See [`DomRef`] for more information.
		dom_binding: Option<CallbackRef<S, fn(dom_ref: DomRef<&'_ web::Element>)>>,
	},
	/// Represents a single [***SVGElement***](https://developer.mozilla.org/en-US/docs/Web/API/SVGElement).
	///
	/// Note that even outermost `<SVG>` elements are [***SVGElement***](https://developer.mozilla.org/en-US/docs/Web/API/SVGElement)s!
	SvgElement {
		/// The [`Element`] to render.
		element: &'a Element<'a, S>,
		/// Registers for [***SVGElement***](https://developer.mozilla.org/en-US/docs/Web/API/SVGElement) reference updates.
		///
		/// See [`DomRef`] for more information.
		dom_binding: Option<CallbackRef<S, fn(dom_ref: DomRef<&'_ web::SvgElement>)>>,
	},
	/// DOM-transparent. This variant uses shallow comparison and hashes based on its `state_key` only.
	///
	/// A (good enough) `content` [hash](`core::hash`) makes for a good `state_key`, but this isn't the only possible scheme and may not be the optimal one for your use case.
	///
	/// # Implementation Contract (reminder)
	///
	/// Note that when diffing a non-[`Memoized`](`Node::Memoized`) [`Node`] into a [`Node::Memoized`] (and vice-versa), renderers must still behave as if the DOM tree was recreated, which means cycling all [***Node***](https://developer.mozilla.org/en-US/docs/Web/API/Node) reference bindings even if they match.
	///
	/// > However, this often happens with matching or near-matching fragments during hydration of a web app.
	/// >
	/// > *If you already have a function to strip subscriptions* (e.g. [***Node***](https://developer.mozilla.org/en-US/docs/Web/API/Node) reference bindings) from a DOM and VDOM tree,
	/// > or even just one to strip all callbacks (but this is less efficient), it's likely more efficient to do so and then recurse.
	/// >
	/// > Make sure the trees are actually somewhat compatible first, or you may end up processing the old VDOM twice for nothing.
	Memoized {
		/// A value that's (very likely to be) distinct between VDOM graphs where the path of two [`Node::Memoized`] instances matches but their [`Node::Memoized::content`] is distinct.
		///
		/// Consider using a (good enough) hash of [`content`](`Node::Memoized::content`) for this purpose.
		state_key: u64,
		/// The VDOM tree memoized by this [`Node`].
		content: &'a Node<'a, S>,
	},
	/// DOM-transparent. Represents a sequence of VDOM nodes.
	///
	/// Used to hint diffs in case of additions and removals.
	Multi(&'a [Node<'a, S>]),
	/// A sequence of VDOM nodes that's transparent at rest, but encodes information on how to reuse and reorder elements when diffing.
	///
	/// **List indices are bad [`ReorderableFragment::dom_key`] values** unless reordered along with the items!
	/// Use the [`Multi`](`Node::Multi`) variant instead if you don't track component identity.
	///
	/// # Implementation Contract
	///
	/// > **This is not a soundness contract**. Code using this crate must not rely on it for soundness.
	/// > However, it is free to panic when encountering an incorrect implementation.
	///
	/// The [`ReorderableFragment::dom_key`] values must be unique within a slice referenced by a [`Node::Keyed`] instance.
	///
	///
	/// If a [`dom_key`](`ReorderableFragment::dom_key`) value appears both in the initial and target slice of a [`ReorderableFragment::dom_key`] diff,
	/// those [`ReorderableFragment`] instances are considered path-matching and any respective [***Node***](https://developer.mozilla.org/en-US/docs/Web/API/Node)(s!) **must**
	/// be moved to their new location without being recreated.
	///
	/// > These rules do not apply between distinct [`ReorderableFragment`] slices, even if they overlap in memory or one is reachable from the other.
	///
	/// > The recursive diff otherwise proceeds as normal. There are no rules on whether it happens before or after the reordering.
	Keyed(&'a [ReorderableFragment<'a, S>]),
	/// Represents a [***Text***](https://developer.mozilla.org/en-US/docs/Web/API/Text) node.
	Text {
		/// The [`Text`](`Node::Text`)'s [***Node.textContent***](https://developer.mozilla.org/en-US/docs/Web/API/Node/textContent).
		///
		/// # Implementation Contract
		///
		/// > **This is not a soundness contract**. Code using this crate must not rely on it for soundness.
		/// > However, it is free to panic when encountering an incorrect implementation.
		///
		/// ## **Security**
		///
		/// This field contains unescaped *plaintext*. Renderers **must** escape **all** control characters and sequences.
		///
		/// Not doing so opens the door for [XSS](https://developer.mozilla.org/en-US/docs/Glossary/Cross-site_scripting) vulnerabilities.
		///
		/// In order to support e.g. formatting instructions, apps should (carefully) parse user-generated content and translate it into a matching VDOM graph.
		///
		/// Live components also have the option of using for example [`Node::HtmlElement::dom_binding`] to set [***Element.innerHTML***](https://developer.mozilla.org/en-US/docs/Web/API/Element/innerHTML),
		/// but this is not recommended due to the difficulty of implementing allow-listing with such an approach.
		text: &'a str,
		/// Registers for [***Text***](https://developer.mozilla.org/en-US/docs/Web/API/Text) reference updates.
		///
		/// See [`DomRef`] for more information.
		dom_binding: Option<CallbackRef<S, fn(dom_ref: DomRef<&'_ web::Text>)>>,
	},
	/// Currently unused.
	///
	/// The plan here is to allow fragments to linger in the DOM after being diffed out, which seems like the most economical way to enable e.g. fade-out animations.
	//[not `doc`] There should be a callback for this occasion, and they should be placed in such a way in the DOM that, by default, they are rendered *in front* of a replacement in the same location.
	RemnantSite(&'a RemnantSite),
}

/// [`Vdom`] A VDOM node that has its DOM identity preserved during DOM updates even after being repositioned within a (path-)matching [`Node::Keyed`].
///
/// For more information, see [`Node::Keyed`].
pub struct ReorderableFragment<'a, S: ThreadSafety> {
	/// A key uniquely identifying a [`ReorderableFragment`] within any directly spanning [`Node::Keyed`].
	pub dom_key: usize,
	/// The [`Node`] to render from this [`ReorderableFragment`].
	pub content: Node<'a, S>,
}

#[allow(clippy::doc_markdown)]
/// [`Vdom`] Represents a single [***HTMLElement***](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement) as `name`, `attributes`, `content` and `event_bindings`.
pub struct Element<'a, S: ThreadSafety> {
	/// The [***Element.tag_name***](https://developer.mozilla.org/en-US/docs/Web/API/Element/tagName).
	///
	/// Unlike in the browser, this is generally treated case-*sensitively*, meaning for example `"div"` doesn't equal `"DIV"`.
	///
	/// Since browsers will generally return the canonical uppercase name, it's recommended to generate the VDOM all-uppercase too, to avoid unnecessary mismatches.
	pub name: &'a str,
	/// Controls the ***options*** parameter of [***Document.createElement()***](https://developer.mozilla.org/en-US/docs/Web/API/Document/createElement)
	/// *or* (currently only) the global [***is***](https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/is) attribute.
	pub creation_options: ElementCreationOptions<'a>,
	/// The [***Element.attributes***](https://developer.mozilla.org/en-US/docs/Web/API/Element/attributes).
	///
	/// Note that while this collection is unordered in the browser, reordering attributes will generally affect diffing performance.
	pub attributes: &'a [Attribute<'a>],
	/// Maps to [***Node.childNodes***](https://developer.mozilla.org/en-US/docs/Web/API/Node/childNodes).
	pub content: Node<'a, S>,
	/// DOM event bindings requested by a component.
	///
	/// See [`EventBinding`] for more information.
	pub event_bindings: &'a [EventBinding<'a, S>],
}

/// [`Vdom`] Maps to ***options*** parameter values of [***Document.createElement()***](https://developer.mozilla.org/en-US/docs/Web/API/Document/createElement)
/// (including ***undefined***) *or* (currently only) the global [***is***](https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/is) attribute.
///
/// # Options
///
/// ## `is`
///
/// The ***tag name*** of a previously [defined](https://developer.mozilla.org/en-US/docs/Web/API/CustomElementRegistry/define)
/// [***customized built-in element***](https://developer.mozilla.org/en-US/docs/Web/Web_Components/Using_custom_elements#customized_built-in_elements)
/// to instantiate over a built-in HTML element.
///
/// When rendering HTML, this controls the global [***is***](https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/is) attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ElementCreationOptions<'a> {
	is: Option<&'a str>,
}
impl<'a> Default for ElementCreationOptions<'a> {
	fn default() -> Self {
		Self::new()
	}
}
#[allow(clippy::inline_always)] // Trivial getters and setters.
impl<'a> ElementCreationOptions<'a> {
	/// Creates a new [`ElementCreationOptions`] with all fields set to [`None`].
	#[inline(always)]
	#[must_use]
	pub const fn new() -> Self {
		Self { is: None }
	}

	/// Indicates whether this [`ElementCreationOptions`] instance can be omitted entirely in a [***Document.createElement()***](https://developer.mozilla.org/en-US/docs/Web/API/Document/createElement)
	/// call.
	#[inline(always)]
	#[must_use]
	pub const fn matches_undefined(&self) -> bool {
		matches!(self, Self { is: None })
	}

	/// Retrieves the ***tag name*** of a previously [defined](https://developer.mozilla.org/en-US/docs/Web/API/CustomElementRegistry/define)
	/// [***customized built-in element***](https://developer.mozilla.org/en-US/docs/Web/Web_Components/Using_custom_elements#customized_built-in_elements)
	/// to use.
	#[inline(always)]
	#[must_use]
	pub const fn is(&self) -> Option<&'a str> {
		self.is
	}
	/// Sets the ***tag name*** of a previously [defined](https://developer.mozilla.org/en-US/docs/Web/API/CustomElementRegistry/define)
	/// [***customized built-in element***](https://developer.mozilla.org/en-US/docs/Web/Web_Components/Using_custom_elements#customized_built-in_elements)
	/// to use.
	#[inline(always)]
	pub fn set_is(&mut self, is: Option<&'a str>) {
		self.is = is
	}
	/// Sets the ***tag name*** of a previously [defined](https://developer.mozilla.org/en-US/docs/Web/API/CustomElementRegistry/define)
	/// [***customized built-in element***](https://developer.mozilla.org/en-US/docs/Web/Web_Components/Using_custom_elements#customized_built-in_elements)
	/// to use.
	#[inline(always)]
	#[must_use]
	pub const fn with_is(self, is: Option<&'a str>) -> Self {
		#[allow(clippy::needless_update)]
		Self { is, ..self }
	}
}

/// [`Vdom`] Represents a single DOM event binding with `name` and `callback`.
///
#[allow(clippy::doc_markdown)]
/// Renderers usually should either manage these through [***EventTarget.addEventListener***](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget/addEventListener)/[***….removeEventListener***](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget/removeEventListener)
/// or ignore them entirely. See [`web`] for a bit more information on that.
///
/// Note that the running total of [`CallbackRegistration`]s made can be limited to [`u32::MAX`] or around four billion.
/// (See [`callback_registry`] for information on how to get around this, if necessary.)
///
/// While this limit is likely hard to hit, economizing registrations a little will still (indirectly) improve app performance.
/// Lazily registering callbacks for events only when rendering is also the easiest way for framework developers to use [pinning](core::pin) to avoid heap allocations.
pub struct EventBinding<'a, S: ThreadSafety> {
	/// The event name.
	pub name: &'a str,
	/// A callback reference created via [`CallbackRegistration`].
	pub callback: CallbackRef<S, fn(event: web::Event)>,
	/// Controls the ***options*** parameter of [***EventTarget.addEventListener()***](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget/addEventListener).
	///
	/// Note that [`EventBindingOptions`] is created with the [`EventBindingOptions.passive()`] flag already enabled!
	pub options: EventBindingOptions,
}

/// [`Vdom`] Maps to ***options*** parameter values of [***EventTarget.addEventListener()***](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget/addEventListener).
///
/// Note that all constructors initialize instances with [`.passive()`](`EventBindingOptions::passive()`) set to true.
///
/// Also note that these flags aren't part of any soundness contract! Don't rely on them for memory safety.
///
/// # Flags
///
/// ## `capture`
///
/// Controls whether a [`web::Event`] should be dispatched while bubbling down rather than up along the DOM.
///
/// ## `once`
///
/// Controls whether an associated [`CallbackRef`] should be invoked at most once for this [`EventBinding`].
///
/// This carries over for as long as the [`EventBinding`]'s VDOM identity doesn't change.
///
/// ## `passive` (default)
///
/// Controls whether a callback is disallowed from calling [`web_sys::Event::prevent_default()`](https://docs.rs/web-sys/0.3.48/web_sys/struct.Event.html#method.prevent_default).
///
/// Calling that method while this flag is enabled shouldn't produce any effects other than printing a warning to a browser's JavaScript console.
///
/// [This flag can significantly improve performance when applied to certain events.](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget/addEventListener#improving_scrolling_performance_with_passive_listeners)
///
/// > ***passive: true*** isn't always the default in web browsers for backwards compatibility reasons.
/// >
/// > As `lignin` is a new framework, it's able to break with that tradition for more consistency and a better default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventBindingOptions(u8);
mod event_bindings_impl {
	#![allow(clippy::inline_always)] // Trivial bit manipulation.
	#![allow(clippy::trivially_copy_pass_by_ref)] // Erased by inlining.

	#[allow(unused_imports)] // Largely for documentation.
	use crate::{web, CallbackRef, EventBinding, EventBindingOptions};

	pub const CAPTURE: u8 = 0b_0001;
	pub const ONCE: u8 = 0b_0010;
	pub const PASSIVE: u8 = 0b_0100;

	impl Default for EventBindingOptions {
		/// Creates a new [`EventBindingOptions`] instance with [`.passive()`] already set to `true`. [See more.](`Default::default`)
		#[inline(always)]
		fn default() -> Self {
			Self::new()
		}
	}

	#[allow(clippy::match_bool)]
	impl EventBindingOptions {
		/// Creates a new [`EventBindingOptions`] instance with [`.passive()`] already set to `true`.
		#[inline(always)]
		#[must_use]
		pub const fn new() -> Self {
			Self(PASSIVE)
		}

		/// Indicates whether a [`web::Event`] should be dispatched while bubbling down rather than up along the DOM.
		#[inline(always)]
		#[must_use]
		pub const fn capture(&self) -> bool {
			self.0 & CAPTURE == CAPTURE
		}
		/// Sets whether a [`web::Event`] should be dispatched while bubbling down rather than up along the DOM.
		#[inline(always)]
		pub fn set_capture(&mut self, capture: bool) {
			*self = self.with_capture(capture)
		}
		/// Sets whether a [`web::Event`] should be dispatched while bubbling down rather than up along the DOM.
		#[inline(always)]
		#[must_use]
		pub const fn with_capture(self, capture: bool) -> Self {
			Self(match capture {
				true => self.0 | CAPTURE,
				false => self.0 & !CAPTURE,
			})
		}

		/// Indicates whether an associated [`CallbackRef`] should be invoked at most once for this [`EventBinding`]. [See more.](#once)
		#[inline(always)]
		#[must_use]
		pub const fn once(&self) -> bool {
			self.0 & ONCE == ONCE
		}
		/// Sets whether an associated [`CallbackRef`] should be invoked at most once for this [`EventBinding`]. [See more.](#once)
		#[inline(always)]
		pub fn set_once(&mut self, once: bool) {
			*self = self.with_once(once)
		}
		/// Sets whether an associated [`CallbackRef`] should be invoked at most once for this [`EventBinding`]. [See more.](#once)
		#[inline(always)]
		#[must_use]
		pub const fn with_once(self, once: bool) -> Self {
			Self(match once {
				true => self.0 | ONCE,
				false => self.0 & !ONCE,
			})
		}

		/// `(default)` Indicates whether a callback is disallowed from calling [`web_sys::Event::prevent_default()`](https://docs.rs/web-sys/0.3.48/web_sys/struct.Event.html#method.prevent_default).
		/// [See more.](#passive)
		#[inline(always)]
		#[must_use]
		pub const fn passive(&self) -> bool {
			self.0 & PASSIVE == PASSIVE
		}
		/// `(default)` Sets whether a callback is disallowed from calling [`web_sys::Event::prevent_default()`](https://docs.rs/web-sys/0.3.48/web_sys/struct.Event.html#method.prevent_default).
		/// [See more.](#passive)
		#[inline(always)]
		pub fn set_passive(&mut self, passive: bool) {
			*self = self.with_passive(passive)
		}
		/// `(default)` Sets whether a callback is disallowed from calling [`web_sys::Event::prevent_default()`](https://docs.rs/web-sys/0.3.48/web_sys/struct.Event.html#method.prevent_default).
		/// [See more.](#passive)
		#[inline(always)]
		#[must_use]
		pub const fn with_passive(self, passive: bool) -> Self {
			Self(match passive {
				true => self.0 | PASSIVE,
				false => self.0 & !PASSIVE,
			})
		}
	}
}

/// [`Vdom`] Represents a single HTML [***Attr***](https://developer.mozilla.org/en-US/docs/Web/API/Attr) with `name` and `value`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct Attribute<'a> {
	/// The [***name***](https://developer.mozilla.org/en-US/docs/Web/API/Attr#properties).
	///
	/// # Implementation Contract
	///
	/// ## Security
	///
	/// While applications should generally avoid it, [`Attribute::name`] may contain [characters that are unexpected in this position](https://html.spec.whatwg.org/multipage/syntax.html#attributes-2).
	///
	/// Renderers may only process these verbatim iff they can expect this to not cause security issues.
	///
	/// > For example: Passing an invalid attribute name to a DOM API *isolated in a dedicated parameter* is *probably* okay,
	/// > as long as something along the way validates it doesn't contain `'\0'`.
	/// >
	/// > Serializing an invalid attribute name to HTML is a **very** bad idea, so renderers must never do so.
	pub name: &'a str,
	/// The unescaped [***value***](https://developer.mozilla.org/en-US/docs/Web/API/Attr#properties).
	pub value: &'a str,
}

mod sealed {
	use super::{ThreadBound, ThreadSafe};
	use crate::{
		callback_registry::CallbackSignature, remnants::RemnantSite, web, Attribute, CallbackRef,
		CallbackRegistration, DomRef, Element, ElementCreationOptions, EventBinding,
		EventBindingOptions, Node, ReorderableFragment, ThreadSafety,
	};

	pub trait Sealed {}
	impl Sealed for fn(web::Event) {}
	impl<T> Sealed for fn(DomRef<&'_ T>) {}
	impl Sealed for ThreadBound {}
	impl Sealed for ThreadSafe {}
	impl<'a> Sealed for Attribute<'a> {}
	impl<'a> Sealed for ElementCreationOptions<'a> {}
	impl Sealed for EventBindingOptions {}
	impl<R, C: CallbackSignature> Sealed for CallbackRegistration<R, C> {}
	impl<S: ThreadSafety, C: CallbackSignature> Sealed for CallbackRef<S, C> {}
	impl<'a, S: ThreadSafety> Sealed for Element<'a, S> {}
	impl<'a, S: ThreadSafety> Sealed for EventBinding<'a, S> {}
	impl<'a, S: ThreadSafety> Sealed for Node<'a, S> {}
	impl<'a, S: ThreadSafety> Sealed for ReorderableFragment<'a, S> {}
	impl Sealed for RemnantSite {}
}

/// Marker trait for thread-safety tokens.
pub trait ThreadSafety: Sealed + Into<ThreadBound>
where
	Self: Sized + Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash,
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
/// Somewhat uselessly implemented on [`Attribute`], [`ElementCreationOptions`] and [`EventBindingOptions`], which are always [`ThreadSafe`].
pub trait Vdom: Sealed
where
	Self: Sized + Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash,
{
	/// The [`ThreadSafety`] of the [`Vdom`] type, either [`ThreadSafe`] or [`ThreadBound`].
	///
	/// This comes from a generic type argument `S`, but [`Attribute`] and [`EventBindingOptions`] are always [`ThreadSafe`].
	type ThreadSafety: ThreadSafety;
}

impl<'a> Vdom for Attribute<'a> {
	type ThreadSafety = ThreadSafe;
}

impl<'a> Vdom for ElementCreationOptions<'a> {
	type ThreadSafety = ThreadSafe;
}

impl Vdom for EventBindingOptions {
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
vdom_impls!(Element, EventBinding, Node, ReorderableFragment);

impl<S, C> Vdom for CallbackRef<S, C>
where
	S: ThreadSafety,
	C: CallbackSignature,
{
	type ThreadSafety = S;
}
