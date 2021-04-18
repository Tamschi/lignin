#![cfg(feature = "callbacks")]

use std::pin::Pin;

use lignin::{web::Event, CallbackRef, CallbackRegistration, ThreadSafe};
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn one() {
	let pinned = unsafe { Pin::new_unchecked(&()) };
	let registration = CallbackRegistration::<(), fn(Event)>::new(pinned, |_, _| {});

	let reference = registration.to_ref();
	let js_value = reference.into_js();
	let reference_2 = unsafe { CallbackRef::<ThreadSafe, fn(Event)>::from_js(&js_value) }.unwrap();
	assert_eq!(reference, reference_2);
}
