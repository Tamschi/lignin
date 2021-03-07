use lignin::{
	callback_registry::ToRefThreadBoundFallback, web::Event, CallbackRef, CallbackRegistration,
	ThreadBound, ThreadSafe,
};

pub fn it_compiles_safe(
	registration: &CallbackRegistration<(), fn(Event)>,
) -> CallbackRef<ThreadSafe, fn(Event)> {
	registration.to_ref()
}

pub fn it_compiles_bound(
	registration: &CallbackRegistration<*mut (), fn(Event)>,
) -> CallbackRef<ThreadBound, fn(Event)> {
	registration.to_ref()
}
