//! Primitives functions on which the higher abstractions in the crate are built on.

use crate::{
    database_error::{TestDatabaseError, TestDatabaseResult},
    query_helper,
};
use diesel::{query_dsl::RunQueryDsl, Connection};
use migrations_internals as migrations;
use migrations_internals::MigrationConnection;
use std::path::Path;

/// Drops the database, completely removing every table (and therefore every row) in the database.
///
/// # Arguments
/// * `admin_conn` - Admin connection to the database.
/// * `database_name` - The name of the database to be deleted.
pub fn drop_database<T>(admin_conn: &T, database_name: &str) -> TestDatabaseResult<()>
where
    T: Connection,
    <T as Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
{
    query_helper::drop_database(database_name)
        .if_exists()
        .execute(admin_conn)
        .map_err(TestDatabaseError::from)
        .map(|_| ())
}


/// Creates a database with a given name.
///
/// # Arguments
/// * `admin_conn` - Admin connection to the database.
/// * `database_name` - The name of the new database to be created.
pub fn create_database<T>(admin_conn: &T, database_name: &str) -> TestDatabaseResult<()>
where
    T: Connection,
    <T as Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
{
    query_helper::create_database(database_name)
        .execute(admin_conn)
        .map_err(TestDatabaseError::from)
        .map(|_| ())
}

/// Creates tables in the database.
///
/// # Arguments
/// * `normal_conn` - Non-admin connection to the database.
/// * `migrations_directory` - Directory to the migrations directory.
///
/// # Note
/// The connection used here should be different from the admin connection used for resetting the database.
/// Instead, the connection should be to the database on which tests will be performed on.
pub fn run_migrations<T>(normal_conn: &T, migrations_directory: &Path) -> TestDatabaseResult<()>
where
    T: MigrationConnection,
    <T as Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
{
    migrations::run_pending_migrations_in_directory(
        normal_conn,
        migrations_directory,
        &mut ::std::io::sink(),
    )
    .map_err(TestDatabaseError::from)
}