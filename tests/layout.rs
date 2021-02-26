use lignin::{CallbackRef, Element, EventBinding, Node, ThreadBound, ThreadSafe};
use static_assertions::{assert_eq_align, assert_eq_size};

assert_eq_align!(Element<'static, ThreadSafe>, Element<'static, ThreadBound>);
assert_eq_size!(Element<'static, ThreadSafe>, Element<'static, ThreadBound>);

assert_eq_align!(
	EventBinding<'static, ThreadSafe>,
	EventBinding<'static, ThreadBound>
);
assert_eq_size!(
	EventBinding<'static, ThreadSafe>,
	EventBinding<'static, ThreadBound>
);

assert_eq_align!(Node<'static, ThreadSafe>, Node<'static, ThreadBound>);
assert_eq_size!(Node<'static, ThreadSafe>, Node<'static, ThreadBound>);

assert_eq_align!(
	CallbackRef<(), ThreadSafe>,
	CallbackRef<(), ThreadBound>
);
assert_eq_size!(
	CallbackRef<(), ThreadSafe>,
	CallbackRef<(), ThreadBound>
);
