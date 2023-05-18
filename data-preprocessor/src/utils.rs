use std::future::Future;

use sqlx::{Pool, Postgres};

use crate::models::{
    CargoCrateRGNode, CargoCrateVersionRGNode, CargoDependenciesDBResponse, CargoUserRGNode,
};

pub fn get_users_from_db_async(
    pool: &Pool<Postgres>,
) -> impl Future<Output = Result<Vec<CargoUserRGNode>, sqlx::Error>> + '_ {
    sqlx::query_as::<_, CargoUserRGNode>(
        r#"
            select id, gh_login as "gh_username", gh_avatar, name as "preferred_name" from users;
        "#,
    )
    .fetch_all(pool)
}

pub fn get_crates_from_db_async(
    pool: &Pool<Postgres>,
) -> impl Future<Output = Result<Vec<CargoCrateRGNode>, sqlx::Error>> + '_ {
    sqlx::query_as::<_, CargoCrateRGNode>(
        r#"
            select id, name from crates;
        "#,
    )
    .fetch_all(pool)
}

pub fn get_crate_versions_from_db_async(
    pool: &Pool<Postgres>,
) -> impl Future<Output = Result<Vec<CargoCrateVersionRGNode>, sqlx::Error>> + '_ {
    sqlx::query_as::<_, CargoCrateVersionRGNode>(
        r#"
            select id, crate_id, num, features from versions;
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
