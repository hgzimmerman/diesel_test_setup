use crate::core::drop_database;
use crate::setup::*;
use crate::test_util::{
    database_exists, MYSQL_ADMIN_URL, MYSQL_ORIGIN, POSTGRES_ADMIN_URL, POSTGRES_ORIGIN,
};
use crate::{Pool};
use diesel::{Connection, MysqlConnection, PgConnection};
use std::ops::Deref;
use std::path::Path;

#[test]
fn cleanup_drops_db_after_panic() {
    let url_origin = POSTGRES_ORIGIN;
    let db_name = "cleanup_drops_db_after_panic_TEST_DB".to_string();

    // Make sure that the db doesn't exist beforehand.
    {
        let admin_conn = PgConnection::establish(POSTGRES_ADMIN_URL)
            .expect("Should be able to connect to admin db");
        drop_database(&admin_conn, &db_name).expect("should drop");;
    }

    std::panic::catch_unwind(|| {
        let admin_conn = PgConnection::establish(POSTGRES_ADMIN_URL)
            .expect("Should be able to connect to admin db");
        let _ = setup_named_db_pool(
            admin_conn,
            url_origin,
            Path::new("test_assets/postgres/migrations"),
            db_name.clone(),
        )
        .expect("create db");
        panic!("expected_panic");
    })
    .expect_err("Should catch panic.");

    let admin_conn =
        PgConnection::establish(POSTGRES_ADMIN_URL).expect("Should be able to connect to admin db");
    let database_exists: bool =
        database_exists(&admin_conn, &db_name).expect("Should determine if database exists");
    assert!(!database_exists)
}

#[test]
fn cleanup_drops_database() {
    let url_origin = POSTGRES_ORIGIN;
    let db_name = "cleanup_drops_database_TEST_DB".to_string();

    let admin_conn =
        PgConnection::establish(POSTGRES_ADMIN_URL).expect("Should be able to connect to admin db");
    // precautionary drop
    drop_database(&admin_conn, &db_name).expect("should drop");

    let pool_and_cleanup = setup_named_db_pool(
        admin_conn,
        url_origin,
        Path::new("test_assets/postgres/migrations"),
        db_name.clone(),
    )
    .unwrap();

    let admin_conn =
        PgConnection::establish(POSTGRES_ADMIN_URL).expect("Should be able to connect to admin db");

    let db_exists: bool =
        database_exists(&admin_conn, &db_name).expect("Should determine if database exists");
    assert!(db_exists);

    std::mem::drop(pool_and_cleanup);

    let db_exists: bool =
        database_exists(&admin_conn, &db_name).expect("Should determine if database exists");
    assert!(!db_exists)
}

#[test]
fn lack_of_assignment_still_allows_correct_drop_order() {
    let url_origin = POSTGRES_ORIGIN;
    let db_name = "lack_of_assignment_still_allows_correct_drop_order_TEST".to_string();

    let admin_conn =
        PgConnection::establish(POSTGRES_ADMIN_URL).expect("Should be able to connect to admin db");
    // precautionary drop
    drop_database(&admin_conn, &db_name).expect("should drop");

    setup_named_db_pool(
        admin_conn,
        url_origin,
        Path::new("test_assets/postgres/migrations"),
        db_name.clone(),
    )
    .unwrap();
}

#[test]
fn normal_assignment_allows_correct_drop_order() {
    let url_origin = POSTGRES_ORIGIN;
    let db_name = "normal_assignment_allows_correct_drop_order_TEST".to_string();

    let admin_conn =
        PgConnection::establish(POSTGRES_ADMIN_URL).expect("Should be able to connect to admin db");
    // precautionary drop
    drop_database(&admin_conn, &db_name).expect("should drop");

    let _pool_and_cleanup = setup_named_db_pool(
        admin_conn,
        url_origin,
        Path::new("test_assets/postgres/migrations"),
        db_name.clone(),
    )
    .unwrap();
}

#[test]
fn late_assignment_allows_correct_drop_order() {
    let url_origin = POSTGRES_ORIGIN;
    let db_name = "late_assignment_allows_correct_drop_order_TEST".to_string();

    let admin_conn =
        PgConnection::establish(POSTGRES_ADMIN_URL).expect("Should be able to connect to admin db");
    // precautionary drop
    drop_database(&admin_conn, &db_name).expect("should drop");

    let x = setup_named_db_pool(
        admin_conn,
        url_origin,
        Path::new("test_assets/postgres/migrations"),
        db_name.clone(),
    )
    .unwrap();
    let _pool = x.pool;
}

#[test]
fn deref_out_of_function_maintains_correct_drop_order() {
    let url_origin = POSTGRES_ORIGIN;
    let db_name = "deref_should_break_TEST".to_string();

    let admin_conn =
        PgConnection::establish(POSTGRES_ADMIN_URL).expect("Should be able to connect to admin db");
    // precautionary drop
    drop_database(&admin_conn, &db_name).expect("should drop");

    let _: &Pool<PgConnection> = setup_named_db_pool(
        admin_conn,
        url_origin,
        Path::new("test_assets/postgres/migrations"),
        db_name.clone(),
    )
    .unwrap()
    .deref();
}

#[test]
fn mysql() {
    let url_origin = MYSQL_ORIGIN;
    let db_name = "mysql_TEST".to_string();

    let admin_conn =
        MysqlConnection::establish(MYSQL_ADMIN_URL).expect("Should be able to connect to admin db");

    drop_database(&admin_conn, &db_name).expect("should drop");

    let _ = setup_named_db_pool(
        admin_conn,
        url_origin,
        Path::new("test_assets/mysql/migrations"),
        db_name.clone(),
    )
    .unwrap();
}

// TODO move this to a separate test suite where this won't break other tests
//#[test]
//fn establish_for_ephemeral_connection() {
//    std::env::set_current_dir(std::env::current_dir().unwrap().join(Path::new("test_assets/postgres")));
//    let test_db_conn = EphemeralDatabaseConnection::<PgConnection>::establish(POSTGRES_ADMIN_URL).expect("Should establish connection and create database");
//    std::env::set_current_dir(Path::new("../.."));
//}
