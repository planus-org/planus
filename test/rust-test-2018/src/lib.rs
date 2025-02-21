#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

extern crate alloc;

#[macro_use]
pub mod macros;

pub mod planus_api;
pub mod planus_test;
