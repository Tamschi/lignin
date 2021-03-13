use lignin::{web::Event, CallbackRef, Element, EventBinding, Node, ThreadBound, ThreadSafe};
use static_assertions::{assert_eq_align, assert_eq_size};

assert_eq_align!(
	Element<'static, 'static, ThreadSafe>,
	Element<'static, 'static, ThreadBound>
);
assert_eq_size!(
	Element<'static, 'static, ThreadSafe>,
	Element<'static, 'static, ThreadBound>
);

assert_eq_align!(
	EventBinding<'static, 'static, ThreadSafe>,
	EventBinding<'static, 'static, ThreadBound>
);
assert_eq_size!(
	EventBinding<'static, 'static, ThreadSafe>,
	EventBinding<'static, 'static, ThreadBound>
);

assert_eq_align!(
	Node<'static, 'static, ThreadSafe>,
	Node<'static, 'static, ThreadBound>
);
assert_eq_size!(
	Node<'static, 'static, ThreadSafe>,
	Node<'static, 'static, ThreadBound>
);

assert_eq_align!(
	CallbackRef<ThreadSafe, dyn Fn(Event)>,
	CallbackRef<ThreadBound, dyn Fn(Event)>
);
assert_eq_size!(
	CallbackRef<ThreadSafe, dyn Fn(Event)>,
	CallbackRef<ThreadBound, dyn Fn(Event)>
);
