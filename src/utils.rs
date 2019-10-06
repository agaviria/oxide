use std::env;

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use futures::future::{self, Future};
use serde_json;
use warp::{Filter, Reply, Rejection};

use crate::exception::{self, INTERNAL_SERVER_ERROR};

type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgPooled = PooledConnection<ConnectionManager<PgConnection>>;

/// pg_pool handles the PostgreSQL connection thread pool.
pub fn pg_pool() -> PgPool {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    let pool = Pool::new(manager)
        .expect("PostgreSQL connection pool could not be created");
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

/// Create a filter to get a PostgreSQL connection from a PostgreSQL connection pool.
pub fn pg(
    pg_pool: crate::utils::PgPool,
    ) -> impl Filter<Extract = (crate::utils::PgPooled,),
    Error = Rejection> + Clone {
        warp::any()
            .map(move || pg_pool.clone())
            .and_then(|pg_pool: crate::utils::PgPool| match pg_pool.get() {
                Ok(pg_pooled) => Ok(pg_pooled),
                Err(_) => Err(warp::reject::custom(INTERNAL_SERVER_ERROR)),
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
