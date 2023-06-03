#![recursion_limit = "9999"] // If package has more than this number of features, something is wrong :DD

use std::collections::{HashMap, VecDeque};

use itertools::Itertools;
use redis::Connection;
use redis_graph::{GraphCommands, GraphResult, WithProperties};

fn main() -> anyhow::Result<()> {
    let redis_client = redis::Client::open("redis://127.0.0.1:7500/")?;
    let mut redis_conn = redis_client.get_connection()?;

    let initial_node_req = redis_conn.graph_ro_query(
        "cargo_graph",
        "match (cv: CargoCrateVersion {id: 781878}) return cv",
    )?;
    let initial_root_version_node =
        CargoCrateVersionNode::parse(initial_node_req.data.first().unwrap(), "cv")?;

    let mut connections_to_traverse = Vec::new();
    connections_to_traverse.push(GraphConnection {
        edge: CargoDependsOnEdge {
            src_node_id: 0,
            dest_node_id: initial_root_version_node.node_id,
            optional: false,
            with_features: vec![],
            default_features: true,
            kind: CargoDependencyKind::Normal,
        },
        node: initial_root_version_node,
    });

    let mut traversed_nodes: Vec<CargoCrateVersionNode> = vec![];
    let mut traversed_edges: Vec<CargoDependsOnEdge> = vec![];
    while let Some(connection_to_traverse) = connections_to_traverse.pop() {
        let traversed_connections = traverse_node(
            &mut redis_conn,
            connection_to_traverse.node,
            connection_to_traverse.edge.with_features,
            true,
            true,
            true,
        )?;

        for traversed_connection in traversed_connections.clone() {
            if let Some(already_traversed_node) = traversed_nodes
                .iter()
                .find(|s| s.node_id == traversed_connection.node.node_id)
            {
                let mut edge_to_add = traversed_connection.edge.clone();
                edge_to_add.dest_node_id = already_traversed_node.node_id;
                traversed_edges.push(edge_to_add);
            } else {
                traversed_nodes.push(traversed_connection.node.clone());
                traversed_edges.push(traversed_connection.edge.clone());
                connections_to_traverse.push(traversed_connection);
            };
        }

        dbg!(connections_to_traverse.len());
    }

    Ok(())
}

fn traverse_node(
    redis_conn: &mut Connection,

    root_node: CargoCrateVersionNode,
    wanted_features: Vec<String>,

    include_normal_dependencies: bool,
    include_build_dependencies: bool,
    include_dev_dependencies: bool,
) -> anyhow::Result<Vec<GraphConnection>> {
    let dependencies_query = {
        let mut query = format!(
            "match (:CargoCrateVersion {{id: {}}})-[d:DEPENDS_ON]->(cv:CargoCrateVersion) where ",
            root_node.id
        );
        if include_normal_dependencies {
            query.push_str("d.kind = 0 or ");
        }
        if include_build_dependencies {
            query.push_str("d.kind = 1 or ");
        }
        if include_dev_dependencies {
            query.push_str("d.kind = 2 or ");
        }
        query = query.trim_end_matches("or ").to_owned();
        query.push_str(" return d, cv");

        query
    };
    let dependencies_result = redis_conn.graph_ro_query("cargo_graph", dependencies_query)?;
    let nodes = CargoCrateVersionNode::parse_bulk(&dependencies_result.data, "cv")?;
    let edges = CargoDependsOnEdge::parse_bulk(&dependencies_result.data, "d")?;

    let mut connections: Vec<GraphConnection> = vec![];
    for edge in edges.iter() {
        connections.push(GraphConnection {
            edge: edge.clone(),
            node: nodes
                .iter()
                .find(|s| s.node_id == edge.dest_node_id)
                .unwrap()
                .clone(),
        });
    }

    let mut activated_connections: Vec<GraphConnection> = vec![];

    // All non-optional connection should be active right away.
    for connection in connections.iter() {
        if !connection.edge.optional {
            activated_connections.push(connection.clone());
        }
    }

    let mut traversed_features = vec![];
    for wanted_feature in wanted_features {
        traversed_features.extend(traverse_feature(wanted_feature, &root_node.features));
    }
    let filtered_features = traversed_features.iter().unique();

    // If needed (performance reasons), the following 2 loops could be put inside one loop,
    // however the functionality is much clearer when it's written this way.
    // TODO: It could be a good idea to rewrite it after I put some tests in place as guardrails.
    let dep_features = filtered_features.clone().filter(|s| !s.contains('/'));
    for feature in dep_features {
        let package_to_activate = if feature.contains(':') {
            feature.trim_start_matches("dep:")
        } else {
            feature
        };
        for connection in connections.iter() {
            if connection.node.crate_name == package_to_activate
                && !activated_connections
                    .iter()
                    .any(|s| s.node.node_id == connection.node.node_id)
            {
                activated_connections.push(connection.clone());
            }
        }
    }

    let activate_features = filtered_features
        .clone()
        .filter(|s| s.contains('/') && !s.contains("?/"));
    for feature in activate_features {
        let mut package_part_split = feature.split('/');
        let package_to_activate = package_part_split.next().unwrap();
        let feature_to_add = package_part_split.next().unwrap();

        for connection in connections.iter() {
            if connection.node.crate_name != package_to_activate {
                continue;
            }

            if !activated_connections
                .iter()
                .any(|s| s.node.node_id == connection.node.node_id)
            {
                activated_connections.push(connection.clone());
            }

            activated_connections
                .iter_mut()
                .find(|s| s.edge.dest_node_id == connection.edge.dest_node_id)
                .unwrap()
                .edge
                .with_features
                .push(feature_to_add.to_string());
        }
    }

    let possibly_activating_features = filtered_features.filter(|s| s.contains("?/"));
    for feature in possibly_activating_features {
        let mut package_part_split = feature.split("?/");
        let possibly_active_package = package_part_split.next().unwrap();
        let feature_to_add = package_part_split.next().unwrap();

        for connection in connections.iter() {
            if connection.node.crate_name != possibly_active_package {
                continue;
            }

            let Some(active_connection) = activated_connections.iter_mut().find(|s| s.node.node_id == connection.node.node_id) else {
                continue;
            };

            active_connection
                .edge
                .with_features
                .push(feature_to_add.to_string());
        }
    }

    Ok(activated_connections)
}

