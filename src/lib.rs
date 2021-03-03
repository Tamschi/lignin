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
//! See the implementation contract on [`Node::Text::text`].
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
//! > This both together allows better and easier diff optimization in renderers, but otherwise mustn't be a strict requirement for compatibility.
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
//! ## Hashing
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
//! As `lignin` targets HTML and DOM rather than XML, it does not support [processing instructions](https://developer.mozilla.org/en-US/docs/Web/API/ProcessingInstruction).
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
pub enum Node<'a, S: ThreadSafety> {
	/// Represents a [***Comment***](https://developer.mozilla.org/en-US/docs/Web/API/Comment) node.
	Comment {
		/// The comment's body, as unescaped plaintext.
		///
		/// Renderers shouldn't insert padding whitespace around it, except as required by e.g. pretty-printing.
		///
		///TODO: Forbidden character sequences.
		comment: &'a str,
		/// Registers for [***Comment***](https://developer.mozilla.org/en-US/docs/Web/API/Comment) reference updates.
		///
		/// See [`DomRef`] for more information.
		dom_binding: Option<CallbackRef<S, DomRef<web::Comment>>>,
	},
	/// Represents a single [***HTMLElement***](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement).
	Element {
		/// The [`Element`] to render.
		element: &'a Element<'a, S>,
		/// Registers for [***HTMLElement***](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement) reference updates.
		///
		/// See [`DomRef`] for more information.
		dom_binding: Option<CallbackRef<S, DomRef<web::HtmlElement>>>,
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
		/// Live components also have the option of using for example [`Node::Element::dom_binding`] to set [***Element.innerHTML***](https://developer.mozilla.org/en-US/docs/Web/API/Element/innerHTML),
		/// but this is not recommended due to the difficulty of implementing allow-listing with such an approach.
		text: &'a str,
		/// Registers for [***Text***](https://developer.mozilla.org/en-US/docs/Web/API/Text) reference updates.
		///
		/// See [`DomRef`] for more information.
		dom_binding: Option<CallbackRef<S, DomRef<web::Text>>>,
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
	/// Since browsers will generally return the canonical uppercase name, it's recommended to generate the VDOM this way also, to avoid unnecessary mismatches.
	pub name: &'a str,
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

/// [`Vdom`] Represents a single DOM event binding with `name` and `callback`.
///
#[allow(clippy::doc_markdown)]
/// Renderers usually should either manage these through [***EventTarget.addEventListener***](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget/addEventListener)/[***….removeEventListener***](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget/removeEventListener)
/// or ignore them entirely. See [`web`] for a bit more information on that.
///
/// Note that the running total of [`CallbackRegistration`]s made can be limited to [`u32::MAX`] or around four billion.
/// (See [`callback_registry`] for information on how to get around this, if necessary.)
///
/// While this limit is likely hard to hit, economising registrations a little will still (indirectly) improve app performance.
/// Lazily registering callbacks for events only when rendering is also the easiest way for framework developers to use [pinning](core::pin) to avoid heap allocations.
pub struct EventBinding<'a, S: ThreadSafety> {
	/// The event name.
	pub name: &'a str,
	/// A callback reference created via [`CallbackRegistration`].
	pub callback: CallbackRef<S, web::Event>,
}

/// [`Vdom`] Represents a single HTML [***Attr***](https://developer.mozilla.org/en-US/docs/Web/API/Attr) with `name` and `value`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct Attribute<'a> {
	/// The unescaped [***name***](https://developer.mozilla.org/en-US/docs/Web/API/Attr#properties).
	///
	///TODO?: Forbidden characters.
	pub name: &'a str,
	/// The unescaped [***value***](https://developer.mozilla.org/en-US/docs/Web/API/Attr#properties).
	pub value: &'a str,
}

mod sealed {
	use super::{ThreadBound, ThreadSafe};
	use crate::{
		remnants::RemnantSite, Attribute, CallbackRef, CallbackRegistration, Element, EventBinding,
		Node, ReorderableFragment, ThreadSafety,
	};

	pub trait Sealed {}
	impl Sealed for ThreadBound {}
	impl Sealed for ThreadSafe {}
	impl<'a> Sealed for Attribute<'a> {}
	impl<R, T> Sealed for CallbackRegistration<R, T> {}
	impl<S: ThreadSafety, T> Sealed for CallbackRef<S, T> {}
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
/// Somewhat uselessly implemented on [`Attribute`], which is always [`ThreadSafe`].
pub trait Vdom: Sealed
where
	Self: Sized + Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash,
{
	/// The [`ThreadSafety`] of the [`Vdom`] type, either [`ThreadSafe`] or [`ThreadBound`].
	///
	/// This comes from a generic type argument `S`, but [`Attribute`] is always [`ThreadSafe`].
	type ThreadSafety: ThreadSafety;
}

impl<'a> Vdom for Attribute<'a> {
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

impl<S: ThreadSafety, T> Vdom for CallbackRef<S, T> {
	type ThreadSafety = S;
}
