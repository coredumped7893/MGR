use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use osmpbfreader::NodeId;

use crate::parser::ApprovedHighwayType;

#[derive(Debug, Clone, Default)]
pub struct Graph {
    nodes: HashSet<Node>,
    edges: Vec<Edge>,
    edge_by_node_id: HashMap<(NodeId, NodeId), Edge>,
    edge_connections: HashMap<NodeId, Vec<NodeId>>,// Map source node to list of target nodes
}

#[derive(Debug, Clone, Copy)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    pub length: f64,
    pub highway_type: ApprovedHighwayType,
}

// Edge direction does not matter
impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        (self.from == other.from && self.to == other.to) || (self.from == other.to && self.to == other.from)
    }
}

impl Eq for Edge {}

const DEFAULT_GRAPH_NODES_CAPACITY: usize = 5000;

//Each point can have 2 edges (incoming and out-coming)
const DEFAULT_GRAPH_EDGES_CAPACITY: usize = DEFAULT_GRAPH_NODES_CAPACITY * 2;

#[derive(Debug, Clone, Copy)]
pub struct Node {
    id: NodeId,
    lat: f64,
    lon: f64,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            nodes: HashSet::with_capacity(DEFAULT_GRAPH_NODES_CAPACITY),
            edges: Vec::with_capacity(DEFAULT_GRAPH_EDGES_CAPACITY),
            edge_by_node_id: Default::default(),
            edge_connections: Default::default(),
        }
    }


    pub fn stats(&self) -> (usize, usize) {
        (self.nodes.len(), self.edges.len())
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node);
    }

    pub fn add_edge_connection(&mut self, from: NodeId, to: NodeId) {
        let connections = self.edge_connections.entry(from).or_default();
        connections.push(to);
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edge_by_node_id.insert((edge.from, edge.to), edge.clone());
        self.edges.push(edge);
    }

    pub fn nodes(&self) -> &HashSet<Node> {
        &self.nodes
    }

    pub fn nodes_mut(&mut self) -> &mut HashSet<Node> {
        &mut self.nodes
    }

    pub fn edges(&self) -> &Vec<Edge> {
        &self.edges
    }
    pub fn edge_connections(&self) -> &HashMap<NodeId, Vec<NodeId>> {
        &self.edge_connections
    }
    pub fn edge_by_node_id(&self) -> &HashMap<(NodeId, NodeId), Edge> {
        &self.edge_by_node_id
    }
}


impl Node {
    pub fn new(id: NodeId, lat: f64, lon: f64) -> Self {
        Node {
            id,
            lat,
            lon,
        }
    }

    pub fn id(id: NodeId) -> Self {
        Node {
            id,
            lat: 0.0,
            lon: 0.0,
        }
    }

    pub fn get_id(&self) -> NodeId {
        self.id
    }

    pub fn get_coordinates(&self) -> (f64, f64) {
        (self.lat, self.lon)
    }

    pub fn lat(&self) -> f64 {
        self.lat
    }
    pub fn lon(&self) -> f64 {
        self.lon
    }
}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Node {}

impl Edge {
    pub fn new(from: NodeId, to: NodeId, length: f64, highway_type: ApprovedHighwayType) -> Self {
        Edge {
            from,
            to,
            length,
            highway_type,
        }
    }

    //Create edge from node to node and auto calculate length from provided graph context
    pub fn create(graph: &Graph, from: NodeId, to: NodeId) -> Self {
        let from_node_opt = graph.nodes().get(&Node::id(from));
        let to_node_opt = graph.nodes().get(&Node::id(to));

        if let (Some(from_node), Some(to_node)) = (from_node_opt, to_node_opt) {
            let length = Edge::length(from_node, to_node);
            Edge::new(from, to, length, ApprovedHighwayType::NA)
        } else {
            //@TODO handle without panic
            panic!("Cannot create edge from {:?} to {:?} because one of the nodes does not exist", from, to);
        }
    }

