//! Callback registry plumbing, for renderers and app runners that support them **and** need to run indefinitely.

use crate::{sealed::Sealed, ThreadBound, ThreadSafe, ThreadSafety};
use core::{
	fmt::Debug,
	marker::{PhantomData, PhantomPinned},
	num::NonZeroU32,
	pin::Pin,
};

#[cfg(feature = "callbacks")]
mod callbacks_on {
	extern crate std;

	use super::CallbackRegistration;
	use core::{
		convert::TryInto,
		marker::{PhantomData, PhantomPinned},
		mem,
		num::NonZeroU32,
		pin::Pin,
	};
	use lazy_static::lazy_static;
	use mem::size_of_val;
	use std::{collections::HashMap, sync::RwLock};

	lazy_static! {
		static ref REGISTRY: RwLock<Registry> = RwLock::default();
	}

	struct Registry {
		key_count: u32,
		entries: HashMap<NonZeroU32, Entry>,
	}
	impl Default for Registry {
		fn default() -> Self {
			Self {
				key_count: 0,
				entries: HashMap::default(),
			}
		}
	}

	struct Entry {
		receiver_address: usize,
		invoke_typed_address: usize,
		handler_address: usize,
	}

	#[must_use]
	pub fn register<R, T>(
		receiver: Pin<&'_ R>,
		handler: fn(*const R, T),
	) -> CallbackRegistration<R, T> {
		let mut registry = REGISTRY.write().expect("always Ok");
		if registry.key_count == u32::MAX {
			drop(registry);
			panic!("[lignin] Callback registry keys exhausted")
		} else {
			fn invoke_typed<R, T>(receiver_address: usize, handler_address: usize, parameter: T) {
				let receiver = receiver_address as *const R;
				let handler = unsafe {
					// SAFETY: The pointer to invoke_typed is taken with matching monomorphization just below.
					mem::transmute::<usize, fn(*const R, T)>(handler_address)
				};
				handler(receiver, parameter)
			}

			registry.key_count += 1;
			let key = NonZeroU32::new(registry.key_count).expect("always Some");
			assert!(registry
				.entries
				.insert(
					key,
					Entry {
						receiver_address: receiver.get_ref() as *const R as usize,
						invoke_typed_address: invoke_typed::<R, T> as usize,
						handler_address: handler as usize,
					},
				)
				.is_none());
			CallbackRegistration {
				key,
				phantom: PhantomData,
				pinned: PhantomPinned,
			}
		}
	}

	pub fn deregister<R, T>(registration: &CallbackRegistration<R, T>) {
		let removed = REGISTRY
			.write()
			.expect("always Ok")
			.entries
			.remove(&registration.key)
			.is_some();
		assert!(removed)
	}

	pub fn invoke<T>(key: NonZeroU32, parameter: T) {
		let registry = REGISTRY.read().expect("always Ok");
		if let Some(entry) = registry.entries.get(&key) {
			let invoke_typed = unsafe {
				// SAFETY: Same type as above.
				mem::transmute::<usize, fn(usize, usize, T)>(entry.invoke_typed_address)
			};
			invoke_typed(entry.receiver_address, entry.handler_address, parameter)
		}
	}

	#[must_use]
	pub fn registry_exhaustion() -> u8 {
		let registry = REGISTRY.read().expect("always Ok");
		(registry.key_count >> ((size_of_val(&registry.key_count) - 1) * 8))
			.try_into()
			.expect("always Ok")
	}
}

#[allow(dead_code)]
#[allow(clippy::inline_always)] // All functions are no operations or constants or similar.
#[allow(clippy::let_underscore_drop)]
#[allow(clippy::needless_pass_by_value)]
mod callbacks_off {
	use core::{
		marker::{PhantomData, PhantomPinned},
		num::NonZeroU32,
		pin::Pin,
	};

	use super::CallbackRegistration;

	#[inline(always)]
	#[must_use]
	pub fn register<R, T>(
		receiver: Pin<&'_ R>,
		handler: fn(*const R, T),
	) -> CallbackRegistration<R, T> {
		let _ = receiver;
		let _ = handler;
		CallbackRegistration {
			key: NonZeroU32::new(u32::MAX).expect("always Ok"),
			phantom: PhantomData,
			pinned: PhantomPinned,
		}
	}

	#[inline(always)]
	pub const fn deregister<R, T>(registration: &CallbackRegistration<R, T>) {
		let _ = registration;
	}

	#[inline(always)]
	pub fn invoke<T>(key: NonZeroU32, parameter: T) {
		let _ = key;
		let _ = parameter;
	}

	#[inline(always)]
	#[must_use]
	pub const fn registry_exhaustion() -> u8 {
		0
	}
}

#[cfg(feature = "callbacks")]
use callbacks_on as callbacks;

#[cfg(not(feature = "callbacks"))]
use callbacks_off as callbacks;

