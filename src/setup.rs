use crate::connection_wrapper::{EphemeralDatabaseConnection, EphemeralDatabasePool};
use crate::{
    cleanup::Cleanup, database_error::TestDatabaseError, core::run_migrations,
    RemoteConnection,
};
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
    Conn: MigrationConnection + RemoteConnection + 'static,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    PooledConnection<ConnectionManager<Conn>>: Deref<Target = Conn>,
{
    /// Creates a new builder.
    ///
    /// # Arguments
    ///
    /// * `admin_conn` - Admin connection used for creating and dropping databases.
    /// * `database_origin` - The scheme and authority of the database that will be created.
    /// The name will be appended to this to create the URL that connects to the new database.
    ///
    /// # Notes
    ///
    /// * The `admin_conn` should have been created with the same origin present in `database_origin`.
    /// * The `database_origin` should NOT have a trailing '/'.
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
    ///
    /// * `directory` - The directory where the migrations are found.
    /// This should point to the automatically created 'migrations' directory per Diesel's expectations.
    ///
    /// # Notes
    ///
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
    ///
    /// * If you provide your own database name, then it is expected to be url-safe (no spaces, url-unsafe characters).
    /// * This will overwrite any configuration made using `db_name`.
    pub fn db_name_prefix<T: Into<String>>(mut self, prefix: T) -> Self {
        self.db_name = DatabaseNameOption::RandomWithPrefix(prefix.into());
        self
    }

    /// Creates a new database, runs migrations on it, and returns a `Pool` connected to it.
    ///
    /// # Notes
    ///
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
            &*migrations_directory,
            db_name,
        )
    }

    /// Creates a new database, runs migrations on it, and returns a `Connection` connected to it.
    ///
    /// # Notes
    ///
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
pub(crate) fn setup_named_db_pool<Conn>(
    admin_conn: Conn,
    database_origin: &str,
    migrations_directory: &Path,
    db_name: String,
) -> Result<EphemeralDatabasePool<Conn>, TestDatabaseError>
where
    Conn: MigrationConnection + RemoteConnection + 'static,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    PooledConnection<ConnectionManager<Conn>>: Deref<Target = Conn>,
{
    // This makes the assumption that the provided database name does not already exist on the system.
    crate::core::create_database(&admin_conn, &db_name)?;

    let url = format!("{}/{}", database_origin, db_name);
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
    Conn: MigrationConnection + RemoteConnection + 'static,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
{
    crate::core::create_database(&admin_conn, &db_name)?;

    let url = format!("{}/{}", database_origin, db_name); // TODO this may only work with Postgres
    let connection = Conn::establish(&url)?;

    run_migrations(&connection, migrations_directory)?;
    let cleanup = Cleanup(admin_conn, db_name);

    Ok(EphemeralDatabaseConnection {
        cleanup,
        connection,
    })
}
