use chrono::Utc;
use serde::Serialize;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct UserResponse {
    pub id: Uuid,
    pub user_name: String,
    pub display_name: String,
    pub email: String,
    pub is_active: bool,
    pub is_verified: bool,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct UsersResponse(pub Vec<UserResponse>);

#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 40, message = "user name is required and must be at least 3 characters"))]
    pub user_name: String,

    #[validate(email(message = "email must be a valid email"))]
    pub email: String,

    #[validate(length(min = 3, message = "display name is required and must be at least 3 characters"))]
    pub display_name: String,

    #[validate(length(min = 6, message = "For security purposes, passwords require a minimum of 6 characters"))]
    pub password: String,
}

/// Query a user through their id field.
// pub fn get_user_by_id(id: Path<(Uuid)>, pool: Data<PoolType>,)

// impl From<User> for UserResponse {
//     fn from(user: User) -> UserResponse {
//         UserResponse {
//             id: user.id,
//             user_name: user.user_name,
//             email: user.email,

//         }
//     }
// }
