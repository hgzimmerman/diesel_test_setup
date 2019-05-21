use diesel::Connection;
use diesel::{r2d2};
use diesel::r2d2::ConnectionManager;
use crate::reset::drop_database;
use crate::reset::run_migrations;
use migrations_internals::MigrationConnection;
use r2d2::PooledConnection;
use std::ops::Deref;
use std::path::Path;
use crate::database_error::DatabaseError;

/// Cleanup wrapper.
/// Contains the admin connection and the name of the database (not the whole url).
///
/// When this struct goes out of scope, it will use the data it owns to drop the database it's
/// associated with.
pub struct Cleanup<Conn>(Conn, String)
where
    Conn: Connection,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword;

impl <Conn> Drop for Cleanup<Conn>
where
    Conn: Connection,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword
{
    fn drop(&mut self) {
        drop_database(&self.0, &self.1)
            .expect("Couldn't drop database at end of test.");
    }
}

/// Creates a db with a random name using the administrative connection.
/// The database will be deleted when the Cleanup return value is dropped.
///
/// # Arguments
/// * `admin_conn`: A connection to a database that has the authority to create other databases on the system.
/// * `database_origin`: A string representing the scheme + authority pointing to the database server that tests will be conducted on.
/// * `migrations directory`: A string pointing to the directory where Diesel migrations are stored.
///
/// # Returns
/// * A pool connected to the new database.
/// * A RAII cleanup token that drops the database when it exits scope.
///
/// # Note
/// The `admin_conn` should have been created with the same origin present in `database_origin`.
pub fn setup_pool_random_db<Conn>(
    admin_conn: Conn,
    database_origin: &str,
    migrations_directory: &Path,
) -> Result<(r2d2::Pool<ConnectionManager<Conn>>, Cleanup<Conn>), DatabaseError>
where
    Conn: MigrationConnection + 'static,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    PooledConnection<ConnectionManager<Conn>>: Deref<Target=Conn>
{
    let db_name = nanoid::generate(40); // Gets a random url-safe string.
    setup_pool_named_db(admin_conn, database_origin, migrations_directory, db_name)
}



/// Utility function that creates a database with a known name and runs migrations on it.
///
/// Returns a Pool of connections.
fn setup_pool_named_db<Conn>(
    admin_conn: Conn,
    database_origin: &str,
    migrations_directory: &Path,
    db_name: String,
) -> Result<(r2d2::Pool<ConnectionManager<Conn>>, Cleanup<Conn>), DatabaseError>
where
    Conn: MigrationConnection + 'static,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    PooledConnection<ConnectionManager<Conn>>: Deref<Target=Conn>
{
    // This makes the assumption that the provided database name does not already exist on the system.
    crate::reset::create_database(&admin_conn, &db_name)?;

    let url = format!("{}/{}", database_origin, db_name);
    let manager = ConnectionManager::<Conn>::new(url);

    let pool = r2d2::Pool::builder()
        .max_size(3)
        .build(manager)
        .map_err(|e: r2d2::PoolError | DatabaseError::from(e))?;

    run_migrations(pool.get().unwrap().deref(), migrations_directory)?;

    let cleanup = Cleanup(admin_conn, db_name);
    Ok((pool, cleanup))
}

/// Creates a db with a random name using the administrative connection.
/// The database will be deleted when the Cleanup return value is dropped.
///
/// # Arguments
/// * `admin_conn`: A connection to a database that has the authority to create other databases on the system.
/// * `database_origin`: A string representing the scheme + authority pointing to the database server that tests will be conducted on.
/// * `migrations directory`: A string pointing to the directory where Diesel migrations are stored.
///
/// # Returns
/// * A single connection to the new database.
/// * A RAII cleanup token that drops the database when it exits scope.
///
/// # Note
/// The `admin_conn` should have been created with the same origin present in `database_origin`.
pub fn setup_random_db<Conn>(
    admin_conn: Conn,
    database_origin: &str,
    migrations_directory: &Path,
) -> Result<(Conn, Cleanup<Conn>), DatabaseError>
where
    Conn: MigrationConnection + 'static,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    PooledConnection<ConnectionManager<Conn>>: Deref<Target=Conn>
{
    let db_name = nanoid::generate(40); // Gets a random url-safe string.
    setup_named_db(admin_conn, database_origin, migrations_directory, db_name)
}

/// Utility function that creates a database with a known name and runs migrations on it.
///
/// Returns a single connection.
fn setup_named_db<Conn>(
    admin_conn: Conn,
    database_origin: &str,
    migrations_directory: &Path,
    db_name: String,
) -> Result<(Conn, Cleanup<Conn>), DatabaseError>
where
    Conn: MigrationConnection + 'static,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    PooledConnection<ConnectionManager<Conn>>: Deref<Target=Conn>
{
    crate::reset::create_database(&admin_conn, &db_name)?;

    let url = format!("{}/{}", database_origin, db_name);
    let conn = Conn::establish(&url).map_err(DatabaseError::from)?;

    run_migrations(&conn, migrations_directory)?;
    let cleanup = Cleanup(admin_conn, db_name);
    Ok((conn, cleanup))
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use diesel::PgConnection;
    use crate::reset::test_util::database_exists;

    /// Should point to the base postgres account.
    /// One that has authority to create and destroy other database instances.
    ///
    /// It is expected to be on the same database server as the one indicated by DATABASE_ORIGIN.
    pub(crate) const DROP_DATABASE_URL: &str = env!("DROP_DATABASE_URL");
    /// The origin (scheme, user, password, address, port) of the test database.
    ///
    /// This determines which database server is connected to, but allows for specification of
    /// a specific database instance within the server to connect to and run tests with.
    const DATABASE_ORIGIN: &str = env!("TEST_DATABASE_ORIGIN");

    #[test]
    fn cleanup_drops_db_after_panic() {
        let url_origin = DATABASE_ORIGIN;
        let db_name = "cleanup_drops_db_after_panic_TEST_DB".to_string();

        std::panic::catch_unwind(|| {
            let admin_conn = PgConnection::establish(DROP_DATABASE_URL)
                .expect("Should be able to connect to admin db");
            let _ =
                setup_pool_named_db(admin_conn, url_origin, Path::new("../db/migrations"), db_name.clone());
            panic!("expected_panic");
        })
        .expect_err("Should catch panic.");

        let admin_conn = PgConnection::establish(DROP_DATABASE_URL)
            .expect("Should be able to connect to admin db");
        let database_exists: bool = database_exists(&admin_conn, &db_name)
            .expect("Should determine if database exists");
        assert!(!database_exists)
    }

    #[test]
    fn cleanup_drops_database() {
        let url_origin = DATABASE_ORIGIN;
        let db_name = "cleanup_drops_database_TEST_DB".to_string();

         let admin_conn = PgConnection::establish(DROP_DATABASE_URL)
                .expect("Should be able to connect to admin db");
        let (pool, cleanup) =
                setup_pool_named_db(admin_conn, url_origin, Path::new("../db/migrations"), db_name.clone())
                    .unwrap();

        let admin_conn = PgConnection::establish(DROP_DATABASE_URL)
            .expect("Should be able to connect to admin db");

        let db_exists: bool = database_exists( &admin_conn, &db_name)
            .expect("Should determine if database exists");
        assert!(db_exists);

        std::mem::drop(pool);
        std::mem::drop(cleanup);

        let db_exists: bool = database_exists( &admin_conn, &db_name)
            .expect("Should determine if database exists");
        assert!(!db_exists)
    }
}
