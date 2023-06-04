use redis_graph::{GraphResult, WithProperties};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Models
#[derive(Debug, Clone, Serialize)]
pub struct CargoCrateVersionNode {
    pub node_id: u64,

    pub id: i32,
    pub num: String,
    pub features: HashMap<String, Vec<String>>,
    pub crate_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CargoDependsOnEdge {
    pub src_node_id: u64,
    pub dest_node_id: u64,

    pub optional: bool,
    pub with_features: Vec<String>,
    pub kind: CargoDependencyKind,
}

// Helper types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
            _ => panic!("Not implemented / possible based on Cargo standard."),
        }
    }
}

// Custom parsing for each model
pub trait RedisGraphParser {
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
            kind: edge.get_property::<i32>("kind")?.unwrap().into(),
        })
    }
}
