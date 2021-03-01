//! This module is private but contains various convenience implementations not used by the rest of the library that may be useful to consumers of this crate.
#![allow(clippy::match_same_arms)]

//TODO: Implement `PartialOrd` and `Ord`.

use crate::{
	auto_safety::Align, CallbackRef, CallbackRegistration, Element, EventBinding, Node,
	ReorderableFragment, ThreadBound, ThreadSafe, ThreadSafety,
};
use core::{
	any::type_name,
	fmt::{self, Debug, Formatter},
	hash::{Hash, Hasher},
};

//TODO:
// The derives of common traits for most types in this library emit bounds on S, which aren't necessary but appear in the documentation.
// It would be cleaner to explicitly implement all of these traits.

//TODO: How important are `PartialOrd` and `Ord`?

impl From<ThreadSafe> for ThreadBound {
	/// Unreachable. Available as compatibility marker when handling generic [`ThreadSafety`] directly.
	fn from(_: ThreadSafe) -> Self {
		unreachable!()
	}
}

impl<T> From<CallbackRef<ThreadSafe, T>> for CallbackRef<ThreadBound, T> {
	#[allow(clippy::inline_always)]
	#[inline(always)] // No-op.
	fn from(thread_safe: CallbackRef<ThreadSafe, T>) -> Self {
		thread_safe.align()
	}
}

impl<S: ThreadSafety, T> Debug for CallbackRef<S, T> {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_struct(type_name::<Self>())
			.field("key", &self.key)
			.finish()
	}
}

impl<S: ThreadSafety, T> Clone for CallbackRef<S, T> {
	fn clone(&self) -> Self {
		*self
	}
}
impl<S: ThreadSafety, T> Copy for CallbackRef<S, T> {}
impl<S1: ThreadSafety, S2: ThreadSafety, T> PartialEq<CallbackRef<S2, T>> for CallbackRef<S1, T> {
	fn eq(&self, other: &CallbackRef<S2, T>) -> bool {
		self.key == other.key
	}
}
impl<S: ThreadSafety, T> Eq for CallbackRef<S, T> {}
impl<S: ThreadSafety, T> Hash for CallbackRef<S, T> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.key.hash(state)
	}
}

impl<R, T> From<&CallbackRegistration<R, T>> for CallbackRef<ThreadSafe, T>
where
	R: Sync,
{
	fn from(registration: &CallbackRegistration<R, T>) -> Self {
		registration.to_ref()
	}
}

impl<R, T> From<&CallbackRegistration<R, T>> for CallbackRef<ThreadBound, T> {
	fn from(registration: &CallbackRegistration<R, T>) -> Self {
		registration.to_ref_thread_bound()
	}
}

macro_rules! vdom_ergonomics {
	([$(
		$VdomName:ident {
			debug: |&$debug_self:ident, $debug_f:ident| $debug:expr,
			partial_eq: |&$eq_self:ident, $eq_other:ident| $partial_eq:expr,
			hash: |&$hash_self:ident, $hash_state:ident| $hash:expr,
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
	)*};
}

vdom_ergonomics!([
	Element {
		debug: |&self, f| f
			.debug_struct("Element")
			.field("name", &self.name)
			.field("attributes", &self.attributes)
			.field("content", &self.content)
			.field("event_bindings", &self.event_bindings)
			.finish(),
		partial_eq: |&self, other| self.name == other.name
			&& self.attributes == other.attributes
			&& self.content == other.content
			&& self.event_bindings == other.event_bindings,
		hash: |&self, state| {
			self.name.hash(state);
			self.attributes.hash(state);
			self.event_bindings.hash(state);
			self.content.hash(state); // Recursion.
		},
	},
	EventBinding {
		debug: |&self, f| f
			.debug_struct("EventBinding")
			.field("name", &self.name)
			.field("callback", &self.callback)
			.finish(),
		partial_eq: |&self, other| self.name == other.name && self.callback == other.callback,
		hash: |&self, state| {
			self.name.hash(state);
			self.callback.hash(state);
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
			Node::Element {
				element,
				dom_binding,
			} => f
				.debug_struct("Node::Element")
				.field("element", element)
				.field("dom_binding", dom_binding)
				.finish(),
			Node::Memoized { state_key, content } => f
				.debug_struct("Node::Memoized")
				.field("state_key", state_key)
				.field("content", content)
				.finish(),
			Node::Multi(nodes) => f.debug_tuple("Node::Multi").field(nodes).finish(),
			Node::Keyed(pairs) => f.debug_tuple("Node::Keyed").field(pairs).finish(),
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
				Node::Element {
					element: e_1,
					dom_binding: db_1,
				},
				Node::Element {
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
			(Node::Element { .. }, _) => false,
			(
				Node::Memoized {
					state_key: sk_1, ..
				},
				Node::Memoized {
					state_key: sk_2, ..
				},
			) => sk_1 == sk_2,
			(Node::Memoized { .. }, _) => false,
			(Node::Multi(n_1), Node::Multi(n_2)) => n_1 == n_2,
			(Node::Multi(_), _) => false,
			(Node::Keyed(p_1), Node::Keyed(p_2)) => p_1 == p_2,
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
			(Node::RemnantSite(rs_1), Node::RemnantSite(rs_2)) => rs_1 == rs_2,
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
			Node::Element {
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
	},
	ReorderableFragment {
		debug: |&self, f| f
			.debug_struct("ReorderableFragment")
			.field("dom_key", &self.dom_key)
			.field("content", &self.content)
			.finish(),
		partial_eq: |&self, other| self.dom_key == other.dom_key && self.content == other.content,
		hash: |&self, state| {
			self.dom_key.hash(state);
			self.content.hash(state); // Recursion.
		},
	}
]);

// Conversions between distinct types //

impl<'a, S> From<&'a Element<'a, S>> for Node<'a, S>
where
	S: ThreadSafety,
{
	fn from(element: &'a Element<'a, S>) -> Self {
		Self::Element {
			element,
			dom_binding: None,
		}
	}
}

impl<'a, S> From<&'a mut Element<'a, S>> for Node<'a, S>
where
	S: ThreadSafety,
{
	fn from(element: &'a mut Element<'a, S>) -> Self {
		Self::Element {
			element,
			dom_binding: None,
		}
	}
}

impl<'a, S: ThreadSafety> Node<'a, S> {
	// Calculates the aggregate surface level length of this [`Node`] in DOM nodes.
	//
	// This operation is recursive across *for example* [`Node::Multi`] and [`Node::Keyed`], which sum up their contents in this regard.
	#[must_use]
	#[allow(clippy::missing_panics_doc)] //TODO
	pub fn dom_len(&self) -> usize {
		match self {
			Node::Comment { .. } | Node::Element { .. } | Node::Text { .. } => 1,
			Node::Memoized { content: node, .. } => node.dom_len(),
			Node::Multi(nodes) => nodes.iter().map(Node::dom_len).sum(),
			Node::Keyed(pairs) => pairs.iter().map(|pair| pair.content.dom_len()).sum(),
			Node::RemnantSite(_) => {
				todo!("RemnantSite dom_len")
			}
		}
	}
}
