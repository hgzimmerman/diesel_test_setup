# Diesel Test Setup

A small library for setting up a database using Diesel, and then tearing down the database once the test is finished.

Given a connection to a database that has super user permissions, this library will create a new, uniquely named database.
Once a `Cleanup` struct that was created when the database was set up goes out of scope, its destructor will delete the database.


```rust
use diesel_test_setup::TestDatabaseBuilder;

{
    let admin_conn = PgConnection::establish(ADMIN_DATABASE_URL).unwrap();
    const DATABASE_ORIGIN: &str = "postgres://localhost";
    let (_cleanup, pool) = TestDatabaseBuilder::new(
        admin_conn,
        DATABASE_ORIGIN
    )
    .setup_pool()
    .expect("Could not create the database.");

    // Perform your test using pool
}

// Database has been cleaned up.
```


### Features
* Supports Postgres and MySql.
  * MySql is untested, although it should just work.

