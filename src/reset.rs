//! Functions for resetting the database and running migrations on it.

use crate::{
    database_error::{DatabaseError, DatabaseResult},
    query_helper,
};
use diesel::{
    query_dsl::RunQueryDsl, ExpressionMethods, OptionalExtension, PgConnection, QueryDsl,
    QueryResult,
};
use migrations_internals as migrations;
use diesel::dsl::sql;

/// Drops the database, completely removing every table (and therefore every row) in the database.
pub fn drop_database(admin_conn: &PgConnection, database_name: &str) -> DatabaseResult<()> {
    if pg_database_exists(&admin_conn, database_name)? {
        //        std::thread::sleep(std::time::Duration::new(1,0));
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

/// Recreates the database.
pub fn create_database(admin_conn: &PgConnection, database_name: &str) -> DatabaseResult<()> {
    let db_result = query_helper::create_database(database_name)
        .execute(admin_conn)
        .map_err(DatabaseError::from)
        .map(|_| ());
    db_result
}

/// Creates tables in the database.
///
/// # Note
/// The connection used here should be different from the admin connection used for resetting the database.
/// Instead, the connection should be to the database on which tests will be performed on.
pub fn run_migrations(conn: &PgConnection, migrations_directory: &str) {
    use std::path::Path;

    let migrations_dir: &Path = Path::new(migrations_directory);
    migrations::run_pending_migrations_in_directory(conn, migrations_dir, &mut ::std::io::sink())
        .expect("Could not run migrations.");
}

table! {
    pg_database (datname) {
        datname -> Text,
        datistemplate -> Bool,
    }
}

/// Convenience function used when dropping the database.
pub(crate) fn pg_database_exists(conn: &PgConnection, database_name: &str) -> QueryResult<bool> {
    use self::pg_database::dsl::*;

    pg_database
        .select(datname)
        .filter(datname.eq(database_name))
        .filter(datistemplate.eq(false))
        .get_result::<String>(conn)
        .optional()
        .map(|x| x.is_some())
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
