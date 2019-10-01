#![feature(try_trait)]
#![cfg_attr(feature = "alloc", feature(alloc))]
extern crate alloc;

#[macro_use] extern crate failure;
#[macro_use] pub mod macros;
#[macro_use] extern crate magic_crypt;
#[macro_use] extern crate serde_json;

pub mod error;
pub mod hash;
pub mod random;
pub mod token;
