use axum::{
    extract::{Path, State},
    routing::get,
    Router,
};
use bb8_redis::redis::Connection;

use crate::utils::cargo::traverse_tree;

type ConnectionPool = bb8::Pool<bb8_redis::RedisConnectionManager>;

pub struct CargoRouter {}
impl CargoRouter {
    pub fn init(pool: ConnectionPool) -> Router {
        Router::new().nest(
            "/cargo/",
            Router::new()
                .route("/crate/v/:version_id/traverse", get(traverse_version))
                .with_state(pool),
        )
    }
}

async fn traverse_version(Path(id): Path<u32>, State(pool): State<ConnectionPool>) -> &'static str {
    let mut conn: Connection = pool.get().await.unwrap();

    let reply: String = bb8_redis::redis::cmd("PING")
        .query_async(&mut &*conn)
        .await
        .unwrap();

    // let answ = traverse_tree();
    "Hello, world!"
}

async fn handler() -> &'static str {
    "Hello, world!"
}
