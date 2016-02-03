#![feature(const_fn)]
#![feature(plugin)]

#![plugin(stainless)]
#![plugin(clippy)]

// for stainless before_each
#![allow(unused_mut, unused_variables)]

extern crate concrust;
#[macro_use(expect)]
extern crate expectest;

mod test_primitives;
mod test_array_queue;
mod test_linked_queue;
mod test_maps;