fn traverse_feature(
    wanted_feature: String,
    provided_features: &HashMap<String, Vec<String>>,
) -> Vec<String> {
    let Some(feature_array) = provided_features.get(&wanted_feature) else {
        return vec![wanted_feature];
    };

    let mut traversed_features = vec![];
    for feature in feature_array {
        // Catches "dep:example" ; "dep/example" ; "dep?/example"
        if feature.contains(':') || feature.contains('/') {
            traversed_features.push(feature.clone());
        } else {
            traversed_features.extend(traverse_feature(feature.clone(), provided_features));
        }
    }

    traversed_features
}

#[derive(Debug, Clone)]
struct GraphConnection {
    edge: CargoDependsOnEdge,
    node: CargoCrateVersionNode,
}

#[derive(Debug, Clone)]
struct CargoCrateVersionNode {
    node_id: u64,

    id: i32,
    num: String,
    features: HashMap<String, Vec<String>>,
    crate_name: String,
}

#[derive(Debug, Clone)]
struct CargoDependsOnEdge {
    src_node_id: u64,
    dest_node_id: u64,

    optional: bool,
    with_features: Vec<String>,
    default_features: bool,
    kind: CargoDependencyKind,
}

trait RedisGraphParser {
    fn parse(input: &GraphResult, data_variable_name: &str) -> anyhow::Result<Self>
    where
        Self: Sized;

    fn parse_bulk(input: &[GraphResult], data_variable_name: &str) -> anyhow::Result<Vec<Self>>
    where
        Self: Sized,
    {
        input
            .iter()
            .map(|s| RedisGraphParser::parse(s, data_variable_name))
            .collect()
    }

    fn parse_string_to_vec(val: Option<String>) -> Option<Vec<String>> {
        val.map(|s| {
            s.trim_start_matches('[')
                .trim_end_matches(']')
                .split(',')
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        })
    }
}

impl RedisGraphParser for CargoCrateVersionNode {
    fn parse(input: &GraphResult, data_variable_name: &str) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let node = input.get_node(data_variable_name).unwrap();

        Ok(CargoCrateVersionNode {
            node_id: node.id,
            id: node.get_property("id")?.unwrap(),
            num: node.get_property("num")?.unwrap(),
            features: serde_json::from_str(&node.get_property::<String>("features")?.unwrap())?,
            crate_name: node.get_property("crate_name")?.unwrap(),
        })
    }
}

impl RedisGraphParser for CargoDependsOnEdge {
    fn parse(input: &GraphResult, data_variable_name: &str) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let edge = input.get_relation(data_variable_name).unwrap();

        Ok(CargoDependsOnEdge {
            src_node_id: edge.src_node,
            dest_node_id: edge.dest_node,
            optional: edge.get_property::<String>("optional")?.unwrap().parse()?,
            with_features: edge.get_property("with_features")?.unwrap(),
            default_features: edge
                .get_property::<String>("default_features")?
                .unwrap()
                .parse()?,
            kind: edge.get_property::<i32>("kind")?.unwrap().into(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
#[repr(i32)]
pub enum CargoDependencyKind {
    Normal = 0,
    Build = 1,
    Dev = 2,
}

impl std::convert::From<i32> for CargoDependencyKind {
    fn from(value: i32) -> Self {
        match value {
            0 => CargoDependencyKind::Normal,
            1 => CargoDependencyKind::Build,
            2 => CargoDependencyKind::Dev,
            _ => panic!("Not implemented."),
        }
    }
}