/// A callback registration handle that should be held onto by the matching receiver `R` or a container with [pin-projection](https://doc.rust-lang.org/stable/core/pin/index.html#pinning-is-structural-for-field) towards that value.
///
/// [`CallbackRegistration`] is [`!Unpin`](`Unpin`) for convenience: A receiver correctly becomes [`!Unpin`](`Unpin`) if it contains for example a `Cell<Option<CallbackRegistration<R, T>>`¹⁻².
///
/// - - -
///
/// 1. [`impl<T: ?Sized> Unpin for Cell<T> where T: Unpin`](`core::cell::Cell`#impl-Unpin)
/// 2. [`impl<T> Unpin for Option<T> where T: Unpin`](`core::option::Option`#impl-Unpin)
#[allow(clippy::type_complexity)]
#[derive(Debug)]
pub struct CallbackRegistration<R, T> {
	key: NonZeroU32,
	///FIXME: Can this be written with `&R` (removing the manual `Send` and `Sync` impls below)?
	phantom: PhantomData<(*const R, fn(T))>,
	pinned: PhantomPinned,
}
// SAFETY: `CallbackRegistration<R, T>` only refers to a `*const R`, so it acts like `&R` for thread-safety.
//
// Without the `"callbacks"` feature, that pointer is actually unreachable, so this type *could* be more generally `Send` and `Sync`.
// However, since a CallbackRegistration is intended to be primarily handled by the matching `R` instance, this isn't done in order to retain consistency.
unsafe impl<R, T> Send for CallbackRegistration<R, T> where R: Sync {}
unsafe impl<R, T> Sync for CallbackRegistration<R, T> where R: Sync {}
#[allow(clippy::inline_always)] // All functions are very simple.
impl<R, T> CallbackRegistration<R, T> {
	/// Creates a new [`CallbackRegistration<R, T>`] with the given `receiver` and `handler`.
	///
	/// # Safety
	///
	/// **The `receiver` pointer given to `handler` may dangle unless `receiver` remains pinned until the created [`CallbackRegistration`] is dropped.**
	///
	/// You can ensure this most easily by storing the latter in for example a `Cell<Option<CallbackRegistration>>` embedded in the `receiver`.
	///
	/// Dropping the [`CallbackRegistration`] instance prevents any further calls to `handler` through it.
	#[inline(always)] // Proxy function.
	#[must_use]
	pub fn new(receiver: Pin<&'_ R>, handler: fn(receiver: *const R, parameter: T)) -> Self {
		callbacks::register(receiver, handler)
	}

	#[inline(always)] // Basically just a deref-copy.
	#[must_use]
	pub fn to_ref_thread_bound(&self) -> CallbackRef<ThreadBound, T> {
		self.to_ref() // Actually resolves to `ToRefThreadBoundFallback::to_ref`.
	}
}
impl<R, T> CallbackRegistration<R, T>
where
	R: Sync,
{
	// Using a separate `impl` block instead of a `where` clause on the method means it outright doesn't exist if `R: !Sync`.
	// This lets it be resolved on the trait instead even without qualification.

	#[allow(clippy::inline_always)]
	#[inline(always)] // Basically just a deref-copy.
	#[must_use]
	pub fn to_ref(&self) -> CallbackRef<ThreadSafe, T> {
		CallbackRef {
			key: self.key,
			phantom: PhantomData,
		}
	}
}
/// Provides a fallback alternative implementation to [`CallbackRegistry::to_ref`] for use in macro frameworks.
///
/// There is no limitation on the receiver's [`Sync`]ness, but in turn the resulting [`CallbackRef`] is [`ThreadBound`].
pub trait ToRefThreadBoundFallback<T>: Sealed + Sized {
	/// See [`CallbackRegistration::to_ref`], except that this method is unconstrained and that the resulting [`CallbackRef`] is [`ThreadBound`].
	fn to_ref(&self) -> CallbackRef<ThreadBound, T>;
}
impl<R, T> ToRefThreadBoundFallback<T> for CallbackRegistration<R, T> {
	#[allow(clippy::inline_always)]
	#[inline(always)] // Proxy function.
	fn to_ref(&self) -> CallbackRef<ThreadBound, T> {
		self.to_ref_thread_bound()
	}
}
impl<R, T> Drop for CallbackRegistration<R, T> {
	#[allow(clippy::inline_always)]
	#[inline(always)] // Proxy function.
	fn drop(&mut self) {
		callbacks::deregister(self)
	}
}

/// [`Vdom`](`crate::Vdom`) A callback reference linked to a [`CallbackRegistration`].
pub struct CallbackRef<S: ThreadSafety, T> {
	pub(crate) key: NonZeroU32,
	phantom: PhantomData<(S, fn(T))>,
}
impl<S: ThreadSafety, T> CallbackRef<S, T> {
	/// Invokes the stored handler with the stored receiver and `parameter`,
	/// provided that the original [`CallbackRegistration`] hasn't been dropped yet.
	#[allow(clippy::inline_always)]
	#[inline(always)] // Proxy function.
	pub fn call(self, parameter: T) {
		callbacks::invoke(self.key, parameter)
	}
}

/// Indicates how exhausted the global callback registry is on a linear scale, with `0` indicating no or very low exhaustion and `255` indicating almost complete or complete exhaustion.
#[allow(clippy::inline_always)]
#[inline(always)] // Proxy function.
#[must_use]
pub fn registry_exhaustion() -> u8 {
	callbacks::registry_exhaustion()
}

#[allow(clippy::inline_always)]
#[inline(always)] // Proxy function.
pub unsafe fn reset_callback_registry() -> Result<(), ()> {
	todo!()
}

#[allow(clippy::inline_always)]
#[inline(always)] // Proxy function.
pub unsafe fn yet_more_unsafe_force_clear_callback_registry() {
	todo!()
}
