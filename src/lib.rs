//! This crate relies heavily on the implementation of diesel_cli for performing the database resets.

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

pub use setup::{setup_pool_random_db, setup_random_db};
