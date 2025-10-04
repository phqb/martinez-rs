use core::cmp::Reverse;
use std::collections::BinaryHeap;

pub(crate) type MinHeap<T> = BinaryHeap<Reverse<T>>;
