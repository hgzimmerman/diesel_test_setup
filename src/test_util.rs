use diesel::{query_dsl::RunQueryDsl, ExpressionMethods, OptionalExtension, PgConnection, QueryDsl, QueryResult};
use diesel::dsl::sql;
use diesel::table;

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
    use crate::setup::test::DROP_DATABASE_URL;

    #[test]
    fn is_super() {
        let admin_conn = PgConnection::establish(DROP_DATABASE_URL)
            .expect("Should be able to connect to admin db");
        let is_super = is_superuser(&admin_conn).expect("Should get valid response back");
        assert!(is_super)
    }
}



