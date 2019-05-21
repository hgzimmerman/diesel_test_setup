use crate::reset::drop_database;
use diesel::Connection;

/// Cleanup wrapper.
/// Contains the admin connection and the name of the database (not the whole url).
///
/// When this struct goes out of scope, it will use the data it owns to drop the database it's
/// associated with.
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
