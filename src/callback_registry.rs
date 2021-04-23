//! Callback registry plumbing, for renderers and app runners that support them **and** need to run indefinitely.
//!
//! When not using this module directly, apps, if they enable the `"callbacks"` feature, run out of unique callback IDs after more than four billion total [`CallbackRegistration::new`] calls across all threads in a single run of the program.
//! As such, you *probably* don't need to access this module, but if you do then it's available.
#![allow(clippy::inline_always)] // Most functions here are either extremely simple or proxies to the inner module.

use crate::{sealed::Sealed, web, DomRef, ThreadBound, ThreadSafe, ThreadSafety};
use core::{
	fmt::Debug,
	marker::{PhantomData, PhantomPinned},
	mem,
	num::NonZeroU32,
	pin::Pin,
};

/// Indicates whether the `"callbacks"` feature is enabled.
pub const ENABLED: bool = cfg!(feature = "callback");

/// Canonically located at `callback_registry::if_callbacks`.  
/// Identity iff the `"callbacks"` feature is enabled, otherwise empty output.  
/// In most cases, prefer using the [`ENABLED`] constant to always check all of your code.
#[cfg(feature = "callbacks")]
#[macro_export]
macro_rules! if_callbacks {
	{$($tt:tt)*} => {$($tt)*}
}

/// Canonically located at `callback_registry::if_callbacks`.  
/// Identity iff the `"callbacks"` feature is enabled, otherwise empty output.  
/// In most cases, prefer using the [`ENABLED`] constant to always check all of your code.
#[cfg(not(feature = "callbacks"))]
#[macro_export]
macro_rules! if_callbacks {
	{$($tt:tt)*} => {}
}

#[doc(inline)]
pub use if_callbacks;

/// Canonically located at `callback_registry::if_not_callbacks`.  
/// Identity iff the `"callbacks"` feature is **not** enabled, otherwise empty output.  
/// In most cases, prefer using the [`ENABLED`] constant to always check all of your code.
#[cfg(feature = "callbacks")]
#[macro_export]
macro_rules! if_not_callbacks {
	{$($tt:tt)*} => {}
}

/// Canonically located at `callback_registry::if_not_callbacks`.  
/// Identity iff the `"callbacks"` feature is **not** enabled, otherwise empty output.  
/// In most cases, prefer using the [`ENABLED`] constant to always check all of your code.
#[cfg(not(feature = "callbacks"))]
#[macro_export]
macro_rules! if_not_callbacks {
	{$($tt:tt)*} => {$($tt)*}
}

#[doc(inline)]
pub use if_not_callbacks;

#[cfg(feature = "callbacks")]
mod callbacks_on {
	extern crate std;

	use crate::DomRef;

