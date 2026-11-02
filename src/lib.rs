//! Rust implementation of Carbon Blue and BlueExposure on Bevy.
//! Public source counterparts retain their original spelling. Python adapters
//! and Bevy ownership infrastructure have separate implementation boundaries.

mod exposure;
mod os;
pub mod python_runtime;
mod runtime;
pub mod streams;

pub use exposure::events::BlueTestEvents;
pub use exposure::wrapper::{BlueWrapper, PythonKlass};
pub use os::BlueOS;
pub use runtime::{BlueClasses, BluePlugin, BlueRuntime, ClassRegistration, ObjectHandle};
pub use streams::{
    IBlueMemStream, IBlueStream, ICacheable, ICcpStream, IRoot, MemStream, SeekOrigin,
};
