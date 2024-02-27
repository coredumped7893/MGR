use log::warn;
use osmpbfreader::NodeId;

use mgr_map_extractor::graph::{Edge, Graph, Node};

use crate::{GenerationMode, Route, RouteDetails, RouteGenerator, RouteGeneratorStrategy};

pub struct RouteGeneratorGreedy;

#[derive(PartialEq, Eq, Debug)]
enum MetricType {
    EdgeLength,
    //Select node based on the shortest hop distance (distance to the next node)
    GoalDistance, //Selects node that is closest to the goal
}

const ALG_METRIC_TYPE: MetricType = MetricType::GoalDistance;

impl RouteGenerator for RouteGeneratorGreedy {
    /// Generate route using greedy algorithm. Starting point and ending point are provided in route_details
    fn generate_route(graph: &Graph, _generation_strategy: RouteGeneratorStrategy, route_details: &RouteDetails, _mode: GenerationMode) -> Route {
        let mut route = Vec::new();
        let mut current_node = route_details.starting_node;
        let ending_node = graph.nodes().get(&Node::id(route_details.ending_node)).unwrap();

        while current_node != route_details.ending_node {
            match select_next_best(&route, &mut current_node, graph, ending_node) {
                None => {
                    warn!("Cannot select next best node");
                    break;
                }
                Some(n) => {
                    route.push(Edge::create(graph, current_node, n));
                    current_node = n;
                }
            }
        }

        Route(route)
    }
}

//Select next node that is closest to the current node
fn select_next_best(route: &[Edge], current_node: &mut NodeId, graph: &Graph, ending_node: &Node) -> Option<NodeId> {
    let graph_edge_connections = graph.edge_connections();

    let mut edge_candidates: Vec<Edge> = vec![];

    //Iterate over all edges for current node
    for &edge in graph_edge_connections.get(current_node).unwrap() {
        let new_edge = Edge::create(graph, *current_node, edge);
        if route.contains(&new_edge) {
            //Edge is already in the route, skip it
            continue;
        }
        edge_candidates.push(new_edge);
    }

    if edge_candidates.is_empty() {
        return None;
    }

    // debug!("Metric type: {:?}", ALG_METRIC_TYPE);
    if ALG_METRIC_TYPE == MetricType::EdgeLength {
        //Sort edges by the length in place
        edge_candidates.sort_by(|a, b| a.length.partial_cmp(&b.length).unwrap());
    } else {
        let distance_to_destination = |edge: &Edge| {
            Edge::length(
                graph.nodes().get(&Node::id(edge.to)).unwrap(),
                ending_node,
            )
        };

        //Sort edges by the distance to the ending node
        edge_candidates.sort_by(|a, b| {
            let a_dist = distance_to_destination(a);
            let b_dist = distance_to_destination(b);
            a_dist.partial_cmp(&b_dist).unwrap()
        })
    }
    Some(edge_candidates[0].to)
}