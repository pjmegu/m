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

pub(crate) mod cacher;
pub(crate) mod hostfunc;
pub(crate) mod module;
pub(crate) mod param;
pub(crate) mod spec_type;
pub(crate) mod universe;

pub use cacher::*;
pub use module::*;
pub use param::*;
pub use universe::PluginUniverse;
