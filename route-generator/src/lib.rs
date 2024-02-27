use log::warn;
use osmpbfreader::NodeId;

use mgr_map_extractor::graph::{Edge, Graph};
use mgr_weather::image_wrapper::PixelColor;

pub mod providers;

#[derive(Debug, Clone)]
pub struct RouteDetails {
    pub starting_node: NodeId,
    pub ending_node: NodeId,
}

#[derive(Debug, Clone)]
pub struct Route(pub Vec<Edge>);

#[derive(Debug)]
pub struct PixRoute(pub Vec<GridPosition>);

#[derive(Debug, Clone, Copy)]
pub struct WxRouteDetails {
    pub starting_position: GridPosition,
    pub ending_position: GridPosition,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GridPosition {
    pub x: f32,
    pub y: f32,
}

impl Route {
    pub fn edges(&self) -> &Vec<Edge> {
        &self.0
    }
}

pub trait RouteGenerator {
    fn generate_route(graph: &Graph, generation_strategy: RouteGeneratorStrategy, route_details: &RouteDetails, mode: GenerationMode) -> Route;
    fn generate_route_rn(_generation_strategy: RouteGeneratorStrategy, _route_details: &WxRouteDetails) -> PixRoute {
        warn!("Real number mode not supported for this route generator");
        PixRoute(vec![])
    }

    fn real_mode_supported() -> bool {
        false
    }

    fn generate_route_real_num(_route_details: &WxRouteDetails, _grid: &Vec<PixelColor>) -> PixRoute {
        warn!("Real mode not supported for this route generator");
        PixRoute(vec![])
    }
}

#[repr(C)]
pub enum GenerationMode{
    WX,
    Graph
}

#[repr(C)]
pub enum RouteGeneratorStrategy {
    Empty,
    Greedy,
    ACO,
    PSO,
}

pub fn generate_route_rn(generation_strategy: RouteGeneratorStrategy, route_details: &WxRouteDetails, grid: &Vec<PixelColor>) -> PixRoute {
    match generation_strategy {
        RouteGeneratorStrategy::Greedy => providers::greedy::RouteGeneratorGreedy::generate_route_real_num(route_details, grid),
        RouteGeneratorStrategy::ACO => providers::aco::RouteGeneratorACO::generate_route_real_num(route_details, grid),
        RouteGeneratorStrategy::PSO => providers::pso::RouteGeneratorPSO::generate_route_real_num(route_details, grid),
        RouteGeneratorStrategy::Empty => PixRoute(vec![]),
    }
}

pub fn generate_route(graph: &Graph, generation_strategy: RouteGeneratorStrategy, route_details: &RouteDetails) -> Route {
    match generation_strategy {
        RouteGeneratorStrategy::Greedy => providers::greedy::RouteGeneratorGreedy::generate_route(graph, generation_strategy, route_details, GenerationMode::Graph),
        RouteGeneratorStrategy::ACO => providers::aco::RouteGeneratorACO::generate_route(graph, generation_strategy, route_details, GenerationMode::Graph),
        RouteGeneratorStrategy::PSO => providers::pso::RouteGeneratorPSO::generate_route(graph, generation_strategy, route_details, GenerationMode::Graph),
        RouteGeneratorStrategy::Empty => Route(vec![]),
    }
}
