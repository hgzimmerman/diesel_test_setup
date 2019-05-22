//! Provides functions for setting up unique databases that are automatically dropped when tests finish.

#[cfg(test)]
#[macro_use]
extern crate diesel;
#[cfg(not(test))]
extern crate diesel;

extern crate migrations_internals;

pub mod cleanup;
mod database_error;
mod query_helper;
mod reset;
pub mod setup;
#[cfg(test)]
mod test_util;

pub use setup::{setup_unique_db_pool, setup_unique_db};
pub use cleanup::Cleanup;
