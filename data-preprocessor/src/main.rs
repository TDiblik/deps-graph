mod constants;
mod models;
mod utils;

use std::dbg;

use anyhow::Result;
use constants::CARGO_GRAPH_NAME;
use redis_graph::*;
use sqlx::postgres::PgPoolOptions;
use utils::{get_crate_versions_from_db_async, get_crates_from_db_async, get_users_from_db_async};

use crate::utils::get_raw_dependencies_from_db_async;

#[tokio::main]
async fn main() -> Result<()> {
    let postgres_pool = PgPoolOptions::new()
        .max_connections(4)
        .connect("postgresql://dumpuser:c4rg0DUmP@localhost:7501/dumpdb")
        .await?;

    let redis_client = redis::Client::open("redis://127.0.0.1:7500/")?;
    let mut redis_conn = redis_client.get_connection()?;

    // Reset redis to original state
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

    let users = db_results.0?;
    let crates = db_results.1?;
    let create_versions = db_results.2?;
    let dependencies = db_results.3?;

    dbg!(users);
    dbg!(crates);
    dbg!(create_versions);
    dbg!(dependencies);

    Ok(())
}
