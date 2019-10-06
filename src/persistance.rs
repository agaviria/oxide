use crate::config::Cfg;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

type PgPool = Pool<ConnectionManager<PgConnection>>;
type PgPooled = PooledConnection<ConnectionManager<PgConnection>>;

fn pg_pool() -> PgPool {
    let manager =
        ConnectionManager::<PgConnection>::new("postgres://ag@localhost/oxide");
}
