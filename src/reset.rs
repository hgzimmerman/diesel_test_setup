//! Functions for resetting the database and running migrations on it.

use crate::{
    database_error::{DatabaseError, DatabaseResult},
    query_helper,
};
use diesel::{query_dsl::RunQueryDsl, Connection};
use migrations_internals as migrations;
use migrations_internals::MigrationConnection;
use std::path::Path;

table! {
    pg_database (datname) {
        datname -> Text,
        datistemplate -> Bool,
    }
}


/// Drops the database, completely removing every table (and therefore every row) in the database.
pub fn drop_database<T>(admin_conn: &T, database_name: &str) -> DatabaseResult<()>
where
    T: Connection,
    <T as Connection>::Backend: diesel::backend::SupportsDefaultKeyword
{
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
}


pub fn create_database<T>(admin_conn: &T, database_name: &str) -> DatabaseResult<()>
where
    T: Connection,
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
pub fn run_migrations<T>(normal_conn: &T, migrations_directory: &Path) -> Result<(), DatabaseError>
where
    T: MigrationConnection,
    <T as Connection>::Backend: diesel::backend::SupportsDefaultKeyword
{
    migrations::run_pending_migrations_in_directory(normal_conn, migrations_directory, &mut ::std::io::sink())
        .map_err(DatabaseError::from)
}


#[cfg(test)]
pub mod test_util {
    use super::*;
    use diesel::{query_dsl::RunQueryDsl, ExpressionMethods, OptionalExtension, PgConnection, QueryDsl, QueryResult};
    use diesel::dsl::sql;

    /// Does the database with the given name exist?
    ///
    /// Utility function that may be of some use in the future.
    pub fn database_exists(conn: &PgConnection, database_name: &str) -> QueryResult<bool> {
        use self::pg_database::dsl::*;
        pg_database
            .select(datname)
            .filter(datname.eq(database_name))
            .filter(datistemplate.eq(false))
            .get_result::<String>(conn)
            .optional()
            .map(|x| x.is_some())
    }

    /// Indicates if the current connection has superuser privileges.
    ///
    /// Utility function that may be of some use in the future.
    #[allow(dead_code)]
    pub fn is_superuser(conn: &PgConnection) -> QueryResult<bool> {
        // select usesuper from pg_user where usename = CURRENT_USER;

        table! {
            pg_user (usename) {
                usename -> Text,
                usesuper -> Bool,
            }
        }
        pg_user::table
            .select(pg_user::usesuper)
            .filter(sql("usename = CURRENT_USER"))
            .get_result::<bool>(conn)
    }

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
}




