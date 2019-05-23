use crate::primitives::drop_database;
use diesel::Connection;

/// Drops test databases when it exits scope.
///
/// Contains the admin connection and the name of the database.
/// When this struct goes out of scope, it will use the data it owns to drop the database it's
/// associated with.
///
/// # Warning
/// ### When dealing with tuple of type `(Conn, Cleanup)` or `(Pool, Cleanup)`
/// * Proper database cleanup requires that `Cleanup` is dropped _after_ the connection.
/// * Failure to assign the connection out of the tuple returned from this function will cause the
/// `Cleanup` struct to be dropped first.
/// If `Cleanup` drops first, an error indicating that the database is still in use will be thrown
/// and the database will not be dropped, polluting your RDBMS namespace with test databases.
#[derive(Debug)]
pub struct Cleanup<Conn>(pub(crate) Conn, pub(crate) String)
where
    Conn: Connection,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword;

impl<Conn> Drop for Cleanup<Conn>
where
    Conn: Connection,
    <Conn as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
{
    fn drop(&mut self) {
        drop_database(&self.0, &self.1).expect("Couldn't drop database at end of test.");
    }
}
