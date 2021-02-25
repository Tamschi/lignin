//! Transitive [`ThreadSafety`] inference, mainly for use by frameworks.
//!
//! All methods in this module are always-inlined no-ops, meaning that there is zero runtime cost to them.
//!
//! > This feature relies on opaque return types (`-> impl Trait`) leaking [`Send`] and [`Sync`], so the theoretical limit here, even after specialisation lands, are four distinct 'real' types with restrictions on conversion incompatibilities.
//! > Fortunately, `lignin` only needs two of these slots with straightforward compatibility, the `!Send + !Sync` and the `Send + Sync` one.
//! >
//! > Please refer to the item documentation for implementation details.
#![allow(clippy::inline_always)]

use crate::{Node, ThreadBound, ThreadSafe, ThreadSafety};

mod sealed {
	/// It's probably good to be a bit more specific in [`Align`](`super::Align`)'s signature, among others.
	/// The bounds are necessary the default implementations in derived traits and also prevent their object-safety, which is good because that would at best only add useless dynamic dispatch overhead.
	pub trait ThreadBindable: Copy + Sized {}
}

/// Deanonymize towards the general ([`ThreadBound`]) case.
pub trait Auto<ThreadBound>
where
	Self: sealed::ThreadBindable,
	ThreadBound: sealed::ThreadBindable,
{
	/// Deanonymize towards a compatible concrete type.
	///
	/// This method is by reference, so it will resolve with lower priority than the by-value method on [`Deanonymize`].  
	/// Note that not all tooling will show the correct overload here, but the compiler knows which to pick.
	#[must_use]
	#[inline(always)] // No-op.
	fn deanonymize(&self) -> ThreadBound {
		unsafe {
			// SAFETY:
			// Under normal circumstances, this trait or method would have to be `unsafe`.
			// However, we're ensuring only sound implementations exist by sealing it and carefully implementing it only across layout-compatible types.
			*(self as *const _ as *const _)
		}
	}
}

/// Deanonymize towards the special ([`ThreadSafe`]) case. **This trait must be in scope for correct inference!**
pub trait Deanonymize<'a>: sealed::ThreadBindable + Send + Sync {
	type ThreadSafe: sealed::ThreadBindable;
	/// Deanonymize towards a compatible concrete type.
	///
	/// This method is by value, so it will resolve with higher priority than the by-reference method on [`Auto`].  
	/// Note that not all tooling will show the correct overload here, but the compiler knows which to pick.
	#[must_use]
	#[inline(always)] // No-op.
	fn deanonymize(self) -> Self::ThreadSafe {
		unsafe {
			// SAFETY:
			// Under normal circumstances, this trait or method would have to be `unsafe`.
			// However, we're ensuring only sound implementations exist by sealing it and carefully implementing it only across layout-compatible types.
			*(&self as *const _ as *const _)
		}
	}
}

impl<'a, S: ThreadSafety> sealed::ThreadBindable for Node<'a, S> {}

impl<'a, S: ThreadSafety> Auto<Node<'a, ThreadBound>> for Node<'a, S> {}
impl<'a, T: Send + Sync + Auto<Node<'a, ThreadBound>>> Deanonymize<'a> for T {
	type ThreadSafe = Node<'a, ThreadSafe>;
}

impl<'a> Node<'a, ThreadSafe> {
	/// Gently nudges the compiler to choose the thread-safe version of a value if either is possible.
	///
	/// This method is by value, so it will resolve with higher priority than the by-reference method on the thread-bound type.  
	/// Note that not all tooling will show the correct overload here, but the compiler knows which to pick.
	#[must_use]
	#[inline(always)] // No-op.
	pub fn prefer_thread_safe(self) -> Self {
		self
	}
}
impl<'a> Node<'a, ThreadBound> {
	/// Gently nudges the compiler to choose the thread-safe version of a value if either is possible.
	///
	/// This method is by reference, so it will resolve with lower priority than the by-reference method on the thread-safe type.  
	/// Note that not all tooling will show the correct overload here, but the compiler knows which to pick.
	#[must_use]
	#[inline(always)] // No-op.
	pub fn prefer_thread_safe(&self) -> Self {
		*self
	}
}
impl<'a> From<Node<'a, ThreadSafe>> for Node<'a, ThreadBound> {
	#[allow(clippy::inline_always)]
	#[inline(always)] // No-op.
	fn from(thread_safe: Node<'a, ThreadSafe>) -> Self {
		unsafe { *(&thread_safe as *const _ as *const _) }
	}
}

/// Contextually thread-binds an instance, or not. Use only without qualification.
///
/// This trait acts as (i.e.: _is_) [`Into`] on and between thread-bindable types, but without raising `useless_conversion` warnings.
pub trait Align<T: sealed::ThreadBindable>: sealed::ThreadBindable
where
	Self: Into<T>,
{
	/// Contextually thread-binds an instance, or not. Use only without qualification.
	#[allow(clippy::inline_always)]
	#[inline(always)] // No-op.
	fn align(self) -> T {
		self.into()
	}
}
impl<T: Into<U>, U> Align<U> for T
where
	T: sealed::ThreadBindable,
	U: sealed::ThreadBindable,
{
}
