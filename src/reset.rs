//! Functions for resetting the database and running migrations on it.

use crate::{
    database_error::{DatabaseError, DatabaseResult},
    query_helper,
};
use diesel::{query_dsl::RunQueryDsl, ExpressionMethods, OptionalExtension, PgConnection, QueryDsl, QueryResult, /*MysqlConnection, */ Connection};
use migrations_internals as migrations;
use diesel::dsl::sql;
use migrations_internals::MigrationConnection;


//pub fn run_migrations(conn: &PgConnection, migrations_directory: &str) {
//    use std::path::Path;
//
//    let migrations_dir: &Path = Path::new(migrations_directory);
//    migrations::run_pending_migrations_in_directory(conn, migrations_dir, &mut ::std::io::sink())
//        .expect("Could not run migrations.");
//}

table! {
    pg_database (datname) {
        datname -> Text,
        datistemplate -> Bool,
    }
}


/// Drops the database, completely removing every table (and therefore every row) in the database.
pub fn drop_database<T>(admin_conn: &T, database_name: &str) -> DatabaseResult<()>
where
    T: DropCreateDb + Connection,
    <T as Connection>::Backend: diesel::backend::SupportsDefaultKeyword
{
    if admin_conn.database_exists(database_name)? {
        let result = query_helper::drop_database(database_name)
            .if_exists()
            .execute(admin_conn)
            .map_err(DatabaseError::from)
            .map(|_| ());

        if let Err(DatabaseError::QueryError(diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::__Unknown,
            _,
        ))) = result
        {
            eprintln!("Could not drop DB !!!!!!!");
        }
        result
    } else {
        Ok(()) // Database has already been dropped
    }
}


pub fn create_database<T>(admin_conn: &T, database_name: &str) -> DatabaseResult<()>
where
    T: DropCreateDb + Connection,
    <T as Connection>::Backend: diesel::backend::SupportsDefaultKeyword
{
    query_helper::create_database(database_name)
        .execute(admin_conn)
        .map_err(DatabaseError::from)
        .map(|_| ())
}

/// Creates tables in the database.
///
/// # Note
/// The connection used here should be different from the admin connection used for resetting the database.
/// Instead, the connection should be to the database on which tests will be performed on.
pub fn run_migrations<T>(normal_conn: &T, migrations_directory: &str)
where
    T: DropCreateDb + MigrationConnection,
    <T as Connection>::Backend: diesel::backend::SupportsDefaultKeyword
{
    use std::path::Path;

    let migrations_dir: &Path = Path::new(migrations_directory);
    migrations::run_pending_migrations_in_directory(normal_conn, migrations_dir, &mut ::std::io::sink())
        .expect("Could not run migrations.");
}



pub trait DropCreateDb: Connection {


    fn database_exists(&self, database_name: &str) -> QueryResult<bool>;



}


impl DropCreateDb for PgConnection {
    fn database_exists(&self, database_name: &str) -> QueryResult<bool> {
        use self::pg_database::dsl::*;
        pg_database
            .select(datname)
            .filter(datname.eq(database_name))
            .filter(datistemplate.eq(false))
            .get_result::<String>(self)
            .optional()
            .map(|x| x.is_some())
    }
}



table! {
    pg_user (usename) {
        usename -> Text,
        usesuper -> Bool,
    }
}

/// Indicates if the current connection has superuser privileges.
pub fn is_superuser(conn: &PgConnection) -> QueryResult<bool> {
    // select usesuper from pg_user where usename = CURRENT_USER;
    pg_user::table
        .select(pg_user::usesuper)
        .filter(sql("usename = CURRENT_USER"))
        .get_result::<bool>(conn)
}

#[cfg(test)]
mod test {
    use super::*;
    use diesel::Connection;
    use crate::setup::test::DROP_DATABASE_URL;
    #[test]
    fn is_super() {
        let admin_conn = PgConnection::establish(DROP_DATABASE_URL)
            .expect("Should be able to connect to admin db");
        let is_super = is_superuser(&admin_conn).expect("Should get valid response back");
        assert!(is_super)
    }
}
