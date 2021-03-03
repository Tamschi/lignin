use lignin::{
	callback_registry::ToRefThreadBoundFallback, CallbackRef, CallbackRegistration, ThreadBound,
	ThreadSafe,
};

pub fn it_compiles_safe(
	registration: &CallbackRegistration<(), ()>,
) -> CallbackRef<ThreadSafe, ()> {
	registration.to_ref()
}

pub fn it_compiles_bound(
	registration: &CallbackRegistration<*mut (), ()>,
) -> CallbackRef<ThreadBound, ()> {
	registration.to_ref()
}
