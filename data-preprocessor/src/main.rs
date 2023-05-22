use anyhow::Result;
use data_preprocessor::log_debug;
use redis_graph::*;
use sqlx::postgres::PgPoolOptions;

use data_preprocessor::constants::CARGO_GRAPH_NAME;
use data_preprocessor::utils::{
    connect_db_dependencies, gen_crate_versions_redis_graph_node_query,
    gen_crates_redis_graph_node_query, gen_dependency_redis_graph_link_query,
    gen_first_or_latest_version_redis_graph_link_query, gen_published_by_redis_graph_link_query,
    gen_users_redis_graph_node_query, gen_version_redis_graph_link_query,
    get_crate_versions_from_db_async, get_crates_from_db_async, get_raw_dependencies_from_db_async,
    get_users_from_db_async,
};

#[tokio::main]
async fn main() -> Result<()> {
    let postgres_pool = PgPoolOptions::new()
        .max_connections(4)
        .connect("postgresql://dumpuser:c4rg0DUmP@localhost:7501/dumpdb")
        .await?;

    let redis_client = redis::Client::open("redis://127.0.0.1:7500/")?;
    let mut redis_conn = redis_client.get_connection()?;

    log_debug!("Reseting redis to clean state...");
    {
        // Remove everything
        let _ = redis_conn.graph_delete(CARGO_GRAPH_NAME);
        redis::cmd("flushdb").execute(&mut redis_conn);

        // Ensure graph is created and ready
        redis_conn.graph_query(CARGO_GRAPH_NAME, "create (:Example {name: 'tmp'})")?;
        redis_conn.graph_query(
            CARGO_GRAPH_NAME,
            "match (e: Example {name: 'tmp'}) delete e",
        )?;
    }

    log_debug!("Start fetching data from postgres...");
    let db_results = {
        let users_promise = get_users_from_db_async(&postgres_pool);
        let crates_promise = get_crates_from_db_async(&postgres_pool);
        let crate_versions_promise = get_crate_versions_from_db_async(&postgres_pool);
        let dependenies_promise = get_raw_dependencies_from_db_async(&postgres_pool);

        tokio::join!(
            users_promise,
            crates_promise,
            crate_versions_promise,
            dependenies_promise
        )
    };
    log_debug!("Done fetching data from postgres.");

    let users = db_results.0?;
    let crates = db_results.1?;
    let crate_versions = db_results.2?;
    let dependencies = db_results.3?;

    log_debug!("Resolving connected packages and transforming into edge structs...");
    let dependency_edges = connect_db_dependencies(&crate_versions, &dependencies);
    log_debug!("Done connecting packages versions and transforming into edge structs.");

    // Order of queries matters!
    log_debug!("Generating redisgraph queries from data...");

    let mut queries = Vec::new();

    // Nodes
    let mut users_redis_graph_query = gen_users_redis_graph_node_query(&users)?;
    let mut crates_redis_graph_query = gen_crates_redis_graph_node_query(&crates)?;
    let mut crate_versions_redis_graph_query =
        gen_crate_versions_redis_graph_node_query(&crate_versions)?;

    queries.append(&mut users_redis_graph_query);
    queries.append(&mut crates_redis_graph_query);
    queries.append(&mut crate_versions_redis_graph_query);

    // Relations
    let mut published_by_graph_link_query =
        gen_published_by_redis_graph_link_query(&crate_versions)?;
    let mut versions_link_to_crates_graph_link_query =
        gen_version_redis_graph_link_query(&crate_versions)?;
    let mut first_versions_graph_link_query =
        gen_first_or_latest_version_redis_graph_link_query(&crate_versions, false)?;
    let mut latests_versions_graph_link_query =
        gen_first_or_latest_version_redis_graph_link_query(&crate_versions, true)?;
    let mut dependency_graph_link_query = gen_dependency_redis_graph_link_query(&dependency_edges)?;

    queries.append(&mut published_by_graph_link_query);
    queries.append(&mut versions_link_to_crates_graph_link_query);
    queries.append(&mut first_versions_graph_link_query);
    queries.append(&mut latests_versions_graph_link_query);
    queries.append(&mut dependency_graph_link_query);

    log_debug!("Done generating redisgraph queries from data.");

    log_debug!("Executing redisgraph queries...");
    for query in queries {
        let answ = redis_conn.graph_query(CARGO_GRAPH_NAME, query)?;

        log_debug!("{:?}", answ);
    }
    log_debug!("Done executing redisgraph queries.");

    Ok(())
}
