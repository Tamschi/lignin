use lignin::CallbackRegistration;

#[test]
#[cfg_attr(not(feature = "callbacks"), ignore = "only with callbacks")]
fn distinct() {
	let receiver = Box::pin(());
	let a = CallbackRegistration::new(receiver.as_ref(), |_, ()| ());
	let b = CallbackRegistration::new(receiver.as_ref(), |_, ()| ());
	assert_ne!(a.to_ref(), b.to_ref());
	assert_ne!(a.to_ref_thread_bound(), b.to_ref_thread_bound());
}

#[test]
#[cfg_attr(feature = "callbacks", ignore = "only without callbacks")]
fn identical() {
	let receiver = Box::pin(());
	let a = CallbackRegistration::new(receiver.as_ref(), |_, ()| ());
	let b = CallbackRegistration::new(receiver.as_ref(), |_, ()| ());
	assert_eq!(a.to_ref(), b.to_ref());
	assert_eq!(a.to_ref_thread_bound(), b.to_ref_thread_bound());
}
