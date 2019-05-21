//! This is used as a set of common behaviors used for integration testing between the DB and Server crates.
//! Its primary purpose is to provide the `setup` and `setup_client` methods.
//! These will reset the database and populate it with a provided fixture, the values of which are then
//! allowed to be used within the scope of the test.
//!
//! This crate relies heavily on the implementation of diesel_cli for performing the database resets.

#[macro_use]
extern crate diesel;
extern crate migrations_internals;

mod database_error;
mod query_helper;
mod reset;
pub mod setup;

pub use reset::{create_database, drop_database, is_superuser};
pub use setup::setup_pool_random_db;
