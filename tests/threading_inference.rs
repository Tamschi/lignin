#![allow(dead_code)]

use lignin::{
	auto_safety::{Align, AutoSafe, Deanonymize},
	Node, ThreadBound,
};

fn thread_safe_empty() -> impl AutoSafe<Node<'static, ThreadBound>> {
	Node::Multi(&[]).prefer_thread_safe()
}

fn thread_bound_empty() -> impl AutoSafe<Node<'static, ThreadBound>> {
	Node::<ThreadBound>::Multi(&[]).prefer_thread_safe()
}

fn infer_thread_safe() -> impl AutoSafe<Node<'static, ThreadBound>> {
	Node::Multi(vec![thread_safe_empty().deanonymize()].leak())
}

fn infer_thread_bound() -> impl AutoSafe<Node<'static, ThreadBound>> {
	Node::Multi(vec![thread_bound_empty().deanonymize()].leak())
}

fn forward_thread_safe() -> impl AutoSafe<Node<'static, ThreadBound>> {
	thread_safe_empty().deanonymize()
}

fn forward_thread_bound() -> impl AutoSafe<Node<'static, ThreadBound>> {
	thread_bound_empty().deanonymize()
}

fn safe() -> impl AutoSafe<Node<'static, ThreadBound>> {
	Node::Multi(
		vec![
			thread_safe_empty().deanonymize().align(),
			thread_safe_empty().deanonymize().align(),
		]
		.leak(),
	)
	.prefer_thread_safe()
}

fn bound_() -> impl AutoSafe<Node<'static, ThreadBound>> {
	Node::Multi(
		vec![
			thread_bound_empty().deanonymize().align(),
			thread_bound_empty().deanonymize().align(),
		]
		.leak(),
	)
	.prefer_thread_safe()
}

fn mixed() -> impl AutoSafe<Node<'static, ThreadBound>> {
	Node::Multi(
		vec![
			thread_safe_empty().deanonymize().align(),
			thread_bound_empty().deanonymize().align(),
		]
		.leak(),
	)
}
