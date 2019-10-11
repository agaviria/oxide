use futures::future::Future;
use chrono::Utc;
use sentry::hash;
use serde::Deserialize;
use validator::Validate;
use warp::{filters::BoxedFilter, Filter, Rejection};

use crate::exception;
use crate::models;
use crate::utils;
use crate::payload::{ResponseBuilder, Response};

pub fn router(db: BoxedFilter<(crate::utils::PgPooled,)>,
) -> impl Filter<Extract = (Response,), Error = Rejection> + Clone {
    log::trace!("Setting up users router");

    warp::path::end()
        .and(warp::post2())
        .and(create_user(db.clone()))
}

pub fn create_user(db: BoxedFilter<(crate::utils::PgPooled,)>,
) -> impl Filter<Extract = (Response,), Error = Rejection> + Clone {
    use diesel::Connection;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct User {
        uuid: uuid::Uuid,
        user_name: String,
        display_name: String,
        email: String,
        password: String,
        created_at: NaiveDateTime,
        updated_at: NaiveDateTime,
        is_active: bool,
        is_verified: bool,
    }

    crate::utils::deserialize()
        .and(db)
        .and_then(
            |user: User, conn: crate::utils::PgPooled| {
                let user_name = user.user_name.clone();
                log::trace!("Received request to create user with username: {}", user_name);

                utils::threadpool_diesel_ok(move || {
                    conn.transaction(|| {
                        let user_by_user_name = models::User::get_by_username(&conn, &user.user_name)?;
                        let user_by_email = models::User::get_by_email(&conn, &user.email)?;

                        let hash = hash::V1Hash::hash_password(&user.password)
                            .unwrap();
                        let new_user = models::NewUser {
                            user_name: user.user_name,
                            display_name: user.display_name,
                            email: user.email,
                            password: hash.to_string(),
                            created_at: Utc::now().naive_utc(),
                            updated_at: Utc::now().naive_utc(),
                            is_active: true,
                            is_verified: false,
                        };

                        if let Err(validation_errors) = new_user.validate() {
                            let invalid_params = exception::InvalidParams::from(validation_errors);
                            return Ok(Err(warp::reject::custom(
                                        exception::Fault::InvalidParams {
                                            invalid_params
                                        }
                            )))
                        }

                        let mut invalid_params = exception::InvalidParams::new();
                        if user_by_user_name.is_some() {
                            invalid_params.add(
                                "userName",
                                exception::InvalidParamsReason::AlreadyExists
                            )
                        }

                        if user_by_email.is_some() {
                            invalid_params.add(
                                "emailAddress",
                                exception::InvalidParamsReason::AlreadyExists
                            )
                        }

                        if !invalid_params.is_empty() {
                            return Ok(Err(warp::reject::custom(
                                        exception::Fault::InvalidParams {
                                            invalid_params
                                        }
                            )))
                        }

                        let created_user = create_user(&conn, new_user).unwrap();
                        log::info!("Created user: {:?}", user_name);

                        Ok(Ok(ResponseBuilder::created().empty()));

                        Ok(Err(warp::reject::custom(
                                    exception::INTERNAL_SERVER_ERROR
                        )))
                    }

                    )
                }).then(utils::flatten_result)
            },
    )
}
