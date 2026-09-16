//! Package I/O targets WorldState, never host paths, executables or sockets.
pub mod cli;
pub mod deb;
pub mod executables;
pub mod ipc;
pub mod model;
pub mod repository;
pub mod resolver;
pub mod service;
pub mod state;
#[cfg(test)]
mod tests;
pub mod version;
