# Diesel Test Setup

A small library for setting up a database using Diesel, and then tearing down the database once the test is finished.

Given a connection to a database that has super user permissions, this library will create a new, uniquely named database.
Once a `Cleanup` struct that was created when the database was set up goes out of scope, its destructor will delete the database.


### Features
* Supports Postgres, MySql, and squite.
  * MySql and Sqlite are untested, although they should just work.
    * The core exported functions use standard SQL, which should work across all databases supported by Diesel.

