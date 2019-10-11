use std::env;

use bytes::Buf;
use diesel::{
    pg::PgConnection,
    r2d2::{Pool, PooledConnection, ConnectionManager},
};
use futures::future::{self, Future};
use lazy_static;
use regex::Regex;
use serde_json;
use serde::de::DeserializeOwned;
use validator::ValidationError;
use uuid::Uuid;
use warp::{self, Filter, Reply, Rejection};

use crate::exception::{self, INTERNAL_SERVER_ERROR};

/// Holds a bunch of db connections and hands them out to routes as needed.
type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgPooled = PooledConnection<ConnectionManager<PgConnection>>;

/// pg_pool initializes the PostgreSQL connection pool.
pub fn pg_pool() -> PgPool {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    let pool = Pool::new(manager)
        .expect("PostgreSQL connection pool could not be initialized");
    log::info!("initiated postgresSQL thread connection pool");

    pool
}

/// Run a function on a threadpool, returning a future resolving when the function completes.
pub fn fut_threadpool<F, T>(f: F) -> impl Future<Item = T, Error = tokio_threadpool::BlockingError>
where
    F: FnOnce() -> T,
{
    let mut f_only_once = Some(f);
    futures::future::poll_fn(move || {
        tokio_threadpool::blocking(|| {
            let f = f_only_once.take().unwrap();
            f()
        })
    })
}

/// Run a function on a threadpool, returning a future resolving when the
/// function completes.  Any (unexpected!) threadpool error is turned into a
/// Warp rejection, wrapping the Internal Server Error problem.
pub fn threadpool<F, T>(f: F) -> impl Future<Item = T, Error = Rejection>
where
    F: FnOnce() -> T,
{
    fut_threadpool(f).map_err(|_| warp::reject::custom(INTERNAL_SERVER_ERROR))
}

/// Runs a function on a threadpool, ignoring a potential Diesel error inside the threadpool.
/// This error is turned into an internal server error (as Diesel errors are unexpected, and
/// indicative of erroneous queries).
pub fn threadpool_diesel_ok<F, T>(f: F) -> impl Future<Item = T, Error = Rejection>
where
    F: FnOnce() -> Result<T, diesel::result::Error>,
{
    threadpool(f).and_then(|result| match result {
        Ok(v) => future::ok(v),
        Err(_) => future::err(warp::reject::custom(INTERNAL_SERVER_ERROR)),
    })
}

/// Flatten a nested result with equal error types to a single result.
pub fn flatten_result<T, E>(nested: Result<Result<T, E>, E>) -> Result<T, E> {
    match nested {
        Err(e) => Err(e),
        Ok(v) => v,
    }
}

/// Create a filter to get a PostgreSQL connection from a PostgreSQL connection pool.
pub fn pg(
    pg_pool: crate::utils::PgPool,
) -> impl Filter<Extract = (crate::utils::PgPooled,), Error = Rejection> + Clone {
    warp::any()
        .map(move || pg_pool.clone())
        .and_then(|pg_pool: crate::utils::PgPool| match pg_pool.get() {
            Ok(pg_pooled) => Ok(pg_pooled),
            Err(_) => Err(warp::reject::custom(INTERNAL_SERVER_ERROR)),
        })
}

/// matches generic Result to Ok() -or- internal server error.
pub fn ok_or_internal_error<T, E>(r: Result<T, E>) -> Result<T, Rejection> {
    match r {
        Ok(value) => Ok(value),
        Err(_) => Err(warp::reject::custom(INTERNAL_SERVER_ERROR)),
    }
}

/// matches generic Result with Option<T> -or- internal server error.
pub fn some_or_internal_error<T>(r: Option<T>) -> Result<T, Rejection> {
    match r {
        Some(value) => Ok(value),
        None => Err(warp::reject::custom(INTERNAL_SERVER_ERROR)),
    }
}

/// Create a filter to deserialize a request.
pub fn deserialize<T>() -> impl Filter<Extract = (T,), Error = Rejection> + Copy
where
    T: DeserializeOwned + Send,
{
    // Allow a request of at most 64 KiB
    const CONTENT_LENGTH_LIMIT: u64 = 1024 * 64;

    warp::body::content_length_limit(CONTENT_LENGTH_LIMIT)
        .or_else(|_| {
            Err(warp::reject::custom(exception::Fault::PayloadTooLarge {
                limit: CONTENT_LENGTH_LIMIT,
            }))
        })
    .and(warp::body::concat())
        .and_then(|body_buffer: warp::body::FullBody| {
            let body: Vec<u8> = body_buffer.collect();

            serde_json::from_slice(&body).map_err(|err| {
                log::debug!("Request JSON deserialize error: {}", err);
                warp::reject::custom(exception::Fault::InvalidJson {
                    category: (&err).into(),
                })
            })
        })
}

/// Convert rejections into replies.
pub fn handle_rejection(rejection: Rejection) -> Result<impl Reply, Rejection> {
    use crate::exception::{ExceptionMsg, Fault};

    let reply = if let Some(fault) = rejection.find_cause::<Fault>() {
        // This rejection originated in this implementation.
        let static_exception = ExceptionMsg::from(fault);

        warp::reply::with_status(
            serde_json::to_string(&static_exception).unwrap(),
            fault.to_status_code(),
        )
    } else {
        // This rejection originated in Warp.
        let fault = if rejection.is_not_found() {
            exception::NOT_FOUND
        } else {
            exception::INTERNAL_SERVER_ERROR
        };
        let static_exception = ExceptionMsg::from(&fault);

        warp::reply::with_status(
            serde_json::to_string(&static_exception).unwrap(),
            fault.to_status_code(),
        )
    };

    Ok(warp::reply::with_header(
            reply,
            "Content-Type",
            "application/fault+json",
    )
    )
}

/// Validates Passwords
/// - Ensures the password inputs match a required regex pattern
///
///
///  # Returns
///
///  ## ValidationError
/// If the validation fails
pub fn validate_pass(pass: &str) -> Result<(), ValidationError> {
    lazy_static! {
        static ref PASSWORD: Regex = Regex::new(r"^.{6,25}$").unwrap();
    }
    if !PASSWORD.is_match(pass) {
        return Err(ValidationError::new(
                "Password should contain:\n At least 6 characters",
        ));
    }
    Ok(())
}

pub fn uuid_filter() -> warp::filters::BoxedFilter<(Uuid,)> {
    warp::path::param().boxed()
}

pub fn uuid_wrap_filter<T>() -> warp::filters::BoxedFilter<(T,)>
where
    T: From<Uuid> + Send + 'static,
{
    warp::path::param().map(T::from).boxed()
}
