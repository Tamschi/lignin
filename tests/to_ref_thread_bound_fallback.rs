use lignin::{
	callback_registry::ToRefThreadBoundFallback, web::Event, CallbackRef, CallbackRegistration,
	ThreadBound, ThreadSafe,
};

pub fn it_compiles_safe(
	registration: &CallbackRegistration<(), dyn Fn(Event)>,
) -> CallbackRef<ThreadSafe, dyn Fn(Event)> {
	registration.to_ref()
}

pub fn it_compiles_bound(
	registration: &CallbackRegistration<*mut (), dyn Fn(Event)>,
) -> CallbackRef<ThreadBound, dyn Fn(Event)> {
	registration.to_ref()
}
