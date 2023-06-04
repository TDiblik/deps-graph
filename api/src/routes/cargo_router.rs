use crate::{
    models::cargo_db_types::{CargoCrateVersionNode, RedisGraphParser},
    utils::{app_error::AppError, cargo::traverse_tree, constants::CARGO_GRAPH_NAME},
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use redis::aio::Connection;
use redis_graph::AsyncGraphCommands;
use serde::Deserialize;
use serde_json::{json, Value};

pub struct CargoRouter {}
impl CargoRouter {
    pub fn init(app_state: AppState) -> Router {
        Router::new().nest(
            "/cargo/",
            Router::new()
                .route("/crate/v/:version_id/traverse", get(traverse_version))
                .with_state(app_state),
        )
    }
}

#[derive(Deserialize)]
struct TraverseVersionQueryOptions {
    root_features: Option<Vec<String>>,
    root_include_default_features: Option<bool>,

    include_normal_dependencies: Option<bool>,
    include_build_dependencies: Option<bool>,
    include_dev_dependencies: Option<bool>,
}
async fn traverse_version(
    Path(id): Path<u32>,
    Query(query): Query<TraverseVersionQueryOptions>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let mut redis_conn: Connection = app_state.get_redis_conn().await?;

    let root_node_req = redis_conn
        .graph_ro_query(
            CARGO_GRAPH_NAME,
            format!("match (cv: CargoCrateVersion {{id: {id}}}) return cv"),
        )
        .await?;
    let root_node = CargoCrateVersionNode::parse(root_node_req.data.first().unwrap(), "cv")?;

    let answ = traverse_tree(
        &mut redis_conn,
        root_node,
        query.root_features.unwrap_or(vec![]),
        query.root_include_default_features.unwrap_or(true),
        query.include_normal_dependencies.unwrap_or(true),
        query.include_build_dependencies.unwrap_or(false),
        query.include_dev_dependencies.unwrap_or(false),
    )
    .await?;

    Ok(Json(json!(answ)))
}
