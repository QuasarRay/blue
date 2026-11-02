//! Rust implementation of Carbon Blue and BlueExposure on Bevy.
//! Public source counterparts retain their original spelling. Python adapters
//! and Bevy ownership infrastructure have separate implementation boundaries.

mod exposure;
mod runtime;
pub mod streams;

pub use runtime::{BlueClasses, BluePlugin, BlueRuntime, ClassRegistration, ObjectHandle};
pub use streams::{
    IBlueMemStream, IBlueStream, ICacheable, ICcpStream, IRoot, MemStream, SeekOrigin,
};
