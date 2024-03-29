//! This module is private but contains various convenience implementations not used by the rest of the library that may be useful to consumers of this crate.
#![allow(clippy::match_same_arms)]

use crate::{
	auto_safety::Align, callback_registry::CallbackSignature, CallbackRef, CallbackRegistration,
	Element, EventBinding, EventBindingOptions, Node, ReorderableFragment, ThreadBound, ThreadSafe,
	ThreadSafety,
};
use core::{
	any::type_name,
	cmp::{self, Ordering},
	fmt::{self, Debug, Formatter},
	hash::{Hash, Hasher},
	matches,
};

impl From<ThreadSafe> for ThreadBound {
	/// Unreachable. Available as compatibility marker when handling generic [`ThreadSafety`] directly.
	fn from(_: ThreadSafe) -> Self {
		unreachable!()
	}
}

impl<C> From<CallbackRef<ThreadSafe, C>> for CallbackRef<ThreadBound, C>
where
	C: CallbackSignature,
{
	#[allow(clippy::inline_always)]
	#[inline(always)] // No-op.
	fn from(thread_safe: CallbackRef<ThreadSafe, C>) -> Self {
		thread_safe.align()
	}
}

impl<S, C> Debug for CallbackRef<S, C>
where
	S: ThreadSafety,
	C: CallbackSignature,
{
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_struct(type_name::<Self>())
			.field("key", &self.key)
			.finish()
	}
}

#[allow(clippy::expl_impl_clone_on_copy)]
impl<S, C> Clone for CallbackRef<S, C>
where
	S: ThreadSafety,
	C: CallbackSignature,
{
	fn clone(&self) -> Self {
		*self
	}
}
impl<S, C> Copy for CallbackRef<S, C>
where
	S: ThreadSafety,
	C: CallbackSignature,
{
}
impl<S1, S2, C> PartialEq<CallbackRef<S2, C>> for CallbackRef<S1, C>
where
	S1: ThreadSafety,
	S2: ThreadSafety,
	C: CallbackSignature,
{
	fn eq(&self, other: &CallbackRef<S2, C>) -> bool {
		self.key == other.key
	}
}
impl<S, C> Eq for CallbackRef<S, C>
where
	S: ThreadSafety,
	C: CallbackSignature,
{
}
impl<S, C> Hash for CallbackRef<S, C>
where
	S: ThreadSafety,
	C: CallbackSignature,
{
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.key.hash(state)
	}
}
impl<S1, S2, C> PartialOrd<CallbackRef<S2, C>> for CallbackRef<S1, C>
where
	S1: ThreadSafety,
	S2: ThreadSafety,
	C: CallbackSignature,
{
	fn partial_cmp(&self, other: &CallbackRef<S2, C>) -> Option<Ordering> {
		self.key.partial_cmp(&other.key)
	}
}
impl<S, C> Ord for CallbackRef<S, C>
where
	S: ThreadSafety,
	C: CallbackSignature,
{
	fn cmp(&self, other: &Self) -> Ordering {
		self.key.cmp(&other.key)
	}
}

impl<R, C> From<&CallbackRegistration<R, C>> for CallbackRef<ThreadSafe, C>
where
	R: Sync,
	C: CallbackSignature,
{
	fn from(registration: &CallbackRegistration<R, C>) -> Self {
		registration.to_ref()
	}
}

impl<R, C> From<&CallbackRegistration<R, C>> for CallbackRef<ThreadBound, C>
where
	C: CallbackSignature,
{
	fn from(registration: &CallbackRegistration<R, C>) -> Self {
		registration.to_ref_thread_bound()
	}
}

