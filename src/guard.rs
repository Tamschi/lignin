//! A VDOM [`Drop`] guard for compatibility between caching components and containers in general.
//!
//! If a [`Node`] producer neither caches nor can act as container for other components which may, then it's fine to return a plain [`Node`] or [`&Node`](https://doc.rust-lang.org/stable/std/primitive.reference.html).

use crate::{Node, ThreadSafety};
use core::{marker::PhantomData, mem::MaybeUninit};

/// A type-erased callback that's consumed upon calling and doesn't need to be allocated inside a `Box<_>`.
///
/// > This should really be either a trait callable with `self: *const Self` or better yet
/// > a `Pin<Box<dyn Send + Sync + Guarded>, Pointing>` where [`Pointing: Allocator`](https://doc.rust-lang.org/stable/core/alloc/trait.Allocator.html)
/// > does absolution nothing. Both is unstable, though.
#[must_use = "Dropping a `ConsumeCallback` does not call it, potentially leaking memory."]
pub struct ConsumedCallback<'a> {
	call: fn(*const ()),
	with: *const (),
	_phantom: PhantomData<&'a ()>,
}
impl<'a> ConsumedCallback<'a> {
	/// Creates a new instance of [`ConsumedCallback`].
	///
	/// # Safety
	///
	/// `call` may be called up to once, with `with`, but only within `'a`.
	pub unsafe fn new(call: fn(*const ()), with: *const ()) -> Self {
		Self {
			call,
			with,
			_phantom: PhantomData,
		}
	}

	/// Invokes the callback.
	pub fn call(self) {
		(self.call)(self.with)
	}
}

/// A drop guard for a shared [`Node`].
///
/// # Implementation Contract
///
/// [`Guard`] consumers **may** delay dropping them arbitrarily, so [`Guard`] producers **should not** rely on that for correctness.
///
/// [`Guard`] consumers **should not** leak them, as [`Guard`] producers **may** leak memory in such a case.
///
/// > These are strong suggestions, since those "**may**"s are likely to be quite common.
/// >
/// > For example, a double-buffering differ running in a browser, as of writing e.g. [lignin-dom](https://docs.rs/lignin-dom/latest/lignin_dom/),
/// > will always delay dropping past the rendering of the updated VDOM.
/// >
/// > On a server, it may make sense to create an atomically- and periodically updated cache for part of the page,
/// > if it renders very slowly for some reason. (I.e. an app could render out a VDOM while calculating or retrieving data synchronously, even if it *probably shouldn't*.)
/// >
/// > In terms of leaks, a good example is subtree caching, which due to delayed [`Guard`] drops **must** store any number of states as necessary or panic if it won't/can't at some point.
pub struct Guard<'a, S: ThreadSafety> {
	vdom: &'a Node<'a, S>,
	guarded: Option<ConsumedCallback<'a>>,
}
impl<'a, S: ThreadSafety> Guard<'a, S> {
	/// Creates a new instance of [`Guard`] which calls `guarded` once only when dropped.
	#[must_use]
	pub fn new_with_callback(vdom: &'a Node<'a, S>, guarded: Option<ConsumedCallback<'a>>) -> Self {
		Self { vdom, guarded }
	}

	///
	/// # Safety
	///
	/// The returned [`Node`] reference becomes invalid once the returned [`ConsumedCallback`] is called.
	#[must_use = "Calling this method may leak memory unless any returned `ConsumedCallback` is called later on."]
	pub unsafe fn leak(mut self) -> (&'a Node<'a, S>, Option<ConsumedCallback<'a>>) {
		(self.vdom, self.guarded.take())
	}

	/// Splits off and stores this [`Guard`]'s drop-[`ConsumedCallback`], leaving an [`&Node<'a, S>`](`Node`).
	///
	/// # Safety
	///
	/// The returned [`Node`] reference becomes invalid once `add_to`'s value is called, if [`Some`] after this call.
	pub unsafe fn peel(
		mut self,
		add_to: &mut Option<ConsumedCallback<'a>>,
		allocate: impl FnOnce() -> &'a mut MaybeUninit<[ConsumedCallback<'a>; 2]>,
	) -> &'a Node<'a, S> {
		if let Some(peel) = self.guarded.take() {
			*add_to = Some(match add_to.take() {
				Some(previous) => {
					fn call_both(both: *const ()) {
						let [first, second] =
							unsafe { both.cast::<[ConsumedCallback<'static>; 2]>().read() };
						first.call();
						second.call();
					}
					let both = (allocate().write([previous, peel])
						as *const [ConsumedCallback<'a>; 2])
						.cast();
					ConsumedCallback::new(call_both, both)
				}
				None => peel,
			});
		}
		self.vdom
	}

	/// Transforms the VDOM without manipulating the callback.
	pub fn map<S2: ThreadSafety>(
		mut self,
		f: impl for<'any> FnOnce(&'any Node<'any, S>) -> &'any Node<'any, S2>,
	) -> Guard<'a, S2> {
		Guard {
			vdom: f(self.vdom),
			guarded: self.guarded.take(),
		}
	}

	/// Transforms the VDOM, optionally adding on another callback.
	pub fn flat_map<S2: ThreadSafety>(
		mut self,
		allocate: impl FnOnce() -> &'a mut MaybeUninit<[ConsumedCallback<'a>; 2]>,
		f: impl for<'any> FnOnce(&'any Node<'any, S>) -> Guard<'any, S2>,
	) -> Guard<'a, S2> {
		unsafe {
			//SAFETY:
			// `self.vdom` can't escape from `f` due to its `'any` lifetime there.
			// The peeled callback is immediately recombined.
			Guard {
				vdom: f(self.vdom).peel(&mut self.guarded, allocate),
				guarded: self.guarded.take(),
			}
		}
	}
}

impl<S: ThreadSafety> Drop for Guard<'_, S> {
	fn drop(&mut self) {
		if let Some(guarded) = self.guarded.take() {
			guarded.call()
		}
	}
}
