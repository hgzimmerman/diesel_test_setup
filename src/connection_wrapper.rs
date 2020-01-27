use crate::{Cleanup, Pool, RemoteConnection};
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::PooledConnection;
use migrations_internals::MigrationConnection;
use std::ops::Deref;


/// A struct that enforces drop order for a pool and the cleanup routine.
#[derive(Debug)]
pub struct EphemeralDatabasePool<Conn>
where
    Conn: MigrationConnection + RemoteConnection + 'static,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    PooledConnection<ConnectionManager<Conn>>: Deref<Target = Conn>,
{
    pub(crate) pool: Pool<Conn>,       // should drop first
    pub(crate) cleanup: Cleanup<Conn>, // should drop second
}

impl<Conn> EphemeralDatabasePool<Conn>
where
    Conn: MigrationConnection + RemoteConnection + 'static,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    PooledConnection<ConnectionManager<Conn>>: Deref<Target = Conn>,
{
    /// Converts the struct into a tuple.
    ///
    /// # Warning
    /// You are responsible for making sure that the `Pool` does not outlive the `Cleanup`.
    #[must_use]
    pub fn into_tuple(self) -> (Pool<Conn>, Cleanup<Conn>) {
        (self.pool, self.cleanup)
    }
}

impl<Conn> Deref for EphemeralDatabasePool<Conn>
where
    Conn: MigrationConnection + RemoteConnection + 'static,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    PooledConnection<ConnectionManager<Conn>>: Deref<Target = Conn>,
{
    type Target = Pool<Conn>;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

/// A struct that enforces drop order for a single connection and the cleanup routine.
#[derive(Debug)]
pub struct EphemeralDatabaseConnection<Conn>
where
    Conn: MigrationConnection + RemoteConnection + 'static,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
{
    pub(crate) connection: Conn,       // should drop first
    pub(crate) cleanup: Cleanup<Conn>, // should drop second
}

impl<Conn> EphemeralDatabaseConnection<Conn>
where
    Conn: MigrationConnection + RemoteConnection + 'static,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
{
    /// Converts the struct into a tuple.
    ///
    /// # Warning
    /// You are responsible for making sure that the `Conn` does not outlive the `Cleanup`.
    #[must_use]
    pub fn into_tuple(self) -> (Conn, Cleanup<Conn>) {
        (self.connection, self.cleanup)
    }
}

