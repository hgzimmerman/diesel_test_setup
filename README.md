# Diesel Test Setup

A small library for setting up a database using Diesel, and then tearing down the database once the test is finished.

Given a connection to a database that has super user permissions, this library will create a new, uniquely named database.
Once a `Cleanup` struct that was created when the database was set up goes out of scope, its destructor will delete the database.


```rust
use diesel_test_setup::TestDatabaseBuilder;

{
    let admin_conn = PgConnection::establish(ADMIN_DATABASE_URL).unwrap();
    const DATABASE_ORIGIN: &str = "postgres://localhost";
    let pool: EphemeralDatabasePool<PgConnection> = TestDatabaseBuilder::new(
        admin_conn,
        DATABASE_ORIGIN
    )
    .setup_pool()
    .expect("Could not create the database.");

    let pool: &Pool<ConnectionManager<PgConnection>> = pool.deref();
    // Perform your test using the reference to a Pool
}

// Database has been cleaned up.
```

### Testing
Currently, tests only cover PostgreSql, and rely on environment configurations that are external to this project.
For those reasons, you shouldn't expect tests to run out-of-the-box.
Some effort is being directed towards setting up a testing environment using Docker so that both PostgreSql and MySql can be tested.


### Features
* Supports PostgreSql and MySql.
  * MySql is untested, although it should just work.
* Both `r2d2::Pool`s and `diesel::Connection`s are supported.
* Automatic destruction of test databases.


### Support Commitment
The scope and features of this project are pretty minimal and should be easy to maintain.
I consider this project to be feature-complete, although if you find something lacking, feel free to open an Issue or PR.
The crate is listed under `passively-maintained` in its `Cargo.toml`, should I cease to be able to maintain this crate,
I will alter that tag to `looking-for-maintainer`
