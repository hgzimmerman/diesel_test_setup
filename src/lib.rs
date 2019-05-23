//! Provides functions for setting up unique databases that are automatically dropped when tests finish.
//!
//! # Examples
//!
//! `pool` is connected to a new database instance set up as if you had ran `diesel migration run` on it.
//!
//! When `_cleanup` exists this scope, the database that `pool` is connected to will be dropped, even if your test panics.
//!
//! ```
//!# use ::diesel_test_setup::Cleanup;
//!# use diesel::PgConnection;
//!# use diesel::Connection;
//!use diesel_test_setup::{TestDatabaseBuilder, EphemeralDatabasePool};
//!# use diesel::r2d2::Pool;
//!# use diesel::r2d2::ConnectionManager;
//!# use std::ops::Deref;
//!# const ADMIN_DATABASE_URL: &str = env!("DROP_DATABASE_URL");
//!
//!{
//!    let admin_conn = PgConnection::establish(ADMIN_DATABASE_URL).unwrap();
//!    const DATABASE_ORIGIN: &str = "postgres://localhost";
//!    let pool: EphemeralDatabasePool<PgConnection> = TestDatabaseBuilder::new(
//!        admin_conn,
//!        DATABASE_ORIGIN
//!    )
//!    .setup_pool()
//!    .expect("Could not create the database.");
//!
//!    let pool: &Pool<ConnectionManager<PgConnection>> = pool.deref();
//!
//!
//!    // Perform your test using `pool`
//!}
//! ```
//!
//! --------
//!
//! This function could have the same signature as one that sets up a Fake database,
//! allowing easily swapping between them depending on if you want to run integration tests
//! or unit tests.
//!
//! Running the test within a `Fn` instead of just returning only a `Pool`
//! is motivated by the requirement to keep the `Cleanup` struct around so it doesn't go out
//! of scope first, while keeping the function signature the same as if you were working with a
//! `Fake` database.
//!```
//!# use diesel::PgConnection;
//!# use diesel::Connection;
//!# use diesel_test_setup::{TestDatabaseBuilder};
//!# use diesel::r2d2::ConnectionManager;
//!# use diesel::r2d2::Pool;
//!# const ADMIN_DATABASE_URL: &str = env!("DROP_DATABASE_URL");
//!# pub struct FakeTestDouble;
//!pub enum DatabaseOrFake {
//!    Pool(Pool<ConnectionManager<PgConnection>>),
//!    Fake(FakeTestDouble),
//!}
//!
//!pub fn execute_test_with_pool<Fun>(mut test_function: Fun)
//!where
//!    Fun: Fn(DatabaseOrFake),
//!{
//!   let admin_conn = PgConnection::establish(ADMIN_DATABASE_URL).unwrap();
//!   let (pool, _cleanup) = TestDatabaseBuilder::new(
//!       admin_conn,
//!       "postgres://localhost",
//!   )
//!       .db_name_prefix("test")
//!       .setup_pool()
//!       .expect("Could not setup the database.")
//!       .into_tuple();
//!
//!    test_function(DatabaseOrFake::Pool(pool));
//!}
//! ```

#[cfg(test)]
#[macro_use]
extern crate diesel;
#[cfg(not(test))]
extern crate diesel;

extern crate migrations_internals;

mod cleanup;
mod connection_wrapper;
mod database_error;
mod query_helper;
mod reset;
mod setup;
#[cfg(test)]
mod test_util;

pub use cleanup::Cleanup;
pub use connection_wrapper::{EphemeralDatabaseConnection, EphemeralDatabasePool};
pub use database_error::{TestDatabaseError, TestDatabaseResult};
pub use setup::TestDatabaseBuilder;

use diesel::r2d2;
use diesel::r2d2::ConnectionManager;

type Pool<Conn> = r2d2::Pool<ConnectionManager<Conn>>;