macro_rules! vdom_ergonomics {
	([$(
		$VdomName:ident {
			debug: |&$debug_self:ident, $debug_f:ident| $debug:expr,
			partial_eq: |&$eq_self:ident, $eq_other:ident| $partial_eq:expr,
			hash: |&$hash_self:ident, $hash_state:ident| $hash:expr,
			cmp: |&$cmp_self:ident, $cmp_other:ident| $cmp:expr,
		}
	),*$(,)?]) => {$(
		impl<'a> From<$VdomName<'a, ThreadSafe>> for $VdomName<'a, ThreadBound> {
			#[allow(clippy::inline_always)]
			#[inline(always)] // No-op.
			fn from(thread_safe: $VdomName<'a, ThreadSafe>) -> Self {
				thread_safe.align()
			}
		}

		impl<'a, S> Debug for $VdomName<'a, S> where
			S: ThreadSafety,
		{
			fn fmt(&$debug_self, $debug_f: &mut Formatter<'_>) -> fmt::Result {
				$debug
			}
		}

		#[allow(clippy::expl_impl_clone_on_copy)]
		impl<'a, S> Clone for $VdomName<'a, S> where
			S: ThreadSafety,
		{
			fn clone(&self) -> Self {
				*self
			}
		}
		impl<'a, S> Copy for $VdomName<'a, S> where
			S: ThreadSafety,
		{}

		impl<'a, S1, S2> PartialEq<$VdomName<'a, S2>> for $VdomName<'a, S1> where
			S1: ThreadSafety,
			S2: ThreadSafety,
		{
			fn eq(&$eq_self, $eq_other: &$VdomName<'a, S2>) -> bool {
				$partial_eq
			}
		}
		impl<'a, S> Eq for $VdomName<'a, S> where
			S: ThreadSafety,
		{}

		impl<'a, S> Hash for $VdomName<'a, S> where
			S: ThreadSafety,
		{
			fn hash<H: Hasher>(&$hash_self, $hash_state: &mut H) {
				$hash
			}
		}

		impl<'a, S1, S2> PartialOrd<$VdomName<'a, S2>> for $VdomName<'a, S1>
		where
			S1: ThreadSafety,
			S2: ThreadSafety,
		{
			#[inline(always)] // Proxy function.
			fn partial_cmp(&self, other: &$VdomName<'a, S2>) -> Option<core::cmp::Ordering> {
				Some(Ord::cmp(self.align_ref(), other.align_ref()))
			}
		}
		impl<'a, S> Ord for $VdomName<'a, S>
		where
			S: ThreadSafety,
		{
			fn cmp(&$cmp_self, $cmp_other: &Self) -> Ordering {
				$cmp
			}
		}
	)*};
}

macro_rules! cmp {
	($first:expr, $second:expr) => {
		let ord = Ord::cmp($first, $second);
		if !matches!(ord, Ordering::Equal) {
			return ord;
		}
	};
}

