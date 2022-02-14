//! Analogous to [`crate::auto_safety`], but for [`Guard`]s.
//!
//! > This is likely a better API in general and may replace the one in [`crate::auto_safety`] in future versions.

use crate::{auto_safety::Align, Guard, ThreadBound, ThreadSafe, ThreadSafety};
use core::mem;
use sealed::Sealed;

mod sealed {
	use super::{AutoSafe, Wrapper};
	use crate::ThreadSafety;

	pub trait Sealed {}
	impl<S: ThreadSafety> Sealed for Wrapper<'_, S> {}
	impl<'a, T> Sealed for &mut T where T: AutoSafe<'a> {}
}

pub(super) enum Wrapper<'a, S: ThreadSafety> {
	Present(Guard<'a, S>),
	Taken,
}
impl<'a, S: ThreadSafety> Wrapper<'a, S> {
	pub(super) fn new(guard: Guard<'a, S>) -> Self {
		Self::Present(guard)
	}

	#[track_caller]
	fn take(&mut self) -> Guard<'a, S> {
		match mem::replace(self, Self::Taken) {
			Wrapper::Present(guard) => guard,
			Wrapper::Taken => panic!("Tried to deanonymize `impl AutoGuard` twice. See `lignin::guard::auto_safety` for more information."),
		}
	}
}

/// Static thread safety smuggling through `impl AutoSafe` returns for [`Guard`] instances.
pub trait AutoSafe<'a>: Sealed {
	/// When specified in consumer code (in the `impl` return type), use the bound variant here.
	type BoundOrActual: 'a;

	/// Call this function as `AutoSafe::deanonymize(…)` on an `&mut &mut impl Autosafe<'a>` [yes, double-mut]
	/// to statically retrieve an instance with the actual type.
	///
	/// # Panics
	///
	/// Iff this function was called on this instance before.
	#[track_caller]
	fn deanonymize(this: &mut Self) -> Self::BoundOrActual;
}
impl<'a, S: ThreadSafety> AutoSafe<'a> for Wrapper<'a, S> {
	type BoundOrActual = Guard<'a, ThreadBound>;

	#[track_caller]
	fn deanonymize(this: &mut Self) -> Self::BoundOrActual {
		let mut guard: Guard<'a, S> = this.take();
		Guard {
			vdom: guard.vdom.align(),
			guarded: guard.guarded.take(),
		}
	}
}
impl<'a, T> AutoSafe<'a> for &mut T
where
	T: Send + Sync + AutoSafe<'a, BoundOrActual = Guard<'a, ThreadBound>>,
{
	type BoundOrActual = Guard<'a, ThreadSafe>;

	#[track_caller]
	fn deanonymize(this: &mut Self) -> Self::BoundOrActual {
		// A `TypeId` check would be better, but isn't possible here because `T` isn't `'static`.
		assert!(mem::size_of::<T>() == mem::size_of::<Wrapper<'a, ThreadSafe>>());
		unsafe { &mut *(*this as *mut T).cast::<Wrapper<'a, ThreadSafe>>() }.take()
	}
}

/// Mainly for use by frameworks. Canonically located at `guard::auto_safe::AutoSafe_alias`.  
/// Creates a custom-visibility alias for [`guard::auto_safety::AutoSafe`](`AutoSafe`).
///
/// See [`auto_safety`#limiting-autosafe-exposure](`crate::auto_safety`#limiting-autosafe-exposure) for more information.
#[macro_export]
macro_rules! guard_AutoSafe_alias {
	($vis:vis $Name:ident) => {
		/// An alias for [`$crate::auto_safety::AutoSafe`] with custom visibility.
		$vis trait $Name<'a>: $crate::guard::auto_safety::AutoSafe<'a> {
			/// When specified in consumer code (in the `impl` return type), use the bound variant here.
			type BoundOrActual: 'a;

			/// Call this function as `AutoSafe::deanonymize(…)` on an `&mut &mut impl Autosafe<'a>` [yes, double-mut]
			/// to statically retrieve an instance with the actual type.
			///
			/// # Panics
			///
			/// Iff this function was called on this instance before.
			#[track_caller]
			fn deanonymize(this: &mut Self) -> Self::BoundOrActual;
		}
		impl<'a, T $Name<'a> for T
		where
			T: $crate::guard::auto_safety::AutoSafe<'a>
		{
			type BoundOrActual = <T as $crate::guard::auto_safety::AutoSafe>>::BoundOrActual;

			#[track_caller]
			fn deanonymize(this: &mut Self) -> Self::BoundOrActual {
				<T as $crate::guard::auto_safety::AutoSafe>::deanonymize(this)
			}
		}
	};
}

pub use crate::guard_AutoSafe_alias as AutoSafe_alias;
