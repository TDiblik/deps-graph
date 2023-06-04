#![recursion_limit = "9999"] // If package has more than this number of features, something is wrong :DD

mod models;
mod routes;
mod utils;

use axum::{routing::get, Json, Router};
use routes::cargo_router::CargoRouter;
use serde::Serialize;
use serde_json::{json, Value};
use std::net::SocketAddr;
use utils::app_error::AppError;

use redis::aio::Connection;

#[derive(Clone, Serialize)]
pub struct AppState {
    redis_conn_string: String,
}
impl AppState {
    fn new(redis_conn_string: String) -> Self {
        AppState { redis_conn_string }
    }
    pub async fn get_redis_conn(&self) -> anyhow::Result<Connection> {
        let redis_client = redis::Client::open(self.redis_conn_string.clone())?;
        Ok(redis_client.get_async_connection().await?)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app_state = AppState::new("redis://127.0.0.1:7500/".into());

    let v1_router = Router::new().merge(CargoRouter::init(app_state.clone()));
    let app = Router::new()
        .route("/", get(handler))
        .with_state(app_state.clone())
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

async fn handler() -> Result<Json<Value>, AppError> {
    Ok(Json(json!({"msg": "Hello world!"})))
}
