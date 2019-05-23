//! Provides functions for setting up unique databases that are automatically dropped when tests finish.
//!
//! # Example
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
