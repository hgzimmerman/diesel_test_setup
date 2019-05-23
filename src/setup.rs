use crate::connection_wrapper::{EphemeralDatabaseConnection, EphemeralDatabasePool};
use crate::{cleanup::Cleanup, database_error::TestDatabaseError, primitives::run_migrations};
use diesel::r2d2::{self, ConnectionManager};
use migrations_internals::find_migrations_directory;
use migrations_internals::MigrationConnection;
use r2d2::PooledConnection;
use std::path::PathBuf;
use std::{ops::Deref, path::Path};

/// Encapsulates the different ways databases can be named.
#[derive(Debug)]
enum DatabaseNameOption {
    Random,
    RandomWithPrefix(String),
    Custom(String),
}

/// Builder for ephemeral test databases.
#[derive(Debug)]
pub struct TestDatabaseBuilder<'a, Conn> {
    /// Connection that is used to create and destroy the database.
    admin_conn: Conn,
    /// The scheme and authority of the database.
    /// This will be used to create new connection(s) when connecting to the newly created database.
    database_origin: &'a str,
    /// The migrations to run
    migrations_directory: Option<PathBuf>,
    /// The name of the database to be created.
    db_name: DatabaseNameOption,
}

impl<'a, Conn> TestDatabaseBuilder<'a, Conn>
where
    Conn: MigrationConnection + 'static,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    PooledConnection<ConnectionManager<Conn>>: Deref<Target = Conn>,
{
    /// Creates a new builder.
    ///
    /// # Arguments
    /// * `admin_conn` - Admin connection used for creating and dropping databases.
    /// * `database_origin` - The scheme and authority of the database that will be created.
    /// The name will be appended to this to create the URL that connects to the new database.
    ///
    /// # Notes
    /// * The `admin_conn` should have been created with the same origin present in `database_origin`.
    /// * The `database_origin` should NOT have a trailing `/`.
    pub fn new(admin_conn: Conn, database_origin: &'a str) -> Self {
        TestDatabaseBuilder {
            admin_conn,
            database_origin,
            migrations_directory: None,
            db_name: DatabaseNameOption::Random,
        }
    }

    /// Specifies the migrations directory that will be used to run migrations on the new database.
    ///
    /// If this isn't specified, then the directory will be searched for,
    /// although it cannot be guaranteed to find the migrations directory if it isn't in or above
    /// your current directory.
    ///
    /// # Arguments
    /// * `directory` - The directory where the migrations are found.
    /// This should point to the automatically created 'migrations' directory per Diesel's expectations.
    ///
    /// # Notes
    /// * If migrations can't be found, then attempting to run `setup_pool` or `setup_connection` will return an error.
    pub fn migrations_directory(mut self, directory: PathBuf) -> Self {
        self.migrations_directory = Some(directory);
        self
    }

    /// Sets the database name.
    /// If none is provided, then a random database name will be generated.
    ///
    /// # Arguments
    /// * `db_name` - The name of the database to be created.
    ///
    /// # Notes
    /// * If you provide your own database name, then it is expected to be url-safe (no spaces, url-unsafe characters).
    /// * This will overwrite any configuration made using `db_name_prefix`.
    pub fn db_name<T: Into<String>>(mut self, db_name: T) -> Self {
        self.db_name = DatabaseNameOption::Custom(db_name.into());
        self
    }

    /// Sets the database name prefix.
    /// This prefix will have a random name appended to it.
    ///
    /// # Arguments
    /// * `prefix` - The prefix to the random database name.
    ///
    /// # Notes
    /// * If you provide your own database name, then it is expected to be url-safe (no spaces, url-unsafe characters).
    /// * This will overwrite any configuration made using `db_name`.
    pub fn db_name_prefix<T: Into<String>>(mut self, prefix: T) -> Self {
        self.db_name = DatabaseNameOption::RandomWithPrefix(prefix.into());
        self
    }

    /// Creates a new database, runs migrations on it, and returns a `Pool` connected to it.
    ///
    /// # Notes
    /// * If you don't specify the migrations directory, the migrations directory must be at the root
    /// of your project in order for this function to operate as expected.
    /// Failure to locate your migrations directory there will prevent this function from finding the migrations directory.
    pub fn setup_pool(self) -> Result<EphemeralDatabasePool<Conn>, TestDatabaseError> {
        let migrations_directory: PathBuf = self
            .migrations_directory
            .map_or_else(|| find_migrations_directory(), Ok)?;
        let db_name = match self.db_name {
            DatabaseNameOption::Random => nanoid::generate(40),
            DatabaseNameOption::Custom(name) => name,
            DatabaseNameOption::RandomWithPrefix(prefix) => {
                format!("{}{}", prefix, nanoid::generate(40))
            }
        };

        setup_named_db_pool(
            self.admin_conn,
            self.database_origin,
            migrations_directory.deref(),
            db_name,
        )
    }

    /// Creates a new database, runs migrations on it, and returns a `Connection` connected to it.
    ///
    /// # Notes
    /// * If you don't specify the migrations directory, the migrations directory must be at the root
    /// of your project in order for this function to operate as expected.
    /// Failure to locate your migrations directory there will prevent this function from finding the migrations directory.
    pub fn setup_connection(self) -> Result<EphemeralDatabaseConnection<Conn>, TestDatabaseError> {
        let migrations_directory: PathBuf = self
            .migrations_directory
            .map_or_else(|| find_migrations_directory(), Ok)?;
        let db_name = match self.db_name {
            DatabaseNameOption::Random => nanoid::generate(40),
            DatabaseNameOption::Custom(name) => name,
            DatabaseNameOption::RandomWithPrefix(prefix) => {
                format!("{}_{}", prefix, nanoid::generate(40))
            }
        };

        setup_named_db(
            self.admin_conn,
            self.database_origin,
            migrations_directory.deref(),
            db_name,
        )
    }
}

