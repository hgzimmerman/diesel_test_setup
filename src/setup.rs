use crate::reset::{ DropCreateDb};
#[cfg(test)]
use diesel::Connection;
use diesel::{r2d2, PgConnection};
use diesel::r2d2::ConnectionManager;
use crate::reset::drop_database;
use crate::reset::run_migrations;
use migrations_internals::MigrationConnection;
use r2d2::PooledConnection;
use std::ops::Deref;

/// Cleanup wrapper.
/// Contains the admin connection and the name of the database (not the whole url).
///
/// When this struct goes out of scope, it will use the data it owns to drop the database it's
/// associated with.
pub struct Cleanup<Conn>(Conn, String)
where
    Conn: DropCreateDb,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword;

impl <Conn: DropCreateDb> Drop for Cleanup<Conn>
where
    Conn: DropCreateDb,
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
/// # Note
/// The `admin_conn` should have been created with the same origin present in `database_origin`.
pub fn setup_pool_random_db<Conn: DropCreateDb + 'static>(
    admin_conn: Conn,
    database_origin: &str,
    migrations_directory: &str, // TODO make this a pathbuf
) -> (r2d2::Pool<ConnectionManager<Conn>>, Cleanup<Conn>)
where
    Conn: DropCreateDb + MigrationConnection + 'static,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    PooledConnection<ConnectionManager<Conn>>: Deref<Target=Conn>
{
    let db_name = nanoid::generate(40); // Gets a random url-safe string.
    setup_pool_named_db(admin_conn, database_origin, migrations_directory, db_name)
}



/// Utility function that creates a database with a known name and runs migrations on it.
///
fn setup_pool_named_db<Conn>(
    admin_conn: Conn,
    url_part: &str,
    migrations_directory: &str,
    db_name: String,
) -> (r2d2::Pool<ConnectionManager<Conn>>, Cleanup<Conn>)
where
    Conn: DropCreateDb + MigrationConnection + 'static,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    PooledConnection<ConnectionManager<Conn>>: Deref<Target=Conn>
{
    // This makes the assumption that the provided database name does not already exist on the system.
    crate::reset::create_database(&admin_conn, &db_name).expect("Couldn't create database");

    let url = format!("{}/{}", url_part, db_name);
    let manager = ConnectionManager::<Conn>::new(url);

    let pool = r2d2::Pool::builder()
        .max_size(3)
        .build(manager)
        .expect("Couldn't create pool");

//    let normal_conn: &Conn = ;
    run_migrations(pool.get().unwrap().deref(), migrations_directory);

    let cleanup = Cleanup(admin_conn, db_name);
    (pool, cleanup)
}

// TODO all tests should be integration style.
// Likely hide them behind some sort of ENV VAR that indicates that it is running in a docker container or something
#[cfg(test)]
pub(crate) mod test {
    use super::*;

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
                setup_pool_named_db(admin_conn, url_origin, "../db/migrations", db_name.clone());
            panic!("expected_panic");
        })
        .expect_err("Should catch panic.");

        let admin_conn = PgConnection::establish(DROP_DATABASE_URL)
            .expect("Should be able to connect to admin db");
        let database_exists: bool = admin_conn.database_exists( &db_name)
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
                setup_pool_named_db(admin_conn, url_origin, "../db/migrations", db_name.clone());

        let admin_conn = PgConnection::establish(DROP_DATABASE_URL)
            .expect("Should be able to connect to admin db");

        let database_exists: bool = admin_conn.database_exists( &db_name)
            .expect("Should determine if database exists");
        assert!(database_exists);

        std::mem::drop(pool);
        std::mem::drop(cleanup);

        let database_exists: bool = admin_conn.database_exists( &db_name)
            .expect("Should determine if database exists");
        assert!(!database_exists)
    }
}
