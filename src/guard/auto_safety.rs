//! Analogous to [`crate::auto_safety`], but for [`Guard`]s.
//!
//! > This is likely a better API in general and may replace the one in [`crate::auto_safety`] in future versions.

use crate::{Guard, ThreadBound, ThreadSafe, ThreadSafety};
use core::mem::{self, ManuallyDrop};
use sealed::Sealed;

mod sealed {
	use core::mem::ManuallyDrop;

	#[allow(deprecated)]
	use super::AutoSafe;
	use super::Bound;
	use crate::{Guard, ThreadSafety};

	pub trait Sealed {}
	#[allow(deprecated)]
	impl<S: ThreadSafety> Sealed for Guard<'_, S> {}
	impl<'a, T> Sealed for &T where T: AutoSafe {}
	impl<'a, B: Bound, T: 'a> Sealed for ManuallyDrop<T> where T: AutoSafe<Bound = B> {}
	impl<'a, B: Bound, T: 'a> Sealed for &ManuallyDrop<T> where T: Send + Sync + AutoSafe<Bound = B> {}
}

/// Static thread safety smuggling through `impl AutoSafe` returns for [`Guard`] instances.
pub trait AutoSafe: Sealed + Sized {
	/// When specified in consumer code (in the `impl` return type), use the bound variant here.
	type Bound: Bound;
}

impl<'a, S: ThreadSafety> AutoSafe for Guard<'a, S> {
	type Bound = Guard<'a, ThreadBound>;
}

/// Marks a thread-bound type and allows access to the matching thread-safe type.
pub trait Bound: Sealed {
	/// The matching thread-safe type.
	type Safe;
}

impl<'a> Bound for Guard<'a, ThreadBound> {
	type Safe = Guard<'a, ThreadSafe>;
}

/// Provides the [`::deanonymize`](`Deanonymize::deanonymize`) function.
pub trait Deanonymize: Sealed + Sized {
	/// When accessed correctly, the actual type of the instance.
	type Actual;

	/// Call this function as `AutoSafe::deanonymize(…)` on an `&&impl Autosafe<'a>` [yes, double-ref]
	/// to statically retrieve an instance with the actual type.
	///
	/// # Safety
	///
	/// The instance passed to this function by reference must not be used afterwards.
	///
	/// # Panics
	///
	/// Iff this function was called on this instance before.
	#[track_caller]
	unsafe fn deanonymize(this: &Self) -> Self::Actual;
}

impl<'a, B: Bound, T: 'a> Deanonymize for &ManuallyDrop<T>
where
	T: Send + Sync + AutoSafe<Bound = B>,
{
	type Actual = B::Safe;

	#[track_caller]
	#[allow(deprecated)]
	unsafe fn deanonymize(this: &Self) -> Self::Actual {
		// A `TypeId` check would be better, but isn't possible here because `T` isn't `'static`.
		assert!(mem::size_of::<T>() == mem::size_of::<B::Safe>());
		(*this as *const ManuallyDrop<T>)
			.cast::<Self::Actual>()
			.read()
	}
}

impl<'a, B: Bound, T: 'a> Deanonymize for ManuallyDrop<T>
where
	T: AutoSafe<Bound = B>,
{
	type Actual = B;

	#[track_caller]
	#[allow(deprecated)]
	unsafe fn deanonymize(this: &Self) -> Self::Actual {
		// A `TypeId` check would be better, but isn't possible here because `T` isn't `'static`.
		assert!(mem::size_of::<T>() == mem::size_of::<B>());
		(this as *const ManuallyDrop<T>)
			.cast::<Self::Actual>()
			.read()
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
		$vis trait $Name: $crate::guard::auto_safety::AutoSafe<Bound = <Self as $Name>::Bound> {
			type Bound: $crate::guard::auto_safety::Bound;
		}
		impl<T> $Name for T
		where
			T: $crate::guard::auto_safety::AutoSafe
		{
			type Bound = <T as $crate::guard::auto_safety::AutoSafe>::Bound;
		}
	};
}

pub use crate::guard_AutoSafe_alias as AutoSafe_alias;
