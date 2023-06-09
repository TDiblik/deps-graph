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
use redis::{aio::Connection, AsyncCommands};
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
    root_features: Option<String>,
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
    #[cfg(debug_assertions)]
    let time_to_traverse = std::time::Instant::now();

    let root_features_raw = query.root_features.unwrap_or("".to_owned());
    let root_features = root_features_raw
        .split(',')
        .map(|s| s.to_string())
        .collect();
    let root_include_default_features = query.root_include_default_features.unwrap_or(true);
    let include_normal_dependencies = query.include_normal_dependencies.unwrap_or(true);
    let include_build_dependencies = query.include_build_dependencies.unwrap_or(false);
    let include_dev_dependencies = query.include_dev_dependencies.unwrap_or(false);
    let redis_cache_traversal_key = format!(
        "{}-{}-{}-{}-{}",
        id,
        root_features_raw,
        include_normal_dependencies,
        include_build_dependencies,
        include_dev_dependencies
    );

    let mut redis_conn: Connection = app_state.get_redis_conn().await?;
    let cached_result: Option<String> = redis_conn.get(redis_cache_traversal_key.clone()).await?;
    if cached_result.is_some() {
        // Parsing takes shit tone of time, fix in future.
        let parsed_cached_result: serde_json::Value =
            serde_json::from_str(cached_result.unwrap().as_str())?;

        #[cfg(debug_assertions)]
        println!(
            "Time it took to find in cache and return {}: {:.2?}",
            id,
            time_to_traverse.elapsed()
        );
        return Ok(Json(parsed_cached_result));
    }

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
        root_features,
        root_include_default_features,
        include_normal_dependencies,
        include_build_dependencies,
        include_dev_dependencies,
    )
    .await?;
    let json_answ = json!(answ);
    redis_conn
        .set(redis_cache_traversal_key, json_answ.to_string())
        .await?;

    #[cfg(debug_assertions)]
    println!(
        "Time it took to traverse {}: {:.2?}",
        id,
        time_to_traverse.elapsed()
    );
    Ok(Json(json_answ))
}
