use crate::{cleanup::Cleanup, database_error::DatabaseError, reset::run_migrations};
use diesel::r2d2::{self, ConnectionManager};
use migrations_internals::MigrationConnection;
use r2d2::PooledConnection;
use std::{ops::Deref, path::Path};
use migrations_internals::find_migrations_directory;

/// Creates a db with a unique name using the administrative connection.
/// The database will be deleted when the Cleanup return value is dropped.
///
/// # Arguments
/// * `admin_conn`: A connection to a database that has the authority to create other databases on the system.
/// * `database_origin`: A string representing the scheme + authority pointing to the database server that tests will be conducted on.
///
/// # Returns
/// * A pool connected to the new database.
/// * A RAII cleanup token that drops the database when it exits scope.
///
/// # Notes
/// * The migrations directory must be at the root of your project in order for this function to operate properly.
/// Failure to locate your migrations directory there will prevent this function from finding the migrations directory.
/// If you insist on using a different migrations directory,
/// [setup_unique_db_pool_with_migrations](fn.setup_unique_db_pool_with_migrations.html) allows you to specify the directory manually.
/// * The `admin_conn` should have been created with the same origin present in `database_origin`.
/// * The `database_origin` should NOT have a trailing `/`.
pub fn setup_unique_db_pool<Conn>(
    admin_conn: Conn,
    database_origin: &str,
) -> Result<(r2d2::Pool<ConnectionManager<Conn>>, Cleanup<Conn>), DatabaseError>
where
    Conn: MigrationConnection + 'static,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    PooledConnection<ConnectionManager<Conn>>: Deref<Target = Conn>,
{
    let migrations_directory = find_migrations_directory()?;
    setup_unique_db_pool_with_migrations(admin_conn, database_origin, migrations_directory.as_path())
}

/// Creates a db with a unique name using the administrative connection.
/// The database will be deleted when the Cleanup return value is dropped.
///
/// # Arguments
/// * `admin_conn`: A connection to a database that has the authority to create other databases on the system.
/// * `database_origin`: A string representing the scheme + authority pointing to the database server that tests will be conducted on.
/// * `migrations directory`: A Path pointing to the directory where Diesel migrations are stored.
///
/// # Returns
/// * A pool connected to the new database.
/// * A RAII cleanup token that drops the database when it exits scope.
///
/// # Notes
/// The `admin_conn` should have been created with the same origin present in `database_origin`.
/// The `database_origin` should NOT have a trailing `/`.
pub fn setup_unique_db_pool_with_migrations<Conn>(
    admin_conn: Conn,
    database_origin: &str,
    migrations_directory: &Path,
) -> Result<(r2d2::Pool<ConnectionManager<Conn>>, Cleanup<Conn>), DatabaseError>
where
    Conn: MigrationConnection + 'static,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    PooledConnection<ConnectionManager<Conn>>: Deref<Target = Conn>,
{
    let db_name = nanoid::generate(40); // Gets a random url-safe string.
    setup_named_db_pool(admin_conn, database_origin, migrations_directory, db_name)
}

/// Utility function that creates a database with a known name and runs migrations on it.
///
/// Returns a Pool of connections.
fn setup_named_db_pool<Conn>(
    admin_conn: Conn,
    database_origin: &str,
    migrations_directory: &Path,
    db_name: String,
) -> Result<(r2d2::Pool<ConnectionManager<Conn>>, Cleanup<Conn>), DatabaseError>
where
    Conn: MigrationConnection + 'static,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    PooledConnection<ConnectionManager<Conn>>: Deref<Target = Conn>,
{
    // This makes the assumption that the provided database name does not already exist on the system.
    crate::reset::create_database(&admin_conn, &db_name)?;

    let url = format!("{}/{}", database_origin, db_name); // TODO this may only work with postgres
    let manager = ConnectionManager::<Conn>::new(url);

    let pool = r2d2::Pool::builder()
        .max_size(3)
        .build(manager)
        .map_err(|e: r2d2::PoolError| DatabaseError::from(e))?;

    run_migrations(pool.get().unwrap().deref(), migrations_directory)?;

    let cleanup = Cleanup(admin_conn, db_name);
    Ok((pool, cleanup))
}

/// Creates a db with a unique name using the administrative connection.
/// The database will be deleted when the Cleanup return value is dropped.
///
/// # Arguments
/// * `admin_conn`: A connection to a database that has the authority to create other databases on the system.
/// * `database_origin`: A string representing the scheme + authority pointing to the database server that tests will be conducted on.
/// * `migrations directory`: A path pointing to the directory where Diesel migrations are stored.
///
/// # Returns
/// * A single connection to the new database.
/// * A RAII cleanup token that drops the database when it exits scope.
///
/// # Note
/// The `admin_conn` should have been created with the same origin present in `database_origin`.
pub fn setup_unique_db<Conn>(
    admin_conn: Conn,
    database_origin: &str,
    migrations_directory: &Path,
) -> Result<(Conn, Cleanup<Conn>), DatabaseError>
where
    Conn: MigrationConnection + 'static,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    PooledConnection<ConnectionManager<Conn>>: Deref<Target = Conn>,
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
    PooledConnection<ConnectionManager<Conn>>: Deref<Target = Conn>,
{
    crate::reset::create_database(&admin_conn, &db_name)?;

    let url = format!("{}/{}", database_origin, db_name); // TODO this may only work with Postgres
    let conn = Conn::establish(&url).map_err(DatabaseError::from)?;

    run_migrations(&conn, migrations_directory)?;
    let cleanup = Cleanup(admin_conn, db_name);
    Ok((conn, cleanup))
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use crate::test_util::database_exists;
    use diesel::{Connection, PgConnection};
    use crate::reset::drop_database;

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

        // Make sure that the db doesn't exist beforehand.
        {
            let admin_conn = PgConnection::establish(DROP_DATABASE_URL)
                .expect("Should be able to connect to admin db");
            drop_database(&admin_conn, &db_name);
            std::mem::drop(admin_conn);
        }

        std::panic::catch_unwind(|| {
            let admin_conn = PgConnection::establish(DROP_DATABASE_URL)
                .expect("Should be able to connect to admin db");
            let _ = setup_named_db_pool(
                admin_conn,
                url_origin,
                Path::new("../migrations"),
                db_name.clone(),
            );
            panic!("expected_panic");
        })
        .expect_err("Should catch panic.");

        let admin_conn = PgConnection::establish(DROP_DATABASE_URL)
            .expect("Should be able to connect to admin db");
        let database_exists: bool =
            database_exists(&admin_conn, &db_name).expect("Should determine if database exists");
        assert!(!database_exists)
    }

    #[test]
    fn cleanup_drops_database() {
        let url_origin = DATABASE_ORIGIN;
        let db_name = "cleanup_drops_database_TEST_DB".to_string();

        let admin_conn = PgConnection::establish(DROP_DATABASE_URL)
            .expect("Should be able to connect to admin db");
        // precautionary drop
        drop_database(&admin_conn, &db_name);

        let (pool, cleanup) = setup_named_db_pool(
            admin_conn,
            url_origin,
            Path::new("../migrations"),
            db_name.clone(),
        )
        .unwrap();

        let admin_conn = PgConnection::establish(DROP_DATABASE_URL)
            .expect("Should be able to connect to admin db");

        let db_exists: bool =
            database_exists(&admin_conn, &db_name).expect("Should determine if database exists");
        assert!(db_exists);

        std::mem::drop(pool);
        std::mem::drop(cleanup);

        let db_exists: bool =
            database_exists(&admin_conn, &db_name).expect("Should determine if database exists");
        assert!(!db_exists)
    }
}
