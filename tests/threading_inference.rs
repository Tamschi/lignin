#![allow(dead_code)]

use lignin::{Align, AutoNode, Deanonymize, Node};

fn threadsafe_empty() -> impl AutoNode<'static> {
	Node::Multi(&[]).prefer_threadsafe()
}

fn thread_unsafe_empty() -> impl AutoNode<'static> {
	Node::<*const ()>::Multi(&[]).prefer_threadsafe()
}

fn infer_threadsafe() -> impl AutoNode<'static> {
	Node::Multi(vec![threadsafe_empty().deanonymize()].leak())
}

fn infer_thread_unsafe() -> impl AutoNode<'static> {
	Node::Multi(vec![thread_unsafe_empty().deanonymize()].leak())
}

fn forward_threadsafe() -> impl AutoNode<'static> {
	threadsafe_empty().deanonymize()
}

fn forward_thread_unsafe() -> impl AutoNode<'static> {
	thread_unsafe_empty().deanonymize()
}

fn safe() -> impl AutoNode<'static> {
	Node::Multi(
		vec![
			threadsafe_empty().deanonymize().align(),
			threadsafe_empty().deanonymize().align(),
		]
		.leak(),
	)
	.prefer_threadsafe()
}

fn unsafe_() -> impl AutoNode<'static> {
	Node::Multi(
		vec![
			thread_unsafe_empty().deanonymize().align(),
			thread_unsafe_empty().deanonymize().align(),
		]
		.leak(),
	)
	.prefer_threadsafe()
}

fn mixed() -> impl AutoNode<'static> {
	Node::Multi(
		vec![
			threadsafe_empty().deanonymize().align(),
			thread_unsafe_empty().deanonymize().align(),
		]
		.leak(),
	)
}
