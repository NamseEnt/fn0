#![allow(clippy::large_enum_variant)]

pub mod auth;
pub mod core;

#[cfg(feature = "audit")]
pub mod audit;
#[cfg(feature = "containerinstances")]
pub mod containerinstances;
#[cfg(feature = "containerinstances")]
pub use containerinstances as container_instances;
#[cfg(feature = "osmanagementhub")]
pub mod osmanagementhub;
#[cfg(feature = "resourcesearch")]
pub mod resourcesearch;
