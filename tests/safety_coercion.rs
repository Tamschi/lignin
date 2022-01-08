use lignin::{auto_safety::Align, Node, ThreadBound, ThreadSafe};

pub fn __() {
	let thread_safe: Node<ThreadSafe> = Node::Multi(&[]);
	let _thread_bound: Node<ThreadBound> = thread_safe.align();
}

pub fn coerce_mine<'a>(node: &'a Node<ThreadSafe>) -> &'a Node<'a, ThreadBound> {
	node
}
pub fn coerce_box(r#box: &Box<ThreadSafe>) -> &Box<ThreadBound> {
	r#box
}
