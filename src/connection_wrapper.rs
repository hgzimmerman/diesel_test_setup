use crate::{Cleanup, Pool, RemoteConnection, TestDatabaseError};
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::PooledConnection;
use migrations_internals::MigrationConnection;
use std::ops::Deref;
use diesel::{QueryResult, ConnectionResult, Queryable};
use diesel::connection::SimpleConnection;
use diesel::connection::Connection;
use diesel::query_builder::{QueryId, QueryFragment, AsQuery};
use diesel::deserialize::QueryableByName;
use diesel::sql_types::HasSqlType;
use diesel::connection::TransactionManager;

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

impl<Conn> Deref for EphemeralDatabaseConnection<Conn>
where
    Conn: MigrationConnection + RemoteConnection + 'static,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    PooledConnection<ConnectionManager<Conn>>: Deref<Target = Conn>,
{
    type Target = Conn;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}



impl <Conn> Connection for EphemeralDatabaseConnection<Conn>
where
    Conn: MigrationConnection + RemoteConnection + 'static,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    <Conn as diesel::Connection>::TransactionManager: TransactionManager<EphemeralDatabaseConnection<Conn>>
{
    type Backend = <Conn as Connection>::Backend;
    type TransactionManager = <Conn as Connection>::TransactionManager;

    /// Establish a connection to the database.
    ///
    /// # Note
    ///
    /// This requires a url pointing to a database that has authority to create other databases.
    fn establish(database_url: &str) -> ConnectionResult<Self> {
        let conn = Conn::establish(database_url)?;
        let origin = {
            let split = database_url.split("/");
            let take = split.clone().count() - 1;
            split.take(take).collect::<String>()
        };
        crate::TestDatabaseBuilder::new(conn, &origin)
            .db_name_prefix("diesel_test_setup")
            .setup_connection()
            .map_err(|e| {
                match e {
                    TestDatabaseError::ConnectionError(e) => e,
                    _ => panic!()
                }
            })
    }

    fn execute(&self, query: &str) -> QueryResult<usize> {
        self.connection.execute(query)
    }

    fn query_by_index<T, U>(&self, source: T) -> QueryResult<Vec<U>>
    where
        T: AsQuery,
        T::Query: QueryFragment<Self::Backend> + QueryId,
        Self::Backend: HasSqlType<T::SqlType>,
        U: Queryable<T::SqlType, Self::Backend>
    {
        self.connection.query_by_index(source)
    }

    fn query_by_name<T, U>(&self, source: &T) -> QueryResult<Vec<U>>
    where
        T: QueryFragment<Self::Backend> + QueryId,
        U: QueryableByName<Self::Backend>
    {
        self.connection.query_by_name(source)
    }

    fn execute_returning_count<T>(&self, source: &T) -> QueryResult<usize>
    where
        T: QueryFragment<Self::Backend> + QueryId
    {
        self.connection.execute_returning_count(source)
    }

    fn transaction_manager(&self) -> &Self::TransactionManager {
        self.connection.transaction_manager()
    }
}

impl <Conn> SimpleConnection for EphemeralDatabaseConnection<Conn>
where
    Conn: MigrationConnection + RemoteConnection + SimpleConnection + 'static,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
{
    fn batch_execute(&self, query: &str) -> QueryResult<()> {
        self.connection.batch_execute(query)

    }
}
