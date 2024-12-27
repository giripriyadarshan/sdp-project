# Software Development Practice (SDP) Project

## Problem Statement

Online Retail Management System — A person can browse the offerings anonymously but to make a purchase they must create
an account. The customer will
provide their information (delivery location, payment information etc … can have multiple). A customer can purchase one
or more items at a time & in different quantities. There are different types of items for sale & based on the quantity (
or
other factors) the price of the item may be discounted. Bills will be generated & the customer must pay at time of
order.
The items can be sourced from multiple suppliers. Think Amazon or any other online retailer.

## Libraries Used for the Backend

- [axum](https://github.com/tokio-rs/axum) - Web framework
- [async-graphql](https://github.com/async-graphql/async-graphql) - GraphQL server library
- [sea-orm](https://www.sea-ql.org/SeaORM/) - ORM library
- [jsonwebtoken](https://github.com/Keats/jsonwebtoken) - JWT library
- [argon2](https://github.com/RustCrypto/password-hashes/tree/master/argon2) - Password hashing library
- [And many more](https://github.com/giripriyadarshan/sdp-project/blob/main/api-server/Cargo.toml) - Check the
  Cargo.toml file for more information

## Backend Setup

1. Clone the repository
2. Navigate to the `api-server` directory
3. Setup the `.env` file in the `api-server/` with the following variables and edit the values accordingly
    ```env
    DATABASE_URL=postgresql://username:password@uri_to_the_instance/database_name
    PORT=8088
    PASSWORD_SECRET="passwords@rePassw0rd1ng"
    TOKEN_SECRET="T0k3me@ser1ous1y"
    SMTP_USERNAME="contact@domain.com"
    SMTP_PASSWORD="Pr3ttyStr0ngP@ssw0rd"
   ```
4. Run `cargo run` to start the server

## API Documentation

The API documentation can be found at `http://localhost:$PORT/` after starting the server

## Database Schema

The database schema can be found at `./schema.sql` file

## Graphql Schema for reference

Please check the `./schema.graphql` file for the graphql schema

## For frontend please check [@prashantprakashhh](https://github.com/prashantprakashhh)'s repository list