/// Utility function that creates a database with a known name and runs migrations on it.
///
/// Returns a Pool of connections.
fn setup_named_db_pool<Conn>(
    admin_conn: Conn,
    database_origin: &str,
    migrations_directory: &Path,
    db_name: String,
) -> Result<EphemeralDatabasePool<Conn>, TestDatabaseError>
where
    Conn: MigrationConnection + 'static,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    PooledConnection<ConnectionManager<Conn>>: Deref<Target = Conn>,
{
    // This makes the assumption that the provided database name does not already exist on the system.
    crate::primitives::create_database(&admin_conn, &db_name)?;

    let url = format!("{}/{}", database_origin, db_name); // TODO this may only work with postgres
    let manager = ConnectionManager::<Conn>::new(url);

    let pool = r2d2::Pool::builder().max_size(3).build(manager)?;

    run_migrations(pool.get().unwrap().deref(), migrations_directory)?;

    let cleanup = Cleanup(admin_conn, db_name);
    Ok(EphemeralDatabasePool { cleanup, pool })
}

/// Utility function that creates a database with a known name and runs migrations on it.
///
/// Returns a single connection.
fn setup_named_db<Conn>(
    admin_conn: Conn,
    database_origin: &str,
    migrations_directory: &Path,
    db_name: String,
) -> Result<EphemeralDatabaseConnection<Conn>, TestDatabaseError>
where
    Conn: MigrationConnection + 'static,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
{
    crate::primitives::create_database(&admin_conn, &db_name)?;

    let url = format!("{}/{}", database_origin, db_name); // TODO this may only work with Postgres
    let connection = Conn::establish(&url)?;

    run_migrations(&connection, migrations_directory)?;
    let cleanup = Cleanup(admin_conn, db_name);

    Ok(EphemeralDatabaseConnection {
        cleanup,
        connection,
    })
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use crate::reset::drop_database;
    use crate::test_util::database_exists;
    use crate::Pool;
    use diesel::{Connection, PgConnection};

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
            drop_database(&admin_conn, &db_name).expect("should drop");;
        }

        std::panic::catch_unwind(|| {
            let admin_conn = PgConnection::establish(DROP_DATABASE_URL)
                .expect("Should be able to connect to admin db");
            let _ = setup_named_db_pool(
                admin_conn,
                url_origin,
                Path::new("../migrations"),
                db_name.clone(),
            )
            .expect("create db");
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
        drop_database(&admin_conn, &db_name).expect("should drop");

        let pool_and_cleanup = setup_named_db_pool(
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

        std::mem::drop(pool_and_cleanup);

        let db_exists: bool =
            database_exists(&admin_conn, &db_name).expect("Should determine if database exists");
        assert!(!db_exists)
    }

    #[test]
    fn lack_of_assignment_still_allows_correct_drop_order() {
        let url_origin = DATABASE_ORIGIN;
        let db_name = "lack_of_assignment_still_allows_correct_drop_order_TEST".to_string();

        let admin_conn = PgConnection::establish(DROP_DATABASE_URL)
            .expect("Should be able to connect to admin db");
        // precautionary drop
        drop_database(&admin_conn, &db_name).expect("should drop");

        setup_named_db_pool(
            admin_conn,
            url_origin,
            Path::new("../migrations"),
            db_name.clone(),
        )
        .unwrap();
    }

    #[test]
    fn normal_assignment_allows_correct_drop_order() {
        let url_origin = DATABASE_ORIGIN;
        let db_name = "normal_assignment_allows_correct_drop_order_TEST".to_string();

        let admin_conn = PgConnection::establish(DROP_DATABASE_URL)
            .expect("Should be able to connect to admin db");
        // precautionary drop
        drop_database(&admin_conn, &db_name).expect("should drop");

        let _pool_and_cleanup = setup_named_db_pool(
            admin_conn,
            url_origin,
            Path::new("../migrations"),
            db_name.clone(),
        )
        .unwrap();
    }

    #[test]
    fn late_assignment_allows_correct_drop_order() {
        let url_origin = DATABASE_ORIGIN;
        let db_name = "late_assignment_allows_correct_drop_order_TEST".to_string();

        let admin_conn = PgConnection::establish(DROP_DATABASE_URL)
            .expect("Should be able to connect to admin db");
        // precautionary drop
        drop_database(&admin_conn, &db_name).expect("should drop");

        let x = setup_named_db_pool(
            admin_conn,
            url_origin,
            Path::new("../migrations"),
            db_name.clone(),
        )
        .unwrap();
        let _pool = x.pool;
    }

    #[test]
    fn deref_out_of_function_maintains_correct_drop_order() {
        let url_origin = DATABASE_ORIGIN;
        let db_name = "deref_should_break_TEST".to_string();

        let admin_conn = PgConnection::establish(DROP_DATABASE_URL)
            .expect("Should be able to connect to admin db");
        // precautionary drop
        drop_database(&admin_conn, &db_name).expect("should drop");

        let _: &Pool<PgConnection> = setup_named_db_pool(
            admin_conn,
            url_origin,
            Path::new("../migrations"),
            db_name.clone(),
        )
        .unwrap()
        .deref();
    }
}
