use std::collections::{HashMap, HashSet};

use log::{debug, info};
use osmpbfreader::NodeId;
use rand::distributions::{Distribution, WeightedIndex};
use rand::prelude::SliceRandom;
use rand::thread_rng;

use mgr_map_extractor::graph::{Edge, Graph, Node};

use crate::{GenerationMode, Route, RouteDetails, RouteGenerator, RouteGeneratorStrategy};

//We don't want to use f64 types because they are not supported on the target platform anyway (rPI Zero W)

pub struct RouteGeneratorACO;

pub const SIMULATION_COUNT: i32 = 15;

pub const ANT_COUNT: i32 = 300;

pub const ANT_MAX_MOVES_COUNT: i32 = 5500;

pub const PH_EVAPORATION_RATE: f32 = 0.4;

//How much pheromone is left on the edge after the ant has passed
pub const PH_TRACE_DELTA: f32 = 10.0;

//How strong ant is attracted to the pheromone level
pub const ALPHA_COEFF: f32 = 0.98;

//How strong ant is repelled from the edge distance
pub const BETA_COEFF: f32 = 1.37;

/// Initial level of the pheromone. Must be more than zero otherwise ants will never choose given edge
pub const PH_INITIAL_LEVEL: f32 = 100.0;


type EdgeId = (NodeId, NodeId);//FromId and ToId

#[derive(PartialEq)]
enum OptimizerMode {
    Target,
    Distance,
}

struct ACOState<'a> {
    ph_levels: HashMap<EdgeId, f32>,
    // pheromone levels that ants had left in previous runs
    best_route: Option<Route>,
    best_route_length: f32,
    best_distance_to_target: f32,
    // best route found so far
    graph: &'a Graph,
    route_details: &'a RouteDetails,
    iteration_number: i32,
    optimizer_mode: OptimizerMode,
}

impl<'a> ACOState<'a> {
    fn new(graph: &'a Graph, route_details: &'a RouteDetails) -> Self {
        Self {
            ph_levels: HashMap::new(),
            best_route: None,
            best_route_length: f32::MAX,
            best_distance_to_target: f32::MAX,
            graph,
            route_details,
            iteration_number: 0,
            optimizer_mode: OptimizerMode::Target,
        }
    }

    fn get_best_route(&self) -> &Option<Route> {
        &self.best_route
    }

    fn increase_iteration_number(&mut self) {
        self.iteration_number += 1;
    }
}

struct Ant {
    nodes_visited: HashSet<NodeId>,
}

impl Ant {
    fn new() -> Self {
        Self {
            nodes_visited: HashSet::new()
        }
    }

    //Reset ant state
    fn reset(&mut self) {
        self.nodes_visited.clear();
    }

    //Select next node to visit based on the pheromone level
    //Ant cannot select given edge twice
    fn select_next_node(&self, graph: &Graph, current_node: &Node, end_node: &Node, ph_levels: &HashMap<EdgeId, f32>) -> Option<NodeId> {
        let mut rng = thread_rng();
        //Read all possible edges from the current node
        let mut edge_candidates: Vec<Edge> = vec![];
        graph.edge_connections().get(&current_node.get_id()).unwrap().iter().for_each(|&node_id| {
            let new_edge = Edge::create(graph, current_node.get_id(), node_id);
            if !self.nodes_visited.contains(&node_id) {
                edge_candidates.push(new_edge);
            }
        });

        if edge_candidates.is_empty() {
            return None;
        }

        //Calculate weights for each edge
        let edge_candidates_chances = edge_candidates.iter().map(|edge| {
            let nom_1 = ph_levels.get(&(edge.from, edge.to)).unwrap_or(&PH_INITIAL_LEVEL);
            let distance_to_target_before = Edge::length(graph.nodes().get(&Node::id(edge.from)).unwrap(), end_node) as f32 + 0.0001;
            let distance_to_target_after = Edge::length(graph.nodes().get(&Node::id(edge.to)).unwrap(), end_node) as f32 + 0.0001;

            //@TODO optimizer select
            let delta_to_target = (distance_to_target_before - distance_to_target_after).max(1.0);

            // let nom_2 = ((1.0 / edge.length as f32) + (1.0/distance_to_target)).powf(BETA_COEFF);
            let nom_2 = (delta_to_target).powf(BETA_COEFF);

            ((edge.from, edge.to), nom_1 * nom_2)
        }).collect::<HashMap<EdgeId, f32>>();

        let total_choice_chance = edge_candidates_chances.values().sum::<f32>();

        let mut rng_weights = Vec::<f32>::with_capacity(edge_candidates.len());
        for ec in &edge_candidates {
            //Log edge candidate chances
            let e_id: EdgeId = (ec.from, ec.to);
            let e_chance = edge_candidates_chances.get(&e_id).unwrap_or(&0.0) / total_choice_chance;

            rng_weights.push(e_chance);
            // debug!("Edge from {:?} to {:?} has chance: {}", ec.from, ec.to, e_chance);
        }

        let node_distribution = match WeightedIndex::new(rng_weights.clone()) {
            Ok(v) => v,
            Err(e) => {
                panic!(
                    "Could not create index {}. Generated weights: {:#?}",
                    e, rng_weights
                );
            }
        };

        let next_node = *edge_candidates
            .get(node_distribution.sample(&mut rng))
            .unwrap();

        Some(next_node.to)
    }

