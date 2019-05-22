use diesel::result;

use std::{convert::From, error::Error, fmt, io};

use self::TestDatabaseError::*;
use diesel::{migration::RunMigrationsError, r2d2};
use migrations_internals::MigrationError;

pub type TestDatabaseResult<T> = Result<T, TestDatabaseError>;

/// Errors that can occur while setting up or cleaning up test databases.
#[derive(Debug)]
pub enum TestDatabaseError {
    RunMigrationsError(RunMigrationsError),
    MigrationError(MigrationError),
    PoolCreationError(r2d2::PoolError),
    IoError(io::Error),
    QueryError(result::Error),
    ConnectionError(result::ConnectionError),
}

impl From<io::Error> for TestDatabaseError {
    fn from(e: io::Error) -> Self {
        IoError(e)
    }
}

impl From<result::Error> for TestDatabaseError {
    fn from(e: result::Error) -> Self {
        QueryError(e)
    }
}

impl From<result::ConnectionError> for TestDatabaseError {
    fn from(e: result::ConnectionError) -> Self {
        ConnectionError(e)
    }
}

impl From<r2d2::PoolError> for TestDatabaseError {
    fn from(e: r2d2::PoolError) -> Self {
        PoolCreationError(e)
    }
}

impl From<RunMigrationsError> for TestDatabaseError {
    fn from(e: RunMigrationsError) -> Self {
        RunMigrationsError(e)
    }
}

impl From<MigrationError> for TestDatabaseError {
    fn from(e: MigrationError) -> Self {
        MigrationError(e)
    }
}

impl Error for TestDatabaseError {
    fn description(&self) -> &str {
        match *self {
            RunMigrationsError(ref error) => error
                .source()
                .map(Error::description)
                .unwrap_or_else(|| error.description()),
            MigrationError(ref error) => error
                .source()
                .map(Error::description)
                .unwrap_or_else(|| error.description()),
            PoolCreationError(ref error) => error
                .source()
                .map(Error::description)
                .unwrap_or_else(|| error.description()),
            IoError(ref error) => error
                .source()
                .map(Error::description)
                .unwrap_or_else(|| error.description()),
            QueryError(ref error) => error
                .source()
                .map(Error::description)
                .unwrap_or_else(|| error.description()),
            ConnectionError(ref error) => error
                .source()
                .map(Error::description)
                .unwrap_or_else(|| error.description()),
        }
    }
}

impl fmt::Display for TestDatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.description().fmt(f)
    }
}

impl PartialEq for TestDatabaseError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            //            (&CargoTomlNotFound, &CargoTomlNotFound) => true,
            _ => false,
        }
    }
}
