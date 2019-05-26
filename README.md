# Diesel Test Setup

[![Build Status](https://travis-ci.org/hgzimmerman/diesel_test_setup.svg?branch=master)](https://travis-ci.org/hgzimmerman/diesel_test_setup)

A small library for setting up a database using Diesel, and then tearing down the database once the test is finished.

Given a connection to a database that has super user permissions, this library will create a new, uniquely-named database and run migrations on it.
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
The tests expect Postgres and MySql to be running and for specific environment variables to be set up before running.
Tests are intended to be ran using Docker with the following command `docker-compose run cargo test`.


### Features
* Creation of unique test databases and running of migrations.
* Automatic destruction of test databases.
* Supports PostgreSql and MySql.
* Both `r2d2::Pool`s and `diesel::Connection`s are supported.


### Wait!
Before you choose to use this library, there may be better options for your testing needs.
Take a look at Diesel's built-in [test_transaction](https://docs.diesel.rs/diesel/connection/trait.Connection.html#method.test_transaction).
Diesel Test Setup has higher overhead per-test because it needs to create, migrate, and delete a database for every test.
`test_transaction`, on the other hand, runs your code within a transaction and then rolls it back.

This library is well suited to running tests in an environment where there isn't significant initial test data, and you don't want to manually configure a separate test database than your development database.
It also is useful when you want to set up integration tests that rely on a server framework owning a `r2d2::Pool`, which excludes the possibility of using `test_transaction`.
Using `test_transaction` works great when you have an already populated test database, and want to be able to run large queries against it without waiting for setup/data-population steps.




### Support Commitment
The scope and features of this project are pretty minimal and should be easy to maintain.
I consider this project to be feature-complete, although if you find something lacking, feel free to open an Issue or PR.
The crate is listed under `passively-maintained` in its `Cargo.toml`, should I cease to be able to maintain this crate,
I will alter that tag to `looking-for-maintainer`.
