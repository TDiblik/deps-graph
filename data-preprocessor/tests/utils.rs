use data_preprocessor::utils::connect_db_dependencies;
use std::{assert_eq, collections::HashMap};

use data_preprocessor::models::{
    CargoCrateVersionDBResponse, CargoDependenciesDBResponse, CargoDependencyKind,
    CargoDependencyRGEdgeBuilder,
};

// Important for versions: id, create_id, num
// Important for depenedencies: required_semver, to_crate_id
macro_rules! quick_crate_version {
    ($id:expr, $crate_id:expr, $num:expr) => {
        CargoCrateVersionDBResponse {
            id: $id,
            crate_id: $crate_id,
            num: $num.into(),
            features: sqlx::types::Json(HashMap::new()),
            published_by: None,
            crate_name: "".into(),
        }
    };
}

macro_rules! quick_dependency {
    ($to:expr, $req:expr) => {
        CargoDependenciesDBResponse {
            from_version_id: 1,
            to_crate_id: $to,
            required_semver: $req.into(),
            optional: false,
            default_features: false,
            features: Vec::new(),
            target: None,
            kind: CargoDependencyKind::Normal,
        }
    };
}

macro_rules! quick_edge {
    ($to:expr, $req:expr) => {
        CargoDependencyRGEdgeBuilder {
            from_version_id: 1,
            to_version_id: $to,
            required_semver: $req.into(),
            optional: false,
            default_features: false,
            with_features: Vec::new(),
            target: None,
            kind: CargoDependencyKind::Normal,
        }
    };
}

#[test]
fn basic_db_dependency_connection() {
    let db_crate_versions = vec![
        quick_crate_version![1, 1, "1.0.0"],
        quick_crate_version![2, 1, "1.1.0"],
        quick_crate_version![3, 1, "1.2.0"],
    ];

    let db_dependencies_1 = vec![quick_dependency![1, "^1.0.0"]];
    let expected_output_1: Vec<CargoDependencyRGEdgeBuilder> = vec![quick_edge![3, "^1.0.0"]];
    let output_1 = connect_db_dependencies(&db_crate_versions, &db_dependencies_1);
    assert_eq!(output_1, expected_output_1);

    let db_dependencies_2 = vec![quick_dependency![1, "<=1.0.0"]];
    let expected_output_2: Vec<CargoDependencyRGEdgeBuilder> = vec![quick_edge![1, "<=1.0.0"]];
    let output_2 = connect_db_dependencies(&db_crate_versions, &db_dependencies_2);
    assert_eq!(output_2, expected_output_2);
}

#[test]
fn multiple_versions_and_dependencies() {
    let db_crate_versions = vec![
        quick_crate_version![1, 1, "1.0.0"],
        quick_crate_version![2, 1, "1.1.0"],
        quick_crate_version![3, 1, "1.2.0"],
        quick_crate_version![4, 2, "2.0.0"],
        quick_crate_version![5, 2, "2.1.0"],
    ];

    let db_dependencies = vec![
        quick_dependency![1, "^1.0.0"],
        quick_dependency![1, ">=1.0.0"],
        quick_dependency![1, "~1.0.0"],
        quick_dependency![2, "^2.0.0"],
        quick_dependency![2, "~2.0.0"],
    ];

    let expected_output: Vec<CargoDependencyRGEdgeBuilder> = vec![
        quick_edge![3, "^1.0.0"],
        quick_edge![3, ">=1.0.0"],
        quick_edge![1, "~1.0.0"],
        quick_edge![5, "^2.0.0"],
        quick_edge![4, "~2.0.0"],
    ];

    let output = connect_db_dependencies(&db_crate_versions, &db_dependencies);
    assert_eq!(output, expected_output);
}
