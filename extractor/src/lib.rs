use std::collections::HashSet;

use log::info;
pub use osmpbfreader::NodeId;
use osmpbfreader::OsmObj;

pub(crate) use loader::{DataFetcher, Loader, OutputFormat};
pub use parser::ApprovedHighwayType;

use crate::graph::{Edge, Graph, Node};
use crate::parser::filter_way_data;

mod loader;
mod parser;
pub mod graph;

pub struct CoordinateStats {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}


pub fn produce_connection_graph() -> (Graph, CoordinateStats) {
    info!("Starting extractor");
    info!("Loading map data");
    let loader = Loader::new(DataFetcher::File, OutputFormat::Json, false);
    let mut osm_reader = loader.load();

    let mut osm_nodes: HashSet<Node> = HashSet::with_capacity(100_000);

    // let mut osm_ways: HashSet<MapWay> = HashSet::new();

    let mut graph = Graph::new();

    let mut first_node_iter = true;

    let mut min_lat = 0.0;
    let mut max_lat = 0.0;

    let mut min_lon = 0.0;
    let mut max_lon = 0.0;

    let mut update_min_max_stats = |node: &Node| {
        if first_node_iter {
            min_lat = node.lat();
            max_lat = node.lat();

            min_lon = node.lon();
            max_lon = node.lon();

            first_node_iter = false;
        } else {
            if node.lat() < min_lat {
                min_lat = node.lat();
            }
            if node.lat() > max_lat {
                max_lat = node.lat();
            }

            if node.lon() < min_lon {
                min_lon = node.lon();
            }
            if node.lon() > max_lon {
                max_lon = node.lon();
            }
        }
    };

    for obj in osm_reader.iter().map(Result::unwrap) {
        match obj {
            OsmObj::Node(node) => {
                osm_nodes.insert(Node::new(node.id, node.lat(), node.lon()));
            }
            OsmObj::Way(way) => {
                //Generate edges
                let way_nodes_count = way.nodes.len();
                if let Some((way, road_type)) = filter_way_data(&way) {
                    for way_node_idx in 0..(way_nodes_count - 1) {
                        let node_id_from = way.nodes[way_node_idx];
                        let node_id_to = way.nodes[way_node_idx + 1];

                        insert_many(&[node_id_from, node_id_to], graph.nodes_mut(), &osm_nodes);

                        let node_from = graph.nodes().get(&Node::id(node_id_from)).unwrap();
                        let node_to = graph.nodes().get(&Node::id(node_id_to)).unwrap();

                        update_min_max_stats(node_from);

                        // println!("Node from: {:?}", node_from);

                        let edge_length = Edge::length(node_from, node_to);

                        //Each edge is bidirectional
                        graph.add_edge(Edge::new(node_id_from, node_id_to, edge_length, road_type));
                        graph.add_edge(Edge::new(node_id_to, node_id_from, edge_length, road_type));

                        graph.add_edge_connection(node_id_from, node_id_to);
                        graph.add_edge_connection(node_id_to, node_id_from);
                    }
                }
            }
            _ => {}
        }
    }
    let graph_stats = graph.stats();
    info!("{} nodes {} edges in a graph", graph_stats.0, graph_stats.1);

    info!("Finished generating graph");

    (
        graph,
        CoordinateStats {
            min_lat,
            max_lat,
            min_lon,
            max_lon,
        }
    )
}

fn insert_many(node_id: &[NodeId], set: &mut HashSet<Node>, osm_nodes: &HashSet<Node>) {
    node_id.iter().for_each(|node_id| {
        let key = &Node::id(*node_id);
        set.insert(*osm_nodes.get(key).unwrap());
    });
}


#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_insert_if_not_exists_when_not_exists() {
        let mut set = HashSet::new();
        let mut osm_nodes = HashSet::new();
        let node = Node::new(NodeId(1), 0.0, 0.0);
        osm_nodes.insert(node);

        insert_many(&[NodeId(1)], &mut set, &osm_nodes);

        assert_eq!(set.len(), 1);
        assert_eq!(osm_nodes.len(), 1);
        assert_eq!(set.get(&Node::id(NodeId(1))).unwrap(), &node);
    }

    #[test]
    fn test_insert_if_not_exists_when_exists() {
        let mut set = HashSet::new();
        let mut osm_nodes = HashSet::new();
        let node = Node::new(NodeId(1), 0.0, 0.0);
        osm_nodes.insert(node);

        insert_many(&[NodeId(1)], &mut set, &osm_nodes);

        assert_eq!(set.len(), 1);
        assert_eq!(osm_nodes.len(), 1);
        assert_eq!(set.get(&Node::id(NodeId(1))).unwrap(), &node);
    }
}
