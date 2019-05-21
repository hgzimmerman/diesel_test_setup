use diesel::result;

use std::{convert::From, error::Error, fmt, io};

use self::DatabaseError::*;
use diesel::{migration::RunMigrationsError, r2d2};

pub type DatabaseResult<T> = Result<T, DatabaseError>;

/// Errors that can occur while setting up or operating the test database.
#[derive(Debug)]
pub enum DatabaseError {
    MigrationsError(RunMigrationsError),
    PoolCreationError(r2d2::PoolError),
    IoError(io::Error),
    QueryError(result::Error),
    ConnectionError(result::ConnectionError),
}

impl From<io::Error> for DatabaseError {
    fn from(e: io::Error) -> Self {
        IoError(e)
    }
}

impl From<result::Error> for DatabaseError {
    fn from(e: result::Error) -> Self {
        QueryError(e)
    }
}

impl From<result::ConnectionError> for DatabaseError {
    fn from(e: result::ConnectionError) -> Self {
        ConnectionError(e)
    }
}

impl From<r2d2::PoolError> for DatabaseError {
    fn from(e: r2d2::PoolError) -> Self {
        PoolCreationError(e)
    }
}

impl From<RunMigrationsError> for DatabaseError {
    fn from(e: RunMigrationsError) -> Self {
        MigrationsError(e)
    }
}

impl Error for DatabaseError {
    fn description(&self) -> &str {
        match *self {
            MigrationsError(ref error) => error
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

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.description().fmt(f)
    }
}

impl PartialEq for DatabaseError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            //            (&CargoTomlNotFound, &CargoTomlNotFound) => true,
            _ => false,
        }
    }
}