    //Should be given in meters
    pub fn length(from: &Node, to: &Node) -> f64 {
        let (lat_from, lon_from) = from.get_coordinates();
        let (lat_to, lon_to) = to.get_coordinates();

        let lat_diff = ((lat_from - lat_to).abs()).powf(2.0);
        let lon_diff = ((lon_from - lon_to).abs()).powf(2.0);
        ((lat_diff + lon_diff).sqrt()) * 111.1 * 1000.0
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_map_node_hash_insert() {
        let node1 = Node {
            id: NodeId(1),
            lat: 0.0,
            lon: 0.0,
        };

        let node2 = Node {
            id: NodeId(1),
            lat: 5.1,
            lon: 23.0,
        };

        let node3 = Node {
            id: NodeId(2),
            lat: 0.0,
            lon: 0.0,
        };

        let mut set = HashSet::new();
        set.insert(node1);
        set.insert(node2);
        set.insert(node3);

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_map_node_eq() {
        let node1 = Node {
            id: NodeId(1),
            lat: 7.0,
            lon: 3.0,
        };

        let node2 = Node {
            id: NodeId(4),
            lat: 5.1,
            lon: 23.0,
        };

        let node3 = Node {
            id: NodeId(4),
            lat: 7.8,
            lon: 111.56434,
        };

        let mut set = HashSet::new();
        set.insert(node1);
        set.insert(node2);

        assert_eq!(set.len(), 2);

        assert!(set.contains(&Node {
            id: NodeId(1),
            lat: 0.0,
            lon: 0.0,
        }));

        assert!(set.contains(&Node {
            id: NodeId(4),
            lat: 5.1,
            lon: 23.0,
        }));

        assert_ne!(node1, node2);

        assert_eq!(node2, node3);
    }

    #[test]
    fn test_edge_length() {
        let p1 = Node::new(NodeId(1), 0.0, 0.0);
        let p2 = Node::new(NodeId(1), 3.0, 4.0);

        let len = Edge::length(&p1, &p2);

        assert_eq!(len, 5.0)
    }

    #[test]
    fn test_edge_length_single_point() {
        let p1 = Node::new(NodeId(1), 0.0, 0.0);
        let p2 = Node::new(NodeId(1), 0.0, 0.0);

        let len = Edge::length(&p1, &p2);

        assert_eq!(len, 0.0)
    }

    #[test]
    fn test_node_connections() {
        let mut g = Graph::new();

        g.add_edge_connection(NodeId(1), NodeId(2));
        g.add_edge_connection(NodeId(1), NodeId(3));
        g.add_edge_connection(NodeId(3), NodeId(4));

        assert_eq!(g.edge_connections.len(), 2); // We have two distinct keys
        assert_eq!(g.edge_connections.get(&NodeId(2)), None);
        assert_eq!(g.edge_connections.get(&NodeId(1)), Some(&vec![NodeId(2), NodeId(3)]));
    }

    #[test]
    fn test_node_connections_circular() {
        let mut g = Graph::new();

        g.add_edge_connection(NodeId(1), NodeId(2));
        g.add_edge_connection(NodeId(1), NodeId(3));
        g.add_edge_connection(NodeId(3), NodeId(4));
        g.add_edge_connection(NodeId(3), NodeId(1));
        g.add_edge_connection(NodeId(1), NodeId(1));

        assert_eq!(g.edge_connections.len(), 2); // We have two distinct keys
        assert_eq!(g.edge_connections.get(&NodeId(2)), None);
        assert_eq!(g.edge_connections.get(&NodeId(1)), Some(&vec![NodeId(2), NodeId(3), NodeId(1)]));
    }

    #[test]
    fn edge_equality_same_node_ids() {
        let e1 = Edge::new(NodeId(1), NodeId(2), 0.0, ApprovedHighwayType::Motorway);
        let e2 = Edge::new(NodeId(1), NodeId(2), 0.0, ApprovedHighwayType::Motorway);

        assert_eq!(e1, e2);
    }

    #[test]
    fn edge_equality_direction_doest_not_matter() {
        let e1 = Edge::new(NodeId(1), NodeId(2), 0.0, ApprovedHighwayType::Motorway);
        let e2 = Edge::new(NodeId(2), NodeId(1), 0.0, ApprovedHighwayType::Motorway);

        assert_eq!(e1, e2);
    }

    #[test]
    fn edge_equality_not_equal() {
        let e1 = Edge::new(NodeId(1), NodeId(2), 0.0, ApprovedHighwayType::Motorway);
        let e2 = Edge::new(NodeId(3), NodeId(5), 0.0, ApprovedHighwayType::Motorway);

        assert_ne!(e1, e2);
    }

    #[test]
    fn edge_equality_type_does_not_matter() {
        let e1 = Edge::new(NodeId(1), NodeId(2), 0.0, ApprovedHighwayType::Motorway);
        let e2 = Edge::new(NodeId(1), NodeId(2), 0.0, ApprovedHighwayType::Secondary);

        assert_eq!(e1, e2);
    }

    #[test]
    fn edge_equality_length_does_not_matter() {
        let e1 = Edge::new(NodeId(1), NodeId(2), 0.0, ApprovedHighwayType::Motorway);
        let e2 = Edge::new(NodeId(1), NodeId(2), 12.34, ApprovedHighwayType::Motorway);

        assert_eq!(e1, e2);
    }
}
