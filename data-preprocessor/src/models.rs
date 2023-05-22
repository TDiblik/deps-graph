use std::collections::HashMap;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CargoUserDBResponse {
    pub id: i32,
    pub gh_username: String,
    pub gh_avatar: Option<String>,
    pub preferred_name: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CargoCrateDBResponse {
    pub id: i32,
    pub name: String,
    // TODO: Add description, repository, documentation and homepage (make sure to update sql and redis commands)
}

#[derive(Debug, sqlx::FromRow)]
pub struct CargoCrateVersionDBResponse {
    pub id: i32,
    pub crate_id: i32,
    pub num: String,
    pub features: sqlx::types::Json<HashMap<String, Vec<String>>>,
    // TODO: Add description, repository, documentation and homepage (make sure to update sql and redis commands)
    pub published_by: Option<i32>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CargoDependenciesDBResponse {
    pub from_version_id: i32,
    pub to_crate_id: i32,
    pub required_semver: String,
    pub optional: bool,
    pub default_features: bool,
    pub target: Option<String>,
    pub kind: CargoDependencyKind,
}

#[derive(Debug, PartialEq)]
pub struct CargoDependencyRGEdgeBuilder {
    pub from_version_id: i32,
    pub to_version_id: i32,
    pub required_semver: String,
    pub optional: bool,
    pub default_features: bool,
    pub target: Option<String>,
    pub kind: CargoDependencyKind,
}

#[derive(Debug, Clone, PartialEq, sqlx::Type)]
#[repr(i32)]
pub enum CargoDependencyKind {
    Normal = 0,
    Build = 1,
    Dev = 2,
}
