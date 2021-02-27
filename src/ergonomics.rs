//! This module is private but contains various convenience implementations not used by the rest of the library that may be useful to consumers of this crate.

use crate::{
	auto_safety::Align, CallbackRef, CallbackRegistration, Node, ThreadBound, ThreadSafe,
	ThreadSafety,
};
use core::{
	any::type_name,
	fmt::Debug,
	hash::{Hash, Hasher},
};

//TODO:
// The derives of common traits for most types in this library emit bounds on S, which aren't necessary but appear in the documentation.
// It would be cleaner to explicitly implement all of these traits.

impl From<ThreadSafe> for ThreadBound {
	/// Unreachable. Available as compatibility marker when handling generic [`ThreadSafety`] directly.
	fn from(_: ThreadSafe) -> Self {
		unreachable!()
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
impl<S: ThreadSafety, T> PartialEq for CallbackRef<S, T> {
	fn eq(&self, other: &Self) -> bool {
		self.key == other.key
	}
}
impl<S: ThreadSafety, T> Eq for CallbackRef<S, T> {}
impl<S: ThreadSafety, T> PartialOrd for CallbackRef<S, T> {
	fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
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

impl<'a> From<Node<'a, ThreadSafe>> for Node<'a, ThreadBound> {
	#[allow(clippy::inline_always)]
	#[inline(always)] // No-op.
	fn from(thread_safe: Node<'a, ThreadSafe>) -> Self {
		thread_safe.align()
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
