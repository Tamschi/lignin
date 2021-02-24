use lignin::{CallbackRegistration, Node};
use static_assertions::{assert_impl_all, assert_not_impl_any};

assert_not_impl_any!(Node: Send, Sync);

assert_not_impl_any!(CallbackRegistration<*const (), ()>: Send, Sync);
assert_impl_all!(CallbackRegistration<(), ()>: Send, Sync);
