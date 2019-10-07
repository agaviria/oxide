//! This module holds items related to data manipulation for user object

use crate::schema::users;

use chrono::{prelude::*, NaiveDateTime};
use diesel::prelude::*;
use diesel::{Connection, QueryResult, Queryable, Identifiable};
use diesel::pg::PgConnection;
use validator::Validate;
use validator_derive::Validate;

#[derive(Copy, Clone, Debug, Eq, Hash, Identifiable, PartialEq)]
#[table_name = "users"]
pub struct UserId(
    #[column_name = "id"]
    pub i32
);

#[derive(Clone, Debug, PartialEq, Eq, Queryable, Identifiable)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    display_name: String,
    password: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
    is_active: bool,
    is_verified: bool,
}

/// User trait implementation
impl User {
    /// Query user by id.
    pub fn by_id(conn: &PgConnection, id: UserId) -> QueryResult<Option<User>>
    {
        users::table
            .find(id.0)
            .first(conn)
            .optional()
    }

    /// Query user by username.
    pub fn by_username(conn: &PgConnection, username: &str) -> QueryResult<Option<User>>
    {
        users::table
            .filter(users::username.ilike(username))
            .first(conn)
            .optional()
    }

    /// Query user by email address.
    pub fn by_email(conn: &PgConnection, email: &str) -> QueryResult<Option<User>>
    {
        users::table
            .filter(users::email.ilike(email))
            .first(conn)
            .optional()
    }
}

/// Temporarily struct for new user data, user record for new user entries.
#[derive(Debug, Clone, Insertable, Validate)]
#[table_name = "users"]
pub struct NewUser {
    #[validate(length(min = 3, max = 40))]
    pub username: String,

    #[validate(length(max = 255, message = "Maximum of 255 chars allowed as email field"))]
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 3, max = 40))]
    pub display_name: String,

    #[validate(length(min = 6, message = "Insecure password. Minimum of 6 chars required"))]
    pub password: String,
    is_active: bool,
    is_verified: bool,
}

impl NewUser {
    /// NewUser struct builder
    pub fn new(
        username: String,
        password: String,
        email: String,
    ) -> Self {
        NewUser {
            username: username.to_lowercase(),
            display_name: username,
            password,
            email: email.to_lowercase(),
            is_active: true,
            is_verified: false,
        }
    }

    /// create new user method returns Result<Option<User>>
    pub fn create(&self, conn: &PgConnection) -> QueryResult<Option<User>>
    {
        use crate::schema::users::dsl::*;

        conn.transaction(|| {
            let may_insert_data = diesel::insert_into(users)
                .values(self)
                .on_conflict_do_nothing()
                .get_result::<User>(conn)
                .optional()?;

            Ok(may_insert_data)
        })
    }
}
