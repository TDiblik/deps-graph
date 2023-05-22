use std::{collections::HashMap, future::Future};

use semver::{Version, VersionReq};
use serde_json::json;
use sqlx::{Pool, Postgres};

use crate::{
    constants::REDIS_INSERTION_CHUNK_SIZE,
    models::{
        CargoCrateDBResponse, CargoCrateVersionDBResponse, CargoDependenciesDBResponse,
        CargoDependencyRGEdgeBuilder, CargoUserDBResponse,
    },
};

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        println!($($arg)*)
    };
}

pub fn get_users_from_db_async(
    pool: &Pool<Postgres>,
) -> impl Future<Output = Result<Vec<CargoUserDBResponse>, sqlx::Error>> + '_ {
    sqlx::query_as::<_, CargoUserDBResponse>(
        r#"
            select id, gh_login as "gh_username", gh_avatar, name as "preferred_name" from users;
        "#,
    )
    .fetch_all(pool)
}

pub fn get_crates_from_db_async(
    pool: &Pool<Postgres>,
) -> impl Future<Output = Result<Vec<CargoCrateDBResponse>, sqlx::Error>> + '_ {
    sqlx::query_as::<_, CargoCrateDBResponse>(
        r#"
            select id, name from crates;
        "#,
    )
    .fetch_all(pool)
}

pub fn get_crate_versions_from_db_async(
    pool: &Pool<Postgres>,
) -> impl Future<Output = Result<Vec<CargoCrateVersionDBResponse>, sqlx::Error>> + '_ {
    sqlx::query_as::<_, CargoCrateVersionDBResponse>(
        r#"
            select id, crate_id, num, features, published_by from versions order by id;
        "#,
    )
    .fetch_all(pool)
}

pub fn get_raw_dependencies_from_db_async(
    pool: &Pool<Postgres>,
) -> impl Future<Output = Result<Vec<CargoDependenciesDBResponse>, sqlx::Error>> + '_ {
    sqlx::query_as::<_, CargoDependenciesDBResponse>(
        r#"
            select version_id "from_version_id", crate_id "to_crate_id", req "required_semver", optional, default_features, target, kind from dependencies;
        "#,
    )
    .fetch_all(pool)
}

pub fn gen_users_redis_graph_node_query(users: &[CargoUserDBResponse]) -> anyhow::Result<Vec<String>> {
    gen_redis_creation_command(
        users
            .iter()
            .map(|s| {
                format!(
                    "[{},{},{},{}]",
                    json!(s.id),
                    json!(s.gh_username),
                    json!(s.gh_avatar),
                    json!(s.preferred_name)
                )
            })
            .collect(),
        Some(
            "create (:CargoUser {id: map[0], gh_username: map[1], gh_avatar: map[2], preferred_name: map[3]})"
        )
    )
}

pub fn gen_crates_redis_graph_node_query(
    crates: &[CargoCrateDBResponse],
) -> anyhow::Result<Vec<String>> {
    gen_redis_creation_command(
        crates
            .iter()
            .map(|s| format!("[{},{}]", json!(s.id), json!(s.name),))
            .collect(),
        Some("create (:CargoCrate {id: map[0], name: map[1]})"),
    )
}

pub fn gen_crate_versions_redis_graph_node_query(
    crate_versions: &[CargoCrateVersionDBResponse],
) -> anyhow::Result<Vec<String>> {
    gen_redis_creation_command(
        crate_versions
            .iter()
            .map(|s| {
                format!(
                    "[{},{},{}]",
                    json!(s.id),
                    json!(s.num),
                    json!(json!(s.features).to_string()), // TODO: Dump hack, fix
                )
            })
            .collect(),
        Some("create (:CargoCrateVersion {id: map[0], num: map[1], features: map[2]})"),
    )
}

pub fn gen_published_by_redis_graph_link_query(
    crate_versions: &[CargoCrateVersionDBResponse],
) -> anyhow::Result<Vec<String>> {
    gen_redis_creation_command(
        crate_versions.iter().filter(|s| s.published_by.is_some()).map(|s| {
            format!(
                "[{}, {}]",
                json!(s.published_by.unwrap()), 
                json!(s.id)
            )
        }).collect(), 
        Some("MATCH (cu:CargoUser {id: map[0]}), (cv:CargoCrateVersion {id: map[1]}) CREATE (cu)-[:PUBLISHED]->(cv)")
    )
}

