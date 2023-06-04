mod models;
mod routes;
mod utils;

use axum::{routing::get, Router};
use redis::aio::ConnectionManager;
use routes::cargo_router::CargoRouter;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let manager = bb8_redis::RedisConnectionManager::new("redis://127.0.0.1:7500/")?;
    let pool = bb8::Pool::builder().build(manager).await?;

    let v1_router = Router::new().merge(CargoRouter::init(pool.clone()));
    let app = Router::new()
        .route("/", get(handler))
        .nest("/api/v1", v1_router);

    // Address that server will bind to.
    let addr = SocketAddr::from(([127, 0, 0, 1], 50001));

    // Use `hyper::server::Server` which is re-exported through `axum::Server` to serve the app.
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn handler() -> &'static str {
    "Hello, world!"
}