vdom_ergonomics!([
	Element {
		debug: |&self, f| f
			.debug_struct("Element")
			.field("name", &self.name)
			.field("creation_options", &self.creation_options)
			.field("attributes", &self.attributes)
			.field("event_bindings", &self.event_bindings)
			.field("content", &self.content) // Recursion.
			.finish(),
		partial_eq: |&self, other| self.name == other.name
			&& self.creation_options == other.creation_options
			&& self.attributes == other.attributes
			&& self.event_bindings == other.event_bindings
			&& self.content == other.content, // Recursion.
		hash: |&self, state| {
			self.name.hash(state);
			self.creation_options.hash(state);
			self.attributes.hash(state);
			self.event_bindings.hash(state);
			self.content.hash(state); // Recursion.
		},
		cmp: |&self, other| {
			cmp!(self.name, other.name);
			cmp!(&self.creation_options, &other.creation_options);
			cmp!(self.attributes, other.attributes);
			for i in 0..cmp::min(self.event_bindings.len(), other.event_bindings.len()) {
				cmp!(&self.event_bindings[i], &other.event_bindings[i]);
			}
			cmp!(&self.event_bindings.len(), &other.event_bindings.len());
			self.content.cmp(&other.content) // Recursion.
		},
	},
	EventBinding {
		debug: |&self, f| f
			.debug_struct("EventBinding")
			.field("name", &self.name)
			.field("callback", &self.callback)
			.field("options", &self.options)
			.finish(),
		partial_eq: |&self, other| self.name == other.name
			&& self.callback == other.callback
			&& self.options == other.options,
		hash: |&self, state| {
			self.name.hash(state);
			self.callback.hash(state);
			self.options.hash(state);
		},
		cmp: |&self, other| {
			cmp!(self.name, other.name);
			cmp!(&self.callback, &other.callback);
			self.options.cmp(&other.options)
		},
	},
	Node {
		debug: |&self, f| match self {
			Node::Comment {
				comment,
				dom_binding,
			} => f
				.debug_struct("Node::Comment")
				.field("comment", comment)
				.field("dom_binding", dom_binding)
				.finish(),
			Node::HtmlElement {
				element,
				dom_binding,
			} => f
				.debug_struct("Node::HtmlElement")
				.field("element", element)
				.field("dom_binding", dom_binding)
				.finish(),
			Node::MathMlElement {
				element,
				dom_binding,
			} => f
				.debug_struct("Node::HtmlElement")
				.field("element", element)
				.field("dom_binding", dom_binding)
				.finish(),
			Node::SvgElement {
				element,
				dom_binding,
			} => f
				.debug_struct("Node::SvgElement")
				.field("element", element)
				.field("dom_binding", dom_binding)
				.finish(),
			Node::Memoized { state_key, content } => f
				.debug_struct("Node::Memoized")
				.field("state_key", state_key)
				.field("content", content) // Recursion.
				.finish(),
			Node::Multi(nodes) => f.debug_tuple("Node::Multi").field(nodes).finish(),
			Node::Keyed(fragments) => f.debug_tuple("Node::Keyed").field(fragments).finish(),
			Node::Text { text, dom_binding } => f
				.debug_struct("Node::Text")
				.field("text", text)
				.field("dom_binding", dom_binding)
				.finish(),
			Node::RemnantSite(remnant_site) => f
				.debug_tuple("Node::RemnantSite")
				.field(remnant_site)
				.finish(),
		},
		partial_eq: |&self, other| match (self, other) {
			(
				Node::Comment {
					comment: c_1,
					dom_binding: db_1,
				},
				Node::Comment {
					comment: c_2,
					dom_binding: db_2,
				},
			) =>
				c_1 == c_2
					&& match (db_1, db_2) {
						(None, None) => true,
						(Some(db_1), Some(db_2)) => db_1 == db_2,
						(_, _) => false,
					},
			(Node::Comment { .. }, _) => false,
			(
				Node::HtmlElement {
					element: e_1,
					dom_binding: db_1,
				},
				Node::HtmlElement {
					element: e_2,
					dom_binding: db_2,
				},
			) =>
				e_1 == e_2
					&& match (db_1, db_2) {
						(None, None) => true,
						(Some(db_1), Some(db_2)) => db_1 == db_2,
						(_, _) => false,
					},
			(Node::HtmlElement { .. }, _) => false,
			(
				Node::MathMlElement {
					element: e_1,
					dom_binding: db_1,
				},
				Node::MathMlElement {
					element: e_2,
					dom_binding: db_2,
				},
			) =>
				e_1 == e_2
					&& match (db_1, db_2) {
						(None, None) => true,
						(Some(db_1), Some(db_2)) => db_1 == db_2,
						(_, _) => false,
					},
			(Node::MathMlElement { .. }, _) => false,
			(
				Node::SvgElement {
					element: e_1,
					dom_binding: db_1,
				},
				Node::SvgElement {
					element: e_2,
					dom_binding: db_2,
				},
			) =>
				e_1 == e_2
					&& match (db_1, db_2) {
						(None, None) => true,
						(Some(db_1), Some(db_2)) => db_1 == db_2,
						(_, _) => false,
					},
			(Node::SvgElement { .. }, _) => false,
			(
				Node::Memoized {
					state_key: sk_1, ..
				},
				Node::Memoized {
					state_key: sk_2, ..
				},
			) => sk_1 == sk_2,
			(Node::Memoized { .. }, _) => false,
			(Node::Multi(n_1), Node::Multi(n_2)) => n_1 == n_2, // Recursion.
			(Node::Multi(_), _) => false,
			(Node::Keyed(p_1), Node::Keyed(p_2)) => p_1 == p_2, // Recursion.
			(Node::Keyed(_), _) => false,
			(
				Node::Text {
					text: t_1,
					dom_binding: db_1,
				},
				Node::Text {
					text: t_2,
					dom_binding: db_2,
				},
			) =>
				t_1 == t_2
					&& match (db_1, db_2) {
						(None, None) => true,
						(Some(db_1), Some(db_2)) => db_1 == db_2,
						(_, _) => false,
					},
			(Node::Text { .. }, _) => false,
			(Node::RemnantSite(rs_1), Node::RemnantSite(rs_2)) => rs_1 == rs_2, // Recursion.
			(Node::RemnantSite(_), _) => false,
		},
		hash: |&self, state| match self {
			Node::Comment {
				comment,
				dom_binding,
			} => {
				comment.hash(state);
				dom_binding.hash(state);
			}
			Node::HtmlElement {
				element,
				dom_binding,
			} => {
				dom_binding.hash(state);
				element.hash(state); // Recursion.
			}
			Node::MathMlElement {
				element,
				dom_binding,
			} => {
				dom_binding.hash(state);
				element.hash(state); // Recursion.
			}
			Node::SvgElement {
				element,
				dom_binding,
			} => {
				dom_binding.hash(state);
				element.hash(state); // Recursion.
			}
			Node::Memoized { state_key, .. } => {
				state_key.hash(state)
			}
			Node::Multi(nodes) => nodes.hash(state), // Recursion.
			Node::Keyed(pairs) => pairs.hash(state), // Recursion.
			Node::Text { text, dom_binding } => {
				text.hash(state);
				dom_binding.hash(state)
			}
			Node::RemnantSite(remnant_site) => remnant_site.hash(state), // Recursion (eventually).
		},
		cmp: |&self, other| match (self, other) {
			(
				Node::Comment {
					comment: c_1,
					dom_binding: db_1,
				},
				Node::Comment {
					comment: c_2,
					dom_binding: db_2,
				},
			) => {
				cmp!(c_1, c_2);
				db_1.cmp(db_2)
			}
			(
				Node::HtmlElement {
					element: e_1,
					dom_binding: db_1,
				},
				Node::HtmlElement {
					element: e_2,
					dom_binding: db_2,
				},
			) => {
				cmp!(db_1, db_2);
				e_1.cmp(e_2) // Recursion.
			}
			(
				Node::MathMlElement {
					element: e_1,
					dom_binding: db_1,
				},
				Node::MathMlElement {
					element: e_2,
					dom_binding: db_2,
				},
			) => {
				cmp!(db_1, db_2);
				e_1.cmp(e_2) // Recursion.
			}
			(
				Node::SvgElement {
					element: e_1,
					dom_binding: db_1,
				},
				Node::SvgElement {
					element: e_2,
					dom_binding: db_2,
				},
			) => {
				cmp!(db_1, db_2);
				e_1.cmp(e_2) // Recursion.
			}
			(
				Node::Memoized {
					state_key: sk_1,
					content: c_1,
				},
				Node::Memoized {
					state_key: sk_2,
					content: c_2,
				},
			) => {
				cmp!(sk_1, sk_2);
				c_1.cmp(c_2) // Recursion.
			}
			(Node::Multi(c_1), Node::Multi(c_2)) => {
				c_1.cmp(c_2) // Recursion.
			}
			(Node::Keyed(c_1), Node::Keyed(c_2)) => {
				c_1.cmp(c_2)
			}
			(
				Node::Text {
					text: t_1,
					dom_binding: db_1,
				},
				Node::Text {
					text: t_2,
					dom_binding: db_2,
				},
			) => {
				cmp!(t_1, t_2);
				db_1.cmp(db_2)
			}
			(Node::RemnantSite(rs_1), Node::RemnantSite(rs_2)) => {
				rs_1.cmp(rs_2)
			}
			(Node::Comment { .. }, _) => Ordering::Less,
			(_, Node::Comment { .. }) => Ordering::Greater,
			(Node::HtmlElement { .. }, _) => Ordering::Less,
			(_, Node::HtmlElement { .. }) => Ordering::Greater,
			(Node::MathMlElement { .. }, _) => Ordering::Less,
			(_, Node::MathMlElement { .. }) => Ordering::Greater,
			(Node::SvgElement { .. }, _) => Ordering::Less,
			(_, Node::SvgElement { .. }) => Ordering::Greater,
			(Node::Memoized { .. }, _) => Ordering::Less,
			(_, Node::Memoized { .. }) => Ordering::Greater,
			(Node::Multi(_), _) => Ordering::Less,
			(_, Node::Multi(_)) => Ordering::Greater,
			(Node::Keyed(_), _) => Ordering::Less,
			(_, Node::Keyed(_)) => Ordering::Greater,
			(Node::Text { .. }, _) => Ordering::Less,
			(_, Node::Text { .. }) => Ordering::Greater,
		},
	},
	ReorderableFragment {
		debug: |&self, f| f
			.debug_struct("ReorderableFragment")
			.field("dom_key", &self.dom_key)
			.field("content", &self.content) // Recursion.
			.finish(),
		partial_eq: |&self, other| self.dom_key == other.dom_key && self.content == other.content,
		hash: |&self, state| {
			self.dom_key.hash(state);
			self.content.hash(state); // Recursion.
		},
		cmp: |&self, other| {
			cmp!(&self.dom_key, &other.dom_key);
			self.content.cmp(&other.content) // Recursion.
		},
	}
]);

