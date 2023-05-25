use std::collections::{HashMap, VecDeque};

use redis::Connection;
use redis_graph::{GraphCommands, GraphResult, WithProperties};

fn main() -> anyhow::Result<()> {
    let redis_client = redis::Client::open("redis://127.0.0.1:7500/")?;
    let mut redis_conn = redis_client.get_connection()?;

    // let mut to_fetch: VecDeque<> = VecDeque::new();

    /*
       for i in 100_000..110_000 {
           let answ = redis_conn.graph_ro_query(
               "cargo_graph",
               format!("MATCH (cv: CargoCrateVersion {{id: {}}})-[d:DEPENDS_ON]->(cv2:CargoCrateVersion) RETURN d, cv", i),
           )?;
       }
    */

    let initial_node_req = redis_conn.graph_ro_query(
        "cargo_graph",
        "match (cv: CargoCrateVersion {id: 468088}) return cv",
    )?;
    let initial_root_version_node =
        CargoCrateVersionNode::parse(initial_node_req.data.first().unwrap(), "cv")?;

    dbg!(&initial_root_version_node);
    let a = traverse_node(&mut redis_conn, initial_root_version_node);
    dbg!(a);

    Ok(())
}

fn traverse_node(
    redis_conn: &mut Connection,
    root_node: CargoCrateVersionNode,
) -> anyhow::Result<()> {
    let dependencies_query = redis_conn.graph_ro_query(
        "cargo_graph",
        format!(
            "match (:CargoCrateVersion {{id: {}}})-[d:DEPENDS_ON]->(cv:CargoCrateVersion) return d, cv",
            root_node.id
        ),
    )?;

    let nodes = CargoCrateVersionNode::parse_bulk(&dependencies_query.data, "cv")?;
    let edges = CargoDependsOnEdge::parse_bulk(&dependencies_query.data, "d")?;

    for edge in edges {
        let dest_node = nodes
            .iter()
            .find(|s| s.node_id == edge.dest_node_id)
            .unwrap();
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct CargoCrateVersionNode {
    node_id: u64,

    id: i32,
    num: String,
    features: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
struct CargoDependsOnEdge {
    dest_node_id: u64,

    optional: bool,
    with_features: Option<Vec<String>>,
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
