//! This module is private but contains various convenience implementations not used by the rest of the library that may be useful to consumers of this crate.
#![allow(clippy::match_same_arms)]

use crate::{
	auto_safety::Align, CallbackRef, CallbackRegistration, Element, EventBinding, Node,
	ThreadBound, ThreadSafe, ThreadSafety,
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
impl<S1: ThreadSafety, S2: ThreadSafety, T> PartialOrd<CallbackRef<S2, T>> for CallbackRef<S1, T> {
	fn partial_cmp(&self, other: &CallbackRef<S2, T>) -> Option<core::cmp::Ordering> {
		self.key.partial_cmp(&other.key)
	}
}
impl<S: ThreadSafety, T> Ord for CallbackRef<S, T> {
	fn cmp(&self, other: &Self) -> core::cmp::Ordering {
		self.key.cmp(&other.key)
	}
}
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

		impl<'a, S: ThreadSafety> Debug for $VdomName<'a, S> {
			fn fmt(&$debug_self, $debug_f: &mut Formatter<'_>) -> fmt::Result {
				$debug
			}
		}

		impl<'a, S: ThreadSafety> Clone for $VdomName<'a, S> {
			fn clone(&self) -> Self {
				*self
			}
		}
		impl<'a, S: ThreadSafety> Copy for $VdomName<'a, S> {}

		impl<'a, S1: ThreadSafety, S2: ThreadSafety> PartialEq<$VdomName<'a, S2>> for $VdomName<'a, S1> {
			fn eq(&$eq_self, $eq_other: &$VdomName<'a, S2>) -> bool {
				$partial_eq
			}
		}
		impl<'a, S: ThreadSafety> Eq for $VdomName<'a, S> {}

		impl<'a, S: ThreadSafety> Hash for $VdomName<'a, S> {
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
			Node::Ref(node) => f.debug_tuple("Node::Ref").field(node).finish(),
			Node::Multi(nodes) => f.debug_tuple("Node::Ref").field(nodes).finish(),
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
			(Node::Ref(n_1), Node::Ref(n_2)) => n_1 == n_2,
			(Node::Ref(_), _) => false,
			(Node::Multi(n_1), Node::Multi(n_2)) => n_1 == n_2,
			(Node::Multi(_), _) => false,
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
			Node::Ref(node) => node.hash(state),     // Recursion.
			Node::Multi(nodes) => nodes.hash(state), // Recursion.
			Node::Text { text, dom_binding } => {
				text.hash(state);
				dom_binding.hash(state)
			}
			Node::RemnantSite(remnant_site) => remnant_site.hash(state), // Recursion (eventually).
		},
	}
]);

// Conversions between distinct types //

impl<'a: 'b, 'b, S: ThreadSafety> From<&'a Element<'a, S>> for Node<'b, S> {
	fn from(element: &'a Element<'a, S>) -> Self {
		Self::Element {
			element,
			dom_binding: None,
		}
	}
}

impl<'a: 'b, 'b, S: ThreadSafety> From<&'a mut Element<'a, S>> for Node<'b, S> {
	fn from(element: &'a mut Element<'a, S>) -> Self {
		Self::Element {
			element,
			dom_binding: None,
		}
	}
}
