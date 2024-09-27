//! # What's This?
//! bugi is a WASM plugin system for noda.
//!
//! # Features
//! * wasm plugin system (also can use any other system)
//! * multi threading support
//!
//! # Why not use Extism?
//! Because it lacks the following features.
//! * multi thread
//! * universe system
//!
//! These are necessary for noda (for speeding up and convenience).
//!
//! I also wanted to create my own.

pub mod module;
pub mod spec_type;
pub mod universe;
pub mod param;
pub mod cacher;
pub mod hostfunc;