pub fn gen_version_redis_graph_link_query(
    crate_versions: &[CargoCrateVersionDBResponse],
) -> anyhow::Result<Vec<String>> {
    gen_redis_creation_command(
        crate_versions.iter().map(|s| {
            format!(
                "[{}, {}]",
                json!(s.crate_id), 
                json!(s.id)
            )
        }).collect(), 
        Some("MATCH (cc:CargoCrate {id: map[0]}), (cv:CargoCrateVersion {id: map[1]}) CREATE (cc)-[:VERSION]->(cv)")
    )
}

fn gen_redis_creation_command(
    mapped_data: Vec<String>,
    query_to_append_to_end_of_each_chunk: Option<&str>,
) -> anyhow::Result<Vec<String>> {
    let mut commands: Vec<String> = vec![];

    for data_row in mapped_data.chunks(REDIS_INSERTION_CHUNK_SIZE) {
        let mut query_builder = string_builder::Builder::default();
        query_builder.append("unwind [");
        query_builder.append(data_row.join(",").trim_end_matches(','));
        query_builder.append("] as map ");
        query_builder.append(query_to_append_to_end_of_each_chunk.unwrap_or(""));
        commands.push(query_builder.string()?.trim().to_string());
    }

    Ok(commands)
}

// Assumes db_create_versions are ordered by id and smaller id == smaller version
// (which should be true in theory, because it makes sense, but I haven't checked for it in the dataset)
// TODO: Above assumptions could be flawed, try ordering by semver in the version caching phase and see how much of a performance hit it will be
// TODO: Does not automatically "upgrade" to latest version, causing some packages not matching. fix it. https://doc.rust-lang.org/cargo/reference/resolver.html#pre-releases
#[derive(Debug)]
struct VersionCacher<'a> {
    parsed_version: Version,
    original_crate_version: &'a CargoCrateVersionDBResponse,
}
pub fn connect_db_dependencies(
    db_crate_versions: &Vec<CargoCrateVersionDBResponse>,
    db_dependencies: &Vec<CargoDependenciesDBResponse>,
) -> Vec<CargoDependencyRGEdgeBuilder> {
    // Cache crate versions
    let mut version_hashmap: HashMap<i32, Vec<VersionCacher>> = HashMap::new();
    for version in db_crate_versions {
        version_hashmap
            .entry(version.crate_id)
            .or_insert_with(Vec::new);
    }
    for version in db_crate_versions {
        if let Ok(parsed_version) = Version::parse(&version.num) {
            let current_versions = version_hashmap.get_mut(&version.crate_id).unwrap();
            current_versions.push(VersionCacher {
                parsed_version,
                original_crate_version: version,
            });
        }
    }

    // Connect best possible matches
    // Going from top to bottom (since most packages depend on latest versions)
    // and break when version matches requirements
    let mut dependency_edges = vec![];
    for dep in db_dependencies {
        let Ok(requirement) = VersionReq::parse(&dep.required_semver) else {
            continue;
        };

        let mut best_possible_pick: Option<&CargoCrateVersionDBResponse> = None;
        let all_possible_picks = version_hashmap.get(&dep.to_crate_id).unwrap();
        for possible_pick in all_possible_picks.iter().rev() {
            if requirement.matches(&possible_pick.parsed_version) {
                best_possible_pick = Some(possible_pick.original_crate_version);
                break;
            }
        }

        if let Some(pick) = best_possible_pick {
            dependency_edges.push(CargoDependencyRGEdgeBuilder {
                from_version_id: dep.from_version_id,
                to_version_id: pick.id,
                required_semver: dep.required_semver.clone(),
                optional: dep.optional,
                default_features: dep.default_features,
                target: dep.target.clone(),
                kind: dep.kind.clone(),
            });
        }
    }

    dependency_edges
}
