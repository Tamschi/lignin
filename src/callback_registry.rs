use core::{marker::PhantomData, num::NonZeroU64, pin::Pin};

#[cfg(feature = "callbacks")]
mod callbacks_on {
	extern crate std;

	use super::CallbackRegistration;
	use core::{marker::PhantomData, mem, num::NonZeroU64, pin::Pin};
	use lazy_static::lazy_static;
	use std::{collections::HashMap, sync::RwLock};

	lazy_static! {
		static ref REGISTRY: RwLock<Registry> = RwLock::default();
	}

	struct Registry {
		key_count: u64,
		entries: HashMap<NonZeroU64, Entry>,
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

	pub fn register<'a, R: 'a, T>(
		receiver: Pin<&'_ R>,
		handler: fn(*const R, T),
	) -> CallbackRegistration<'a, R, T> {
		let mut registry = REGISTRY.write().unwrap();
		if registry.key_count == u64::MAX {
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
			let key = NonZeroU64::new(registry.key_count).unwrap();
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
			}
		}
	}

	pub fn deregister<R, T>(registration: &CallbackRegistration<R, T>) {
		let removed = REGISTRY
			.write()
			.unwrap()
			.entries
			.remove(&registration.key)
			.is_some();
		assert!(removed)
	}

	pub fn invoke<T>(key: NonZeroU64, parameter: T) {
		let registry = REGISTRY.read().unwrap();
		if let Some(entry) = registry.entries.get(&key) {
			let invoke_typed = unsafe {
				// SAFETY: Same type as above.
				mem::transmute::<usize, fn(usize, usize, T)>(entry.invoke_typed_address)
			};
			invoke_typed(entry.receiver_address, entry.handler_address, parameter)
		}
	}
}

#[allow(dead_code)]
#[allow(clippy::let_underscore_drop)]
#[allow(clippy::needless_pass_by_value)]
mod callbacks_off {
	use core::{marker::PhantomData, num::NonZeroU64, pin::Pin};

	use super::CallbackRegistration;

	pub fn register<'a, R: 'a, T>(
		receiver: Pin<&'_ R>,
		handler: fn(*const R, T),
	) -> CallbackRegistration<'a, R, T> {
		let _ = receiver;
		let _ = handler;
		CallbackRegistration {
			key: NonZeroU64::new(u64::MAX).unwrap(),
			phantom: PhantomData::default(),
		}
	}

	pub fn deregister<R, T>(registration: &CallbackRegistration<R, T>) {
		let _ = registration;
	}

	pub fn invoke<T>(key: NonZeroU64, parameter: T) {
		let _ = key;
		let _ = parameter;
	}
}

#[cfg(feature = "callbacks")]
use callbacks_on as callbacks;

#[cfg(not(feature = "callbacks"))]
use callbacks_off as callbacks;

#[allow(clippy::type_complexity)]
#[derive(Debug)]
pub struct CallbackRegistration<'a, R, T> {
	key: NonZeroU64,
	phantom: PhantomData<(&'a R, fn(T))>,
}
impl<'a, R, T> CallbackRegistration<'a, R, T> {
	pub fn new(receiver: Pin<&'_ R>, handler: fn(*const R, T)) -> Self
	where
		R: 'a,
	{
		callbacks::register(receiver, handler)
	}

	#[must_use]
	pub fn to_ref(&self) -> CallbackRef<T> {
		self.into()
	}
}
impl<'a, R, T> Drop for CallbackRegistration<'a, R, T> {
	fn drop(&mut self) {
		callbacks::deregister(self)
	}
}

impl<'a, R, T> From<&CallbackRegistration<'a, R, T>> for CallbackRef<T> {
	fn from(registration: &CallbackRegistration<'a, R, T>) -> Self {
		Self {
			key: registration.key,
			phantom: PhantomData::default(),
		}
	}
}

#[allow(clippy::type_complexity)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CallbackRef<T> {
	key: NonZeroU64,
	phantom: PhantomData<(*const (), fn(T))>, // Not Send or Sync!
}
impl<T> Clone for CallbackRef<T> {
	fn clone(&self) -> Self {
		*self
	}
}
impl<T> Copy for CallbackRef<T> {}
impl<T> CallbackRef<T> {
	pub fn call(self, parameter: T) {
		callbacks::invoke(self.key, parameter)
	}
}
