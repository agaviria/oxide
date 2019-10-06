use crate::exception::{Fault, RateLimitException};
use ratelimit_meter::{algorithms::NonConformance, KeyedRateLimiter};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use warp::{Filter, Rejection};

/// Create a filter that gates a request behind a leaky bucket rate limiter.
///
/// # Panics
/// Panics if it is used with a transport not using socket addresses.
pub fn leaky_bucket() -> impl Filter<Extract = (), Error = Rejection> + Clone {
    let limiter = Arc::new(Mutex::new(KeyedRateLimiter::<SocketAddr>::new(
                std::num::NonZeroU32::new(2u32).unwrap(),
                std::time::Duration::from_secs(1),
                )));

    warp::addr::remote()
        .and_then(move |addr: Option<SocketAddr>| {
            let addr = addr
                .expect(
                    "Must be used with a transport utilizing socket addresses."
                );
            let mut limiter = limiter.lock().unwrap();
            match limiter.check(addr) {
                Ok(_) => Ok(()),
                Err(neg) => Err(
                    warp::reject::custom(Fault::RateLimit(RateLimitException {
                        wait_time_millis: neg
                            .wait_time_from(Instant::now())
                            .as_millis() as u64,
                    }))),
            }
        })
    .untuple_one()
}