// Conversions between distinct types //

impl<'a, S> Element<'a, S>
where
	S: ThreadSafety,
{
	/// Wraps a reference to this [`Element`] inside a [`Node::HtmlElement`] without [`dom_binding`](`Node::HtmlElement::dom_binding`).
	///
	/// # Example
	///
	/// ```rust
	/// use lignin::{ElementCreationOptions, Node, ThreadSafe};
	///
	/// fn allocate<'a, T>(value: T) -> &'a T {
	///   // […]
	///   # Box::leak(Box::new(value))
	/// }
	///
	/// let html_node: Node<ThreadSafe> = allocate(lignin::Element {
	///   name: "DIV",
	///   creation_options: ElementCreationOptions::new(),
	///   attributes: &[],
	///   content: Node::Multi(&[]),
	///   event_bindings: &[],
	/// }).as_html();
	/// ```
	#[must_use]
	pub fn as_html(&'a self) -> Node<'a, S> {
		Node::HtmlElement {
			element: self,
			dom_binding: None,
		}
	}

	/// Wraps a reference to this [`Element`] inside a [`Node::SvgElement`] without [`dom_binding`](`Node::SvgElement::dom_binding`).
	///
	/// # Example
	///
	/// ```rust
	/// use lignin::{ElementCreationOptions, Node, ThreadSafe};
	///
	/// fn allocate<'a, T>(value: T) -> &'a T {
	///   // […]
	///   # Box::leak(Box::new(value))
	/// }
	///
	/// let svg_node: Node<ThreadSafe> = allocate(lignin::Element {
	///   name: "SVG",
	///   creation_options: ElementCreationOptions::new(),
	///   attributes: &[],
	///   content: Node::Multi(&[]),
	///   event_bindings: &[],
	/// }).as_svg();
	/// ```
	#[must_use]
	pub fn as_svg(&'a self) -> Node<'a, S> {
		Node::SvgElement {
			element: self,
			dom_binding: None,
		}
	}
}

impl<'a, S1, S2> From<&'a [Node<'a, S1>]> for Node<'a, S2>
where
	S1: ThreadSafety + Into<S2>,
	S2: ThreadSafety,
{
	fn from(content: &'a [Node<'a, S1>]) -> Self {
		Node::Multi(content).align()
	}
}

impl<'a, S1, S2> From<&'a mut [Node<'a, S1>]> for Node<'a, S2>
where
	S1: ThreadSafety + Into<S2>,
	S2: ThreadSafety,
{
	fn from(content: &'a mut [Node<'a, S1>]) -> Self {
		Node::Multi(content).align()
	}
}

impl<'a, S> From<&'a str> for Node<'a, S>
where
	S: ThreadSafety,
{
	fn from(text: &'a str) -> Self {
		Self::Text {
			text,
			dom_binding: None,
		}
	}
}

impl<'a, S> From<&'a mut str> for Node<'a, S>
where
	S: ThreadSafety,
{
	fn from(text: &'a mut str) -> Self {
		Self::Text {
			text,
			dom_binding: None,
		}
	}
}

impl<'a, S: ThreadSafety> Node<'a, S> {
	/// Calculates the aggregate surface level length of this [`Node`] in [***Node***](https://developer.mozilla.org/en-US/docs/Web/API/Node)s.
	///
	/// This operation is recursive across *for example* [`Node::Multi`] and [`Node::Keyed`], which sum up their contents in this regard.
	#[must_use]
	#[allow(clippy::missing_panics_doc)] // todo!
	pub fn dom_len(&self) -> usize {
		match self {
			Node::Comment { .. }
			| Node::HtmlElement { .. }
			| Node::MathMlElement { .. }
			| Node::SvgElement { .. }
			| Node::Text { .. } => 1,
			Node::Memoized { content: node, .. } => node.dom_len(),
			Node::Multi(nodes) => nodes.iter().map(Node::dom_len).sum(),
			Node::Keyed(pairs) => pairs.iter().map(|pair| pair.content.dom_len()).sum(),
			Node::RemnantSite(_) => {
				todo!("RemnantSite dom_len")
			}
		}
	}

	/// Determines whether this [`Node`] represents no [***Node***](https://developer.mozilla.org/en-US/docs/Web/API/Node)s at all.
	///
	/// This operation is recursive across *for example* [`Node::Multi`] and [`Node::Keyed`], which sum up their contents in this regard.
	#[must_use]
	#[allow(clippy::missing_panics_doc)] // todo!
	pub fn dom_empty(&self) -> bool {
		match self {
			Node::Comment { .. }
			| Node::HtmlElement { .. }
			| Node::MathMlElement { .. }
			| Node::SvgElement { .. }
			| Node::Text { .. } => false,
			Node::Memoized { content, .. } => content.dom_empty(),
			Node::Multi(nodes) => nodes.iter().all(Node::dom_empty),
			Node::Keyed(pairs) => pairs.iter().all(|pair| pair.content.dom_empty()),
			Node::RemnantSite(_) => {
				todo!("RemnantSite dom_empty")
			}
		}
	}
}

impl Debug for EventBindingOptions {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.debug_struct("EventBindingOptions")
			.field("capture", &self.capture())
			.field("once", &self.once())
			.field("passive", &self.passive())
			.finish()
	}
}
