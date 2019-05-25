use diesel::{
    dsl::sql, query_dsl::RunQueryDsl, table, ExpressionMethods, OptionalExtension, PgConnection,
    QueryDsl, QueryResult,
};

/// Should point to the base postgres account.
/// One that has authority to create and destroy other database instances.
///
/// It is expected to be on the same database server as the one indicated by DATABASE_ORIGIN.
pub const POSTGRES_ADMIN_URL: &str = env!("POSTGRES_ADMIN_URL");
/// The origin (scheme, user, password, address, port) of the test database.
///
/// This determines which database server is connected to, but allows for specification of
/// a specific database instance within the server to connect to and run tests with.
pub const POSTGRES_ORIGIN: &str = env!("POSTGRES_DB_ORIGIN");

pub const MYSQL_ADMIN_URL: &str = env!("MYSQL_ADMIN_URL");
pub const MYSQL_ORIGIN: &str = env!("MYSQL_DB_ORIGIN");

table! {
    pg_database (datname) {
        datname -> Text,
        datistemplate -> Bool,
    }
}
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

    #[test]
    fn is_super() {
        let admin_conn = PgConnection::establish(POSTGRES_ADMIN_URL)
            .expect("Should be able to connect to admin db");
        let is_super = is_superuser(&admin_conn).expect("Should get valid response back");
        assert!(is_super)
    }
}
