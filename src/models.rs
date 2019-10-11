//! This module holds items related to data manipulation for user object
use std::{
    fmt::{Display, Formatter, Result as FormatResult},
};
use crate::schema::users;
use crate::error::Result as FmtResult;

use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::{Connection, QueryResult, Queryable, Identifiable};
use diesel::pg::PgConnection;
use serde::{Deserialize, Serialize};
use uuid::{Uuid, parser::ParseError};
use validator::Validate;
use validator_derive::Validate;

/// UserUuid is a wrapper for Uuid to allow public properties since User.uuid
/// is a private field by diesel standards
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Default, Hash, Eq, PartialOrd, Ord)]
pub struct UserUuid(pub Uuid);

impl UserUuid {
    pub fn to_query_parameter(self) -> String {
        format!("{}={}", PARAM_NAME, self.0)
    }
    pub fn parse_str(input: &str) -> Result<Self, ParseError> {
        Uuid::parse_str(input).map(UserUuid)
    }
}

impl AsRef<Uuid> for UserUuid {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

const PARAM_NAME: &str = "user_uuid";

impl Display for UserUuid {
    fn fmt(&self, f: &mut Formatter) -> FormatResult {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for UserUuid {
    fn from(uuid: Uuid) -> UserUuid {
        UserUuid(uuid)
    }
}

/// A struct representing all the fields in the 'users' table.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Queryable, Identifiable, TypeName)]
#[primary_key(uuid)]
#[table_name = "users"]
pub struct User {
    /// user id
    pub uuid: Uuid,
    /// account's user name
    pub user_name: String,
    /// public display name
    pub display_name: String,
    /// email address
    pub email: String,
    /// hashed password
    pub password: String,
    /// date time of when 'users' row was created
    pub created_at: NaiveDateTime,
    /// date time of when 'users' row was last updated
    pub updated_at: NaiveDateTime,
    /// is the user active? defaults to true
    pub is_active: bool,
    /// is the user verified through email? defaults to false
    pub is_verified: bool,
}

/// Temporarily struct for new user data, user record for new user entries.
#[derive(Debug, Clone, Insertable, Validate)]
#[table_name = "users"]
pub struct NewUser {
    #[validate(length(min = 3, max = 40))]
    pub user_name: String,


    #[validate(length(min = 3, max = 40))]
    pub display_name: String,

    #[validate(length(max = 255, message = "Maximum of 255 chars allowed as email field"))]
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 6, message = "Insecure password. Minimum of 6 chars required"))]
    pub password: String,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub is_active: bool,
    pub is_verified: bool,
}

impl User {
    /// Create a new user method returns Result<Option<User>>
    /// NewUser struct must be initialized prior to this constructor method.
    pub fn new(new_user: NewUser, conn: &PgConnection) ->
        FmtResult<User>
    {
        use crate::{schema, storage::calls::create_row};

        create_row::<User, NewUser, _>(schema::users::table, new_user, conn)
            // use crate::schema::users::dsl::users;

            // conn.transaction(|| {
            //     let may_insert_data = diesel::insert_into(users)
            //         .values(&self)
            //         .on_conflict_do_nothing()
            //         .get_result::<User>(conn)
            //         .optional()?;

            //     Ok(may_insert_data)
            // })
    }

    /// Query user by User.uuid or return database error.
    pub fn get_by_id(conn: &PgConnection, uuid: UserUuid) -> FmtResult<User> {
        use crate::{schema, storage};

        storage::calls::get_row::<User, _>(schema::users::table, uuid.0, conn)
    }

    ///// Query user by User.id or error out.
    //pub fn get_by_id(conn: &PgConnection, user_id: Uuid) ->
    //    QueryResult<Option<User>>
    //{
    //    use crate::schema::users::dsl::{id, users};

    //    // let fmt_not_found = format!("User {} not found", user_id);
    //    conn.transaction(|| {
    //        let user = users
    //            .filter(id.eq(user_id.to_string()))
    //            .first::<User>(conn)
    //            .optional()?;
    //        //.map_err(|_| APIError::NotFound(fmt_not_found));

    //        Ok(user.into())
    //    })
    //}

    /// Query user by username.
    pub fn get_by_username(conn: &PgConnection, username: &str) -> QueryResult<Option<User>>
    {
        users::table
            .filter(users::user_name.ilike(username))
            .first(conn)
            .optional()
    }

    /// Query user by email address.
    pub fn get_by_email(conn: &PgConnection, email: &str) -> QueryResult<Option<User>>
    {
        users::table
            .filter(users::email.ilike(email))
            .first(conn)
            .optional()
    }
}

// impl NewUser {
//     /// NewUser struct builder
// pub fn new(
//     user_name: String,
//     email: String,
//     display_name: String,
//     password: String,
//     created_at: NaiveDateTime,
//     updated_at: NaiveDateTime,
//     is_active: bool,
//     is_verified: bool,

// ) -> Self {
//     NewUser {
//         id: Uuid::new_v4().to_string(),
//         user_name: user_name.to_lowercase(),
//         display_name: user_name,
//         email: email.to_lowercase(),
//         password,
//         created_at: Utc::now().naive_utc(),
//         updated_at: Utc::now().naive_utc(),
//         is_active: true,
//         is_verified: false,
//     }
// }

//