	use super::{CallbackRegistration, CallbackSignature};
	use core::{
		cell::Cell,
		convert::TryInto,
		marker::{PhantomData, PhantomPinned},
		mem,
		num::NonZeroU32,
		pin::Pin,
	};
	use lazy_static::lazy_static;
	use mem::size_of_val;
	use std::{
		boxed::Box,
		collections::{HashMap, VecDeque},
		panic::{catch_unwind, AssertUnwindSafe},
		result::Result::{Err, Ok},
		sync::RwLock,
	};

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
	) -> CallbackRegistration<R, fn(T)>
	where
		fn(T): CallbackSignature,
	{
		let mut registry = REGISTRY.write().unwrap();
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
			let key = NonZeroU32::new(registry.key_count).unwrap();
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

	#[must_use]
	pub fn register_by_ref<R, T>(
		receiver: Pin<&'_ R>,
		handler: fn(*const R, DomRef<&'_ T>),
	) -> CallbackRegistration<R, fn(DomRef<&'_ T>)>
	where
		fn(DomRef<&'_ T>): CallbackSignature,
	{
		let mut registry = REGISTRY.write().unwrap();
		if registry.key_count == u32::MAX {
			drop(registry);
			panic!("[lignin] Callback registry keys exhausted")
		} else {
			fn invoke_typed<R, T>(
				receiver_address: usize,
				handler_address: usize,
				parameter: DomRef<&'_ T>,
			) {
				let receiver = receiver_address as *const R;
				let handler = unsafe {
					// SAFETY: The pointer to invoke_typed is taken with matching monomorphization just below.
					mem::transmute::<usize, fn(*const R, DomRef<&'_ T>)>(handler_address)
				};
				handler(receiver, parameter)
			}

			registry.key_count += 1;
			let key = NonZeroU32::new(registry.key_count).unwrap();
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

	pub fn deregister<R, C>(registration: &CallbackRegistration<R, C>)
	where
		C: CallbackSignature,
	{
		REGISTRY
			.write()
			.unwrap()
			.entries
			.remove(&registration.key)
			.expect("`CallbackRegistration` double-drop");
	}

	pub fn invoke<T>(key: NonZeroU32, parameter: T)
	where
		fn(T): CallbackSignature,
	{
		CONTINUATION_QUEUE.with(|continuation_queue| {
			let none = continuation_queue.replace(Some(VecDeque::new()));
			debug_assert!(none.is_none());

			// UNWIND SAFETY: The only part we examine is the continuation queue,
			// and we don't run consumer code while holding a reference to it.
			match catch_unwind(AssertUnwindSafe(|| {
				let registry = REGISTRY.read().unwrap();
				if let Some(entry) = registry.entries.get(&key) {
					let invoke_typed = unsafe {
						// SAFETY: Same type as above.
						mem::transmute::<usize, fn(usize, usize, T)>(entry.invoke_typed_address)
					};
					invoke_typed(entry.receiver_address, entry.handler_address, parameter)
				}
			})) {
				Ok(()) => {
					for continuation in continuation_queue.take().unwrap() {
						continuation()
					}
				}
				Err(panic) => {
					continuation_queue.take(); // Drop continuations.
					std::panic::resume_unwind(panic)
				}
			}
		})
	}

	pub fn invoke_with_ref<T>(key: NonZeroU32, parameter: DomRef<&T>)
	where
		fn(DomRef<&'_ T>): CallbackSignature,
	{
		let registry = REGISTRY.read().unwrap();
		if let Some(entry) = registry.entries.get(&key) {
			let invoke_typed = unsafe {
				// SAFETY: Pretty much same type as above, just specified.
				mem::transmute::<usize, fn(usize, usize, DomRef<&'_ T>)>(entry.invoke_typed_address)
			};
			invoke_typed(entry.receiver_address, entry.handler_address, parameter)
		}
	}

	#[must_use]
	pub fn registry_exhaustion() -> u8 {
		let registry = REGISTRY.read().unwrap();
		(registry.key_count >> ((size_of_val(&registry.key_count) - 1) * 8))
			.try_into()
			.unwrap()
	}

	#[allow(clippy::result_unit_err)]
	pub unsafe fn reset_callback_registry() -> Result<(), ()> {
		let mut registry = REGISTRY.write().unwrap();
		#[allow(clippy::option_if_let_else)]
		if let Some(highest) = registry.entries.keys().max() {
			registry.key_count = highest.get();
			Err(())
		} else {
			registry.key_count = 0;
			Ok(())
		}
	}

	pub unsafe fn yet_more_unsafe_force_clear_callback_registry() {
		let mut registry = REGISTRY.write().unwrap();
		registry.entries.clear();
		registry.key_count = 0;
	}

	pub fn when_unlocked_locally<F: 'static + FnOnce()>(continuation: F) {
		CONTINUATION_QUEUE.with(|continuation_queue| {
			match unsafe {
				// SAFETY: All access is thread-local and not recursive.
				&mut *continuation_queue.as_ptr()
			} {
				Some(queue) => queue.push_back(Box::new(continuation)),
				None => continuation(),
			}
		})
	}

	std::thread_local! {
		#[allow(clippy::type_complexity)]
		static CONTINUATION_QUEUE: Cell<Option<VecDeque<Box<dyn FnOnce()>>>> = None.into();
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

	use crate::DomRef;

	use super::{CallbackRegistration, CallbackSignature};

	#[inline(always)]
	#[must_use]
	pub fn register<R, T>(
		receiver: Pin<&'_ R>,
		handler: fn(*const R, T),
	) -> CallbackRegistration<R, fn(T)>
	where
		fn(T): CallbackSignature,
	{
		let _ = receiver;
		let _ = handler;
		CallbackRegistration {
			key: NonZeroU32::new(u32::MAX).unwrap(),
			phantom: PhantomData,
			pinned: PhantomPinned,
		}
	}

	#[inline(always)]
	#[must_use]
	pub fn register_by_ref<R, T>(
		receiver: Pin<&'_ R>,
		handler: fn(*const R, DomRef<&'_ T>),
	) -> CallbackRegistration<R, fn(DomRef<&'_ T>)>
	where
		fn(DomRef<&'_ T>): CallbackSignature,
	{
		let _ = receiver;
		let _ = handler;
		CallbackRegistration {
			key: NonZeroU32::new(u32::MAX).unwrap(),
			phantom: PhantomData,
			pinned: PhantomPinned,
		}
	}

	#[inline(always)]
	pub fn deregister<R, C>(registration: &CallbackRegistration<R, C>)
	where
		C: CallbackSignature,
	{
		let _ = registration;
	}

	#[inline(always)]
	pub fn invoke<T>(key: NonZeroU32, parameter: T) {
		let _ = key;
		let _ = parameter;
	}

	#[inline(always)]
	pub fn invoke_with_ref<T>(key: NonZeroU32, parameter: DomRef<&T>) {
		let _ = key;
		let _ = parameter;
	}

	#[inline(always)]
	#[must_use]
	pub const fn registry_exhaustion() -> u8 {
		0
	}

	#[allow(clippy::result_unit_err)]
	#[inline(always)]
	pub unsafe fn reset_callback_registry() -> Result<(), ()> {
		Ok(())
	}

	#[inline(always)]
	pub unsafe fn yet_more_unsafe_force_clear_callback_registry() {}

	#[inline(always)]
	pub fn when_unlocked_locally<F: FnOnce()>(continuation: F) {
		continuation()
	}
}

#[cfg(feature = "callbacks")]
use callbacks_on as callbacks;

#[cfg(not(feature = "callbacks"))]
use callbacks_off as callbacks;

/// A callback registration handle that should be held onto by the matching receiver `R` or a container with [pin-projection](https://doc.rust-lang.org/stable/core/pin/index.html#pinning-is-structural-for-field) towards that value.
///
/// [`CallbackRegistration`] is [`!Unpin`](`Unpin`) for convenience: A receiver correctly becomes [`!Unpin`](`Unpin`) if it contains for example a `Cell<Option<CallbackRegistration<R, T>>`¹⁻², which can be conveniently initialized in a rendering function called with [`Pin<&…>`](`Pin`) argument.
///
/// To hold onto a [`CallbackRegistration`] without boxing or pinning, use a newtype wrapper with explicit [`Unpin`] implementation.
///
/// - - -
///
/// 1. [`impl<T: ?Sized> Unpin for Cell<T> where T: Unpin`](`core::cell::Cell`#impl-Unpin)
/// 2. [`impl<T> Unpin for Option<T> where T: Unpin`](`core::option::Option`#impl-Unpin)
///
/// # Safety Notes
///
/// When storing [`CallbackRegistration`]s inside a receiver, care must be taken that these fields are dropped first, before any other component state is invalidated.
/// [This should in most cases be done by placing them in the first fields of the receiver data structure.](https://doc.rust-lang.org/reference/destructors.html)
///
/// Code generators that must run more complex code but also want to avoid any overhead from clearing an [`Option`] [can make use of `ManuallyDrop`](`::core::mem::ManuallyDrop`#manuallydrop-and-drop-order).
/// However, this is somewhat more tricky to get right and much less easy to read, so I wouldn't recommend it for one-off code.
///
/// Double-dropping a [`CallbackRegistration`] will lead to, *at best*, a panic, but retrieving a [`CallbackRef`] from a dropped [`CallbackRegistration`] is guaranteed be sound.
/// Such a [`CallbackRef`] will never lead to a handler invocation unless the callback registry is [reset](`reset_callback_registry`).
///
/// ## `receiver` pointer
///
/// It is impossible to soundly derive a `&mut R` from the `*const R`, as this pointer was originally derived from a shared reference.
///
/// Similarly, no `&mut R` for the given instance may be created *or mutably re-borrowed* elsewhere in the program, by whatever means,
/// between the calls to any [`CallbackRegistration::new`] and *dereferencing* `receiver` in `handler`,
/// as doing so would invalidate all sibling references and pointers.
/// (Avoiding this situation dynamically is sound.)
///
/// To still update values, use [atomics](https://doc.rust-lang.org/stable/core/sync/atomic/index.html),
/// [cells](https://doc.rust-lang.org/stable/core/cell/index.html)
/// or, if necessary, [critical sections](https://doc.rust-lang.org/stable/std/sync/index.html#higher-level-synchronization-objects).
#[allow(clippy::type_complexity)]
#[derive(Debug)]
pub struct CallbackRegistration<R, C>
where
	C: CallbackSignature,
{
	key: NonZeroU32,
	///FIXME: Can this be written with `&R` (removing the manual `Send` and `Sync` impls below)?
	phantom: PhantomData<(*const R, C)>,
	pinned: PhantomPinned,
}
// SAFETY: `CallbackRegistration<R, C>` only refers to a `*const R`, so it acts like `&R` for thread-safety.
//
// Without the `"callbacks"` feature, that pointer is actually unreachable, so this type *could* be more generally `Send` and `Sync`.
// However, since a `CallbackRegistration` is intended to be primarily handled by the matching `R` instance, this isn't done in order to retain consistency.
unsafe impl<R, C> Send for CallbackRegistration<R, C>
where
	R: Sync,
	C: CallbackSignature,
{
}
unsafe impl<R, C> Sync for CallbackRegistration<R, C>
where
	R: Sync,
	C: CallbackSignature,
{
}

/// Separate `impl`s due to Rust language limitation. See [`CallbackSignature`] and expect future broadening.
impl<R> CallbackRegistration<R, fn(event: web::Event)> {
	/// Creates a new [`CallbackRegistration<R, T>`] with the given `receiver` and `handler`.
	///
	/// # Deadlocks / Panics
	///
	/// Creating or dropping **any** [`CallbackRegistration`] from within `handler` **may** deadlock or panic.
	///
	/// > This happens due to read-to-write re-entrance of the single internal callback registry [`RwLock`](https://doc.rust-lang.org/stable/std/sync/struct.RwLock.html), but this constraint may be relaxed somewhat in the future.
	/// >
	/// > File an [issue](https://github.com/Tamschi/lignin/issues) or open a [discussion](https://github.com/Tamschi/lignin/discussions) with your use case if you would benefit from that, so that I can better prioritize.
	///
	/// Use [`callback_registry::when_unlocked_locally`](`when_unlocked_locally`) to defer any such operations where necessary.
	///
	/// # Safety
	///
	/// **The `receiver` pointer given to `handler` may dangle unless `receiver` remains pinned until the created [`CallbackRegistration`] is dropped.**
	///
	/// You can ensure this most easily by storing the latter in for example a `Cell<Option<CallbackRegistration>>` embedded in the `receiver`.
	///
	/// Dropping the [`CallbackRegistration`] instance prevents any further calls to `handler` derived from it from running, blocking until this can be guaranteed.
	#[inline(always)] // Proxy function.
	#[must_use]
	pub fn new(receiver: Pin<&'_ R>, handler: fn(receiver: *const R, event: web::Event)) -> Self {
		callbacks::register(receiver, handler)
	}
}
/// Separate `impl`s due to Rust language limitation. See [`CallbackSignature`] and expect future broadening.
impl<R, T> CallbackRegistration<R, fn(dom_ref: DomRef<&'_ T>)> {
	/// Creates a new [`CallbackRegistration<R, T>`] with the given `receiver` and `handler`.
	///
	/// # Deadlocks / Panics
	///
	/// Creating or dropping **any** [`CallbackRegistration`] from within `handler` **may** deadlock or panic.
	///
	/// > This happens due to read-to-write re-entrance of the single internal callback registry [`RwLock`](https://doc.rust-lang.org/stable/std/sync/struct.RwLock.html), but this constraint may be relaxed somewhat in the future.
	/// >
	/// > File an [issue](https://github.com/Tamschi/lignin/issues) or open a [discussion](https://github.com/Tamschi/lignin/discussions) with your use case if you would benefit from that, so that I can better prioritize.
	///
	/// Use [`callback_registry::when_unlocked_locally`](`when_unlocked_locally`) to defer any such operations where necessary.
	///
	/// # Safety
	///
	/// **The `receiver` pointer given to `handler` may dangle unless `receiver` remains pinned until the created [`CallbackRegistration`] is dropped.**
	///
	/// You can ensure this most easily by storing the latter in for example a `Cell<Option<CallbackRegistration>>` embedded in the `receiver`.
	///
	/// Dropping the [`CallbackRegistration`] instance prevents any further calls to `handler` derived from it from running, blocking until this can be guaranteed.
	#[inline(always)] // Proxy function.
	#[must_use]
	pub fn new(
		receiver: Pin<&'_ R>,
		handler: fn(receiver: *const R, dom_ref: DomRef<&'_ T>),
	) -> Self {
		callbacks::register_by_ref(receiver, handler)
	}
}
#[allow(clippy::inline_always)] // All functions are very simple.
impl<R, C> CallbackRegistration<R, C>
where
	C: CallbackSignature,
{
	/// Creates a [`ThreadBound`] [`CallbackRef`] from this [`CallbackRegistration`].
	#[inline(always)]
	#[must_use]
	pub fn to_ref_thread_bound(&self) -> CallbackRef<ThreadBound, C> {
		CallbackRef {
			key: self.key,
			phantom: PhantomData,
		}
	}
}
impl<R, C> CallbackRegistration<R, C>
where
	R: Sync,
	C: CallbackSignature,
{
	// Using a separate `impl` block instead of a `where` clause on the method means it outright doesn't exist if `R: !Sync`.
	// This lets it be resolved on the trait instead even without qualification.

	/// Creates a [`ThreadSafe`] [`CallbackRef`] from this [`CallbackRegistration`].
	///
	/// > If you are developing a macro framework with [`ThreadSafety`] inference, see [`ToRefThreadBoundFallback`] for a way to overload this method appropriately.  
	/// > (See also the warning there.)
	/// >
	/// > For handwritten code or generated code with stricter thread-safety, please use [`.to_ref_thread_bound()`](`Self::to_ref_thread_bound`) instead whenever possible.
	#[allow(clippy::inline_always)]
	#[inline(always)] // Basically just a deref-copy.
	#[must_use]
	pub fn to_ref(&self) -> CallbackRef<ThreadSafe, C> {
		CallbackRef {
			key: self.key,
			phantom: PhantomData,
		}
	}

	/// Destroys a [`CallbackRegistration`] instance without running its destructor.
	///
	/// # Safety
	///
	/// Calling this method is technically always sound due to the soundness requirements on [`CallbackRegistration::new`].
	///
	/// It is still marked as `unsafe` since it has far-reaching implications regarding the validity guarantees of the `receiver` pointers given to [`CallbackRegistration::new`]'s `handler` parameter.
	pub unsafe fn leak(self) {
		mem::forget(self)
	}
}
/// Provides a fallback alternative implementation to [`CallbackRegistration::to_ref`] for use in macro frameworks.
///
/// There is no limitation on the receiver's [`Sync`]ness, but in turn the resulting [`CallbackRef`] is [`ThreadBound`].
///
/// > **Warning:** Using this trait can unhelpfully mask the source of [`ThreadBound`] in a larger application.
/// >
/// > If your framework supports optional [`Sync`]ness annotations, consider requiring them on originally thread-bound components.
pub trait ToRefThreadBoundFallback<C>: Sealed + Sized
where
	C: CallbackSignature,
{
	/// See [`CallbackRegistration::to_ref`], except that this method is unconstrained and that the resulting [`CallbackRef`] is [`ThreadBound`].
	fn to_ref(&self) -> CallbackRef<ThreadBound, C>;
}
impl<R, C> ToRefThreadBoundFallback<C> for CallbackRegistration<R, C>
where
	C: CallbackSignature,
{
	#[allow(clippy::inline_always)]
	#[inline(always)] // Proxy function.
	fn to_ref(&self) -> CallbackRef<ThreadBound, C> {
		self.to_ref_thread_bound()
	}
}
impl<R, C> Drop for CallbackRegistration<R, C>
where
	C: CallbackSignature,
{
	#[allow(clippy::inline_always)]
	#[inline(always)] // Proxy function.
	fn drop(&mut self) {
		callbacks::deregister(self)
	}
}

/// [`Vdom`](`crate::Vdom`) A callback reference linked to a [`CallbackRegistration`].
pub struct CallbackRef<S, C>
where
	S: ThreadSafety,
	C: CallbackSignature,
{
	//SAFETY: This type must be unchanged after a roundtrip through JavaScript via the `CallbackRef::into_js` and `CallbackRef::from_js` methods.
	pub(crate) key: NonZeroU32,
	phantom: PhantomData<(S, C)>,
}
/// Separate `impl`s due to Rust language limitation. See [`CallbackSignature`] and expect future broadening.
impl<S> CallbackRef<S, fn(event: web::Event)>
where
	S: ThreadSafety,
{
	/// Invokes the stored handler with the stored receiver and `parameter`,
	/// provided that the original [`CallbackRegistration`] hasn't been dropped yet.
	#[allow(clippy::inline_always)]
	#[inline(always)] // Proxy function.
	pub fn call(self, parameter: web::Event) {
		// `parameter` is name-matched between implementations, to still allow later unification if Rust gains named parameters.
		callbacks::invoke(self.key, parameter)
	}
}
/// Separate `impl`s due to Rust language limitation. See [`CallbackSignature`] and expect future broadening.
impl<S, T> CallbackRef<S, fn(dom_ref: DomRef<&'_ T>)>
where
	S: ThreadSafety,
{
	/// Invokes the stored handler with the stored receiver and `parameter`,
	/// provided that the original [`CallbackRegistration`] hasn't been dropped yet.
	#[allow(clippy::inline_always)]
	#[inline(always)] // Proxy function.
	pub fn call(self, parameter: DomRef<&T>) {
		// `parameter` is name-matched between implementations, to still allow later unification if Rust gains named parameters.
		callbacks::invoke_with_ref(self.key, parameter)
	}
}

/// Indicates how exhausted the global callback registry is on a linear scale, with `0` indicating no or very low exhaustion and `255` indicating almost complete or complete exhaustion.
#[allow(clippy::inline_always)]
#[inline(always)] // Proxy function.
#[must_use]
pub fn registry_exhaustion() -> u8 {
	callbacks::registry_exhaustion()
}

/// These functions are intended as storage optimization for in-browser renderers.
///
/// The [`CallbackRef`]'s raw numerical value can be passed through JavaScript directly,
/// rather than adding another layer of indirection.
///
/// Only available with the `"callbacks"` feature.
///
/// > Most DOM renderers will still require an [***event listener***](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget/addEventListener)
/// > table in order to unsubscribe from events.
///
/// # Example
///
/// ```rust, no_run
/// use js_sys::Function;
/// use lignin::{CallbackRef, ThreadBound, web};
/// use wasm_bindgen::{closure::Closure, JsCast, JsValue, UnwrapThrowExt};
///
/// let element: web_sys::Element = // …
/// # unreachable!();
/// let callback_ref: CallbackRef<ThreadBound, fn(web::Event)> = // …
/// # unreachable!();
///
/// let common_handler = Closure::<dyn Fn(JsValue, web_sys::Event)>::wrap(
///   Box::new(|callback_ref: JsValue, event: web_sys::Event| {
///     unsafe { CallbackRef::<ThreadBound, fn(web::Event)>::from_js(&callback_ref) }
///     .expect_throw("Invalid `CallbackRef`.")
///     .call(event.into());
///   })
/// );
///
/// let listener = common_handler.as_ref().unchecked_ref::<Function>()
///   .bind1(&JsValue::UNDEFINED, &callback_ref.into_js());
///
/// // `common_handler` must be either leaked or stored somewhere,
/// // since otherwise it will throw when called from JavaScript.
///
/// let result = element.add_event_listener_with_callback("click", &listener);
/// ```
#[cfg(feature = "callbacks")]
impl<S, C> CallbackRef<S, C>
where
	S: ThreadSafety,
	C: ?Sized + CallbackSignature,
{
	/// Returns this [`CallbackRef`]'s identity as [`JsValue`](`wasm_bindgen::JsValue`),
	/// which can then for example be [bound](https://docs.rs/js-sys/0.3/js_sys/struct.Function.html#method.bind1)
	/// to a generic event handler shim.
	///
	/// # Implementation Contract
	///
	/// The return value of this function must be treated as opaque handle.
	#[must_use]
	pub fn into_js(self) -> wasm_bindgen::JsValue {
		let key: f64 = self.key.get().into();
		debug_assert_eq!(unsafe { key.to_int_unchecked::<u32>() }, self.key.get());
		wasm_bindgen::JsValue::from_f64(key)
	}

	/// Reconstructs a [`CallbackRef`] that was previously converted into a [`JsValue`](`wasm_bindgen::JsValue`).
	///
	/// # Safety
	///
	/// - `key` must have been retrieved verbatim from [`.into_js`],
	/// - `S` must be compatible (i.e. the same or [`ThreadBound`]),
	/// - if the original `S` was [`ThreadBound`], the [`CallbackRef`] must be reconstructed on the same thread and
	/// - `C` must be the same except for lifetime changes that would be okay in an assignment.
	#[must_use]
	pub unsafe fn from_js(key: &wasm_bindgen::JsValue) -> Option<Self> {
		let key = key.as_f64()?;

		#[allow(clippy::float_cmp)]
		if key.trunc() != key || key > u32::MAX.into() || key < 1.0 {
			None
		} else {
			Some(Self {
				key: NonZeroU32::new(key.to_int_unchecked())?,
				phantom: PhantomData,
			})
		}
	}
}

#[cfg(test)]
#[test]
fn assert_no_quantization() {
	// Assert field type.
	let _ = CallbackRef::<ThreadSafe, fn(web::Event)> {
		key: NonZeroU32::new(u32::MAX).unwrap(),
		phantom: PhantomData,
	};

	for x in u32::MAX - 1000..u32::MAX {
		let f: f64 = x.into();
		assert_eq!(unsafe { f.to_int_unchecked::<u32>() }, x);
	}
}

/// Tries to rewind the total callback registration counter to zero.
///
/// # Errors
///
/// Should that fail (because there are still callbacks registered), the counter is instead set to the lowest value that ensures no colliding [`CallbackRegistration`].
///
/// # Safety
///
/// The caller (generally a renderer) must ensure that no currently existing [`CallbackRef`]s created from a dropped [`CallbackRegistration`] can have their [`.call(…)`](`CallbackRef::call`) function invoked during or after this call.
///
/// # See Also
///
/// [`yet_more_unsafe_force_clear_callback_registry`], which yet-more-unsafely ignores [`CallbackRegistration`]s that haven't been dropped yet.
#[allow(clippy::inline_always)]
#[allow(clippy::module_name_repetitions)]
#[allow(clippy::result_unit_err)]
#[inline(always)] // Proxy function.
pub unsafe fn reset_callback_registry() -> Result<(), ()> {
	callbacks::reset_callback_registry()
}

/// Clears the callback registry entirely and resets the total callback registration counter to zero.
///
/// # Safety
///
/// Like with [`reset_callback_registry()`], the caller must ensure that no currently existing [`CallbackRef`]s created from a dropped [`CallbackRegistration`] can have their [`.call(…)`](`CallbackRef::call`) function invoked during or after this call.
///
/// Additionally, the caller (usually an `unsafe` app runner) must ensure that no currently existing [`CallbackRegistration`] instances are ever dropped.
/// Failing this second condition doesn't quite cause undefined behavior, but can cause unrelated parts of the app to misbehave and the callback registry to become poisoned immediately or later.
///
/// Please be very careful when using this. Practically any code in the entire app can violate the soundness condition in a difficult to track down way.
#[allow(clippy::inline_always)]
#[allow(clippy::module_name_repetitions)]
#[inline(always)] // Proxy function.
pub unsafe fn yet_more_unsafe_force_clear_callback_registry() {
	callbacks::yet_more_unsafe_force_clear_callback_registry()
}

/// Marks function pointers for which callbacks are implemented.
///
/// > This not being a blanket implementation over [`fn(T)`](https://doc.rust-lang.org/stable/std/primitive.fn.html) is largely related to [Rust#56105](https://github.com/rust-lang/rust/issues/56105).
/// >
/// > In short, an `impl <T> CallbackSignature for fn(T) {}` currently does not cover for example `fn(web::DomRef<&'_ T>)`, but their collision will become a hard error in the future (as of March 2021/Rust 1.50.0).
pub trait CallbackSignature: Sealed + Sized + Copy {}
impl CallbackSignature for fn(event: web::Event) {}
impl<T> CallbackSignature for fn(dom_ref: web::DomRef<&'_ T>) {}

/// Causes a continuation to be called when the callback registry is not locked (anymore) by the current thread.
///
/// > **Warning:**
/// >
/// > Spawning a thread that calls this function and then joining on it from a callback handler is an easy path towards deadlocks, so please avoid doing that.
///
/// More specifically: This function has one of two effects, depending on whether it is called in scope of (and on the same thread as!) a callback managed by `lignin`:
///
/// - If **no** callback is currently running on this thread, the continuation is invoked immediately.
///
/// - If such a callback is currently running on the current thread, `continuation` is scheduled for later execution.
///
///   As soon as the registry becomes unlocked, all such scheduled continuations are run, *in order of their respective [`when_unlocked_locally`] calls*.
///
///   > The current implementation of this is somewhat inefficient and will always allocate.
///   >
///   > I have a more efficient scheduler in mind, but that particular model would require ~~[`set_ptr_value`](https://doc.rust-lang.org/stable/std/primitive.pointer.html#method.set_ptr_value-1)~~
///   > at least [`std::alloc::Allocator`](https://doc.rust-lang.org/stable/std/alloc/trait.Allocator.html)
///   > to be stabilized first in order to construct a consumable box pointing to an allocation arena.
///   >
///   > If you have a better suggestion that works on stable Rust, feel free to [send it my way](https://github.com/Tamschi/lignin/discussions/categories/ideas)
///   > (with permission to actually implement it here if it's extensive enough to warrant that)!
///
/// # Panic Notes
///
/// While this function won't panic by itself (except due to memory limits),
/// it's still possible that a callback handler panics while continuations are pending.
///
/// Should this happen, all pending continuations are dropped without being executed.
#[allow(clippy::inline_always)]
#[inline(always)] // Proxy function.
pub fn when_unlocked_locally(continuation: impl 'static + FnOnce()) {
	callbacks::when_unlocked_locally(continuation)
}