    fn generate_route(&mut self, state: &ACOState) -> (Route, f32) {
        let mut route = Vec::<Edge>::new();
        let mut total_length: f32 = 0.0;
        let mut current_node = state.graph.nodes().get(&Node::id(state.route_details.starting_node)).unwrap();
        self.nodes_visited.insert(current_node.get_id());
        let mut actions_counter: i32 = 0;

        while actions_counter < ANT_MAX_MOVES_COUNT {
            //If end of route reached stop
            if current_node.get_id() == state.route_details.ending_node {
                break;
            }
            let end_node = state.graph.nodes().get(&Node::id(state.route_details.ending_node)).unwrap();
            let next_node = self.select_next_node(state.graph, current_node, end_node, &state.ph_levels);

            match next_node {
                None => {
                    // info!("Cannot select next best node");
                    break;
                }
                Some(selected_node) => {
                    let edge = Edge::create(state.graph, current_node.get_id(), selected_node);
                    self.nodes_visited.insert(selected_node);
                    route.push(edge);
                    total_length += edge.length as f32;
                    current_node = state.graph.nodes().get(&Node::id(selected_node)).unwrap();
                }
            }

            actions_counter += 1;
        }

        (Route(route), total_length)
    }
}

struct AntSwarm {
    ants: Vec<Ant>,
    route_candidates: Vec<Route>,
}

impl AntSwarm {
    fn init() -> AntSwarm {
        let mut ants = Vec::<Ant>::new();
        for _ in 0..ANT_COUNT {
            ants.push(Ant::new());
        }

        Self {
            ants,
            route_candidates: vec![],
        }
    }

    fn reset(&mut self) {
        for ant in self.ants.iter_mut() {
            ant.reset();
        }
        self.route_candidates.clear();
    }

    fn update_pheromone_levels(&self, state: &mut ACOState) {
        //Evaporate pheromone levels
        for (_, ph_level) in state.ph_levels.iter_mut() {
            *ph_level *= 1.0 - PH_EVAPORATION_RATE;
        }

        //Update pheromone levels
        for route in self.route_candidates.iter() {
            route.0.iter().for_each(|edge| {
                let ph_level = state.ph_levels.entry((edge.from, edge.to)).or_insert(PH_INITIAL_LEVEL);
                *ph_level += PH_TRACE_DELTA / (edge.length as f32 * 50.0);
            });
        }

        //Add bonus pheromone levels to the best route
        if let Some(best_route) = state.get_best_route().clone() {
            for edge in best_route.0.iter() {
                let ph_level = state.ph_levels.entry((edge.from, edge.to)).or_insert(PH_INITIAL_LEVEL);
                *ph_level += PH_TRACE_DELTA / edge.length as f32;
            }
        }
    }
}


impl RouteGenerator for RouteGeneratorACO {
    fn generate_route(graph: &Graph, _generation_strategy: RouteGeneratorStrategy, route_details: &RouteDetails, _mode: GenerationMode) -> Route {
        let mut state = ACOState::new(graph, route_details);
        let mut ant_swarm = AntSwarm::init();
        info!("Starting ACO route generation");

        for _simulation_number in 0..SIMULATION_COUNT {
            ant_swarm.reset();
            ant_swarm.ants.iter_mut().for_each(|ant| {
                let (ant_route, ant_route_length) = ant.generate_route(&state);

                //If we reach end of the route, change optimizer mode
                if ant_route.0.last().unwrap().to == route_details.ending_node && state.optimizer_mode == OptimizerMode::Target{
                    debug!("Switching to distance optimizer mode");
                    state.optimizer_mode = OptimizerMode::Distance;
                }

                //Calculate distance to the destination
                let distance_to_destination = Edge::length(
                    graph.nodes().get(&Node::id(ant_route.0.last().unwrap().to)).unwrap(),
                    graph.nodes().get(&Node::id(route_details.ending_node)).unwrap(),
                ) as f32;

                if OptimizerMode::Distance == state.optimizer_mode {
                    if distance_to_destination < state.best_distance_to_target {
                        state.best_distance_to_target = distance_to_destination;
                        state.best_route = Some(ant_route.clone());
                    }
                } else if ant_route_length < state.best_route_length {
                    state.best_route_length = ant_route_length;
                    state.best_route = Some(ant_route.clone());
                }

                ant_swarm.route_candidates.push(ant_route);
            });
            ant_swarm.update_pheromone_levels(&mut state);
        }

        info!("ACO route generation finished");
        state.best_route.unwrap_or(Route(Vec::<Edge>::new()))
    }
}