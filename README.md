# Diesel Reset

A small library for resetting a database using Diesel.

The core exported function prevents parallel tests from running.


#### Todo
It may be a better strategy to implement Drop on some wrapper around a connection pool, and use a different, random, database name for each test.
This won't be really resetting, but using Drop should ensure that if a panic occurs, the database environment isn't polluted by broken tests.