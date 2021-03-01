//! Callback registry plumbing, for renderers and app runners that support them **and** need to run indefinitely.

use crate::{ThreadBound, ThreadSafe, ThreadSafety};
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

	/// Indicates how exhausted the global callback registry is on a linear scale, with `0` indicating no or very low exhaustion and `255` indicating almost complete or complete exhaustion.
	#[must_use]
	pub fn registry_exhaustion() -> u8 {
		let registry = REGISTRY.read().expect("always Ok");
		(registry.key_count >> ((size_of_val(&registry.key_count) - 1) * 8))
			.try_into()
			.expect("always Ok")
	}
}

#[allow(dead_code)]
#[allow(clippy::let_underscore_drop)]
#[allow(clippy::needless_pass_by_value)]
mod callbacks_off {
	use core::{
		marker::{PhantomData, PhantomPinned},
		num::NonZeroU32,
		pin::Pin,
	};

	use super::CallbackRegistration;

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

	pub fn deregister<R, T>(registration: &CallbackRegistration<R, T>) {
		let _ = registration;
	}

	pub fn invoke<T>(key: NonZeroU32, parameter: T) {
		let _ = key;
		let _ = parameter;
	}

	/// Indicates how exhausted the global callback registry is on a linear scale, with `0` indicating no or very low exhaustion and `255` indicating almost complete or complete exhaustion.
	#[must_use]
	pub fn registry_exhaustion() -> u8 {
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
	pub fn new(receiver: Pin<&'_ R>, handler: fn(receiver: *const R, parameter: T)) -> Self {
		callbacks::register(receiver, handler)
	}

	#[must_use]
	pub fn to_ref(&self) -> CallbackRef<ThreadSafe, T>
	where
		R: Sync,
	{
		CallbackRef {
			key: self.key,
			phantom: PhantomData,
		}
	}

	#[must_use]
	pub fn to_ref_thread_bound(&self) -> CallbackRef<ThreadBound, T> {
		CallbackRef {
			key: self.key,
			phantom: PhantomData,
		}
	}
}
impl<R, T> Drop for CallbackRegistration<R, T> {
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
	/// provided that the original [`CallbackReference`] hasn't been dropped yet.
	pub fn call(self, parameter: T) {
		callbacks::invoke(self.key, parameter)
	}
}

pub use callbacks::registry_exhaustion;
