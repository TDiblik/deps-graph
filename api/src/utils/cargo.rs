use std::collections::HashMap;

use itertools::Itertools;
use redis::Connection;
use redis_graph::GraphCommands;

use crate::models::cargo_db_types::{
    CargoCrateVersionNode, CargoDependencyKind, CargoDependsOnEdge, RedisGraphParser,
};

pub fn traverse_tree(
    redis_conn: &mut Connection,

    root_node: CargoCrateVersionNode,
    root_features: Vec<String>,
    root_include_default_features: bool,

    include_normal_dependencies: bool,
    include_build_dependencies: bool,
    include_dev_dependencies: bool,
) -> anyhow::Result<(Vec<CargoCrateVersionNode>, Vec<CargoDependsOnEdge>)> {
    let mut wanted_features = root_features;
    if root_include_default_features {
        wanted_features.push("default".to_owned());
    }

    let mut connections_to_traverse = Vec::new();
    connections_to_traverse.push(GraphConnection {
        edge: CargoDependsOnEdge {
            src_node_id: u64::MAX, // u64::MAX == root
            dest_node_id: root_node.node_id,
            optional: false,
            with_features: vec![],
            kind: CargoDependencyKind::Normal,
        },
        node: root_node,
    });

    let mut traversed_nodes: Vec<CargoCrateVersionNode> = vec![];
    let mut traversed_edges: Vec<CargoDependsOnEdge> = vec![];
    while let Some(connection_to_traverse) = connections_to_traverse.pop() {
        let traversed_connections = traverse_node(
            redis_conn,
            connection_to_traverse.node,
            connection_to_traverse.edge.with_features,
            include_normal_dependencies,
            include_build_dependencies,
            include_dev_dependencies,
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

    Ok((traversed_nodes, traversed_edges))
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
