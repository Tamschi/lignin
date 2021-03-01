use lignin::{CallbackRegistration, Node, ThreadBound, ThreadSafe};
use static_assertions::{assert_impl_all, assert_not_impl_any};

assert_not_impl_any!(Node<ThreadBound>: Send, Sync);
assert_impl_all!(Node<ThreadSafe>: Send, Sync);

assert_not_impl_any!(CallbackRegistration<*const (), ()>: Send, Sync);
assert_impl_all!(CallbackRegistration<(), ()>: Send, Sync);
