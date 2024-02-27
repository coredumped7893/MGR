use mgr_map_extractor::graph::Graph;
use mgr_route_generator::{GenerationMode, providers, Route, RouteDetails, RouteGeneratorStrategy};
use mgr_route_generator::RouteGenerator;
use providers::aco::RouteGeneratorACO;
use providers::greedy::RouteGeneratorGreedy;
use providers::pso::RouteGeneratorPSO;


#[no_mangle]
pub extern "C" fn generate_route(
    strategy: RouteGeneratorStrategy,
    route_details: &RouteDetails,
    graph: &Graph,
    mode: GenerationMode
) -> Route {
    match strategy {
        RouteGeneratorStrategy::Greedy => RouteGeneratorGreedy::generate_route(graph, strategy, route_details, mode),
        RouteGeneratorStrategy::ACO => RouteGeneratorACO::generate_route(graph, strategy, route_details, mode),
        RouteGeneratorStrategy::PSO => RouteGeneratorPSO::generate_route(graph, strategy, route_details, mode),
        RouteGeneratorStrategy::Empty => Route(vec![]),
    }
}
