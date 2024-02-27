use std::ops::Index;

use log::{info, warn};
use osmpbfreader::NodeId;
use rand::distributions::Uniform;
use rand::prelude::Distribution;

use mgr_map_extractor::graph::Graph;
use mgr_weather::image_wrapper::{GRID_SIZE, PixelColor};

use crate::{GenerationMode, GridPosition, PixRoute, Route, RouteDetails, RouteGenerator, RouteGeneratorStrategy, WxRouteDetails};

pub struct RouteGeneratorPSO;

const MAX_ITERATIONS: u32 = 500;
const PARTICLES_COUNT: u32 = 100;
const MIN_VELOCITY: f32 = -15.0;
const MAX_VELOCITY: f32 = 15.0;

const C1_COEFF: f32 = 0.9;
const C2_COEFF: f32 = 1.25;

const STATIC_WEIGHT_COEFF: f32 = 0.99; //0.6

//Threshold fitness value at which simulation is stopped - good enough solution has been found
const FITNESS_THRESHOLD_STOP: f32 = 1.15;

const NUMBER_OF_SEED_POINTS: usize = 5;

struct Velocity2D {
    vx: f32,
    vy: f32,
}

struct Simulation {
    iteration_number: i32,
    route_details: WxRouteDetails,
    particles: Vec<Particle>,
    global_best: f32,
    global_best_position: GridPosition,
    last_fitness_value: f32,
}

struct DSimulation {
    iteration_number: i32,
    route_details: RouteDetails,
    particles: Vec<DParticle>,
    global_best: f32,
    global_best_position: Vec<NodeId>,
    last_fitness_value: f32,
}

struct Particle {
    position: GridPosition,
    velocity: Velocity2D,
    personal_best: f32,
    personal_best_position: GridPosition,
}

struct DParticle {
    position: Vec<NodeId>,
    //Position is a vector of nodes
    velocity: Velocity2D,
    personal_best: f32,
    personal_best_position: Vec<NodeId>,
}


fn designate_points_on_line(start_point: GridPosition, end_point: GridPosition, number_of_points: usize) -> Vec<GridPosition> {
    let dtx = (end_point.x - start_point.x) / number_of_points as f32;
    let dty = (end_point.y - start_point.y) / number_of_points as f32;

    let mut points = vec![];

    for point_number in 0..number_of_points {
        let tx = 0.5 * dtx + (point_number as f32) * dtx;
        let ty = 0.5 * dty + (point_number as f32) * dty;
        let check_point = GridPosition { x: (start_point.x + tx).floor(), y: (start_point.y + ty).floor() };
        //add point if it is not already in the list
        if !points.contains(&check_point) {
            points.push(check_point);
        }
    }
    points
}

impl DSimulation {
    fn new(route_details: RouteDetails) -> DSimulation {
        DSimulation {
            iteration_number: 0,
            route_details,
            particles: vec![],
            global_best: f32::MAX,
            global_best_position: vec![],
            last_fitness_value: f32::MAX,
        }
    }

    fn increase_iteration_count(&mut self) {
        self.iteration_number += 1;
    }

    fn init_particles(&mut self) {}

    fn swarm_update(&mut self, graph: &Graph) {}
}

impl DParticle{

    //Set new position while moving along discrete coordinates
    fn update_position(&mut self, graph: &Graph, global_best: &[NodeId]){
        //Update position
        let mut new_position = self.position.clone();
        let mut rng = rand::thread_rng();
        let between = Uniform::from(0..self.position.len());
        let random_index = between.sample(&mut rng);
    }

    fn update_velocity(){

    }

}

impl Particle {
    fn calculate_fitness(&self, target_point: &GridPosition) -> f32 {
        let x_diff = self.position.x - target_point.x;
        let y_diff = self.position.y - target_point.y;
        (x_diff.powf(2.0) + y_diff.powf(2.0)).sqrt()
    }

    fn update_position(&mut self, global_best_pos: GridPosition, grid: &[PixelColor]) {
        //Check if new position is inside permitted area
        //New position cannot be inside colored area

        let new_pos_x = (self.position.x + self.velocity.vx).floor();
        let new_pos_y = (self.position.y + self.velocity.vy).floor();

        let mut points_to_check = designate_points_on_line(self.position, GridPosition { x: new_pos_x, y: new_pos_y }, 15);
        points_to_check.push(GridPosition { x: new_pos_x, y: new_pos_y });

        for point in points_to_check {
            let grid_cell_index = (point.y * (GRID_SIZE as f32) + point.x) as usize;
            if grid_cell_index >= grid.len() {
                // debug!("New position is outside of grid, not updating position");
                return;
            }

            let pixel_value = grid[grid_cell_index];
            if *pixel_value.index(0) >= 1 ||
                *pixel_value.index(1) >= 1 ||
                *pixel_value.index(2) >= 1
            {
                return;
            }
        }

        self.position.x = new_pos_x;
        self.position.y = new_pos_y;
    }

    fn update_velocity_partial(global_best_pos_partial: f32, particle_best_pos_partial: f32, particle_curr_position: f32, particle_velocity: &mut f32, iteration_number: i32) {
        let between = Uniform::from(0.0..=1.0);
        let mut rng = rand::thread_rng();
        *particle_velocity = calculate_weight_coeff(iteration_number)
            * (*particle_velocity)
            + C1_COEFF * between.sample(&mut rng)
            * (particle_best_pos_partial - particle_curr_position)
            + C2_COEFF * between.sample(&mut rng)
            * (global_best_pos_partial - particle_curr_position);
    }

    fn update_velocity(&mut self, global_best_pos: GridPosition, iteration_number: i32) {
        Particle::update_velocity_partial(global_best_pos.x, self.personal_best_position.x, self.position.x, &mut self.velocity.vx, iteration_number);
        Particle::update_velocity_partial(global_best_pos.y, self.personal_best_position.y, self.position.y, &mut self.velocity.vy, iteration_number);
    }
}

fn calculate_weight_coeff(iter_number: i32) -> f32 {
    STATIC_WEIGHT_COEFF
}

impl Simulation {
    fn new(route_details: WxRouteDetails) -> Simulation {
        Simulation {
            iteration_number: 0,
            route_details,
            particles: vec![],
            global_best: f32::MAX,
            global_best_position: GridPosition { x: 0.0, y: 0.0 },
            last_fitness_value: f32::MAX,
        }
    }

    fn increase_iteration_count(&mut self) {
        self.iteration_number += 1;
    }

    fn init_particles(&mut self, starting_position: Option<GridPosition>) {
        let between = Uniform::from(MIN_VELOCITY..=MAX_VELOCITY);
        let mut rng = rand::thread_rng();

        for _ in 0..PARTICLES_COUNT {
            self.particles.push(Particle {
                position: starting_position.unwrap_or(GridPosition { x: 0.0, y: 0.0 }),
                velocity: Velocity2D { vx: between.sample(&mut rng), vy: between.sample(&mut rng) },
                personal_best: f32::MAX,
                personal_best_position: GridPosition { x: 0.0, y: 0.0 },
            });
        }
    }

    fn swarm_update(&mut self, grid: &[PixelColor]) {
        for particle in self.particles.iter_mut() {
            //Calculate fitness function for each particle
            let fitness = particle.calculate_fitness(&self.route_details.ending_position);
            self.last_fitness_value = fitness;
            //Set personal & global best values
            if fitness < particle.personal_best {
                particle.personal_best = fitness;
                particle.personal_best_position = particle.position;
            }
            if fitness < self.global_best {
                self.global_best = fitness;
                self.global_best_position = particle.position;
            }

            //Update position and velocity
            particle.update_position(self.global_best_position, grid);
            particle.update_velocity(self.global_best_position, self.iteration_number);
        }
    }
}

impl RouteGenerator for RouteGeneratorPSO {
    fn generate_route(graph: &Graph, _generation_strategy: RouteGeneratorStrategy, route_details: &RouteDetails, _mode: GenerationMode) -> Route {
        warn!("Starting discrete mode");
        let mut pix_route = Route(vec![]);

        //Spawn particles
        //Init positions
        let mut simulation = DSimulation::new(route_details.clone());
        simulation.init_particles();
        //Log simulation solution
        info!("Global best value: {} at: {:?}", simulation.global_best, simulation.global_best_position);

        for _iter_id in 0..MAX_ITERATIONS {
            if FITNESS_THRESHOLD_STOP >= simulation.last_fitness_value {
                info!("Good enough solution found, stopping simulation");
                break;
            }
            simulation.swarm_update(graph);
            // pix_route.0.push(simulation.global_best_position);
            simulation.increase_iteration_count();
        }

        info!("Global best value: {} at: {:?}", simulation.global_best, simulation.global_best_position);
        info!("Iteration count: {}", simulation.iteration_number);

        pix_route



    }

    fn real_mode_supported() -> bool {
        true
    }

    fn generate_route_real_num(route_details: &WxRouteDetails, grid: &Vec<PixelColor>) -> PixRoute {
        info!("Starting real number generation");

        let mut pix_route = PixRoute(vec![]);

        //Spawn particles
        //Init positions
        let mut simulation = Simulation::new(*route_details);
        simulation.init_particles(Some(route_details.starting_position));

        //Log simulation solution
        info!("Global best value: {} at: {:?}", simulation.global_best, simulation.global_best_position);

        for _iter_id in 0..MAX_ITERATIONS {
            if FITNESS_THRESHOLD_STOP >= simulation.last_fitness_value {
                info!("Good enough solution found, stopping simulation");
                break;
            }
            simulation.swarm_update(grid);
            pix_route.0.push(simulation.global_best_position);
            simulation.increase_iteration_count();
        }

        info!("Global best value: {} at: {:?}", simulation.global_best, simulation.global_best_position);
        info!("Iteration count: {}", simulation.iteration_number);

        pix_route
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_designate_points_on_line() {
        let start_point = super::GridPosition { x: 0.0, y: 0.0 };
        let end_point = super::GridPosition { x: 10.0, y: 10.0 };
        let number_of_points = 5;

        let points = super::designate_points_on_line(start_point, end_point, number_of_points);
        assert_eq!(points.len(), 5);
        assert_eq!(points[0], super::GridPosition { x: 1.0, y: 1.0 });
        assert_eq!(points[1], super::GridPosition { x: 3.0, y: 3.0 });
        assert_eq!(points[2], super::GridPosition { x: 5.0, y: 5.0 });
        assert_eq!(points[3], super::GridPosition { x: 7.0, y: 7.0 });
        assert_eq!(points[4], super::GridPosition { x: 9.0, y: 9.0 });
    }

    #[test]
    fn test_designate_points_on_line_2() {
        let start_point = super::GridPosition { x: 0.0, y: 0.0 };
        let end_point = super::GridPosition { x: 20.0, y: 10.0 };
        let number_of_points = 5;

        let points = super::designate_points_on_line(start_point, end_point, number_of_points);
        assert_eq!(points.len(), 5);
        assert_eq!(points[0], super::GridPosition { x: 2.0, y: 1.0 });
        assert_eq!(points[1], super::GridPosition { x: 6.0, y: 3.0 });
        assert_eq!(points[2], super::GridPosition { x: 10.0, y: 5.0 });
        assert_eq!(points[3], super::GridPosition { x: 14.0, y: 7.0 });
        assert_eq!(points[4], super::GridPosition { x: 18.0, y: 9.0 });
    }

    #[test]
    fn test_designate_points_on_line_short_route() {
        let start_point = super::GridPosition { x: 0.0, y: 0.0 };
        let end_point = super::GridPosition { x: 3.0, y: -1.0 };
        let number_of_points = 5;

        let points = super::designate_points_on_line(start_point, end_point, number_of_points);
        assert_eq!(points.len(), 3);
        assert_eq!(points[0], super::GridPosition { x: 0.0, y: -1.0 });
        assert_eq!(points[1], super::GridPosition { x: 1.0, y: -1.0 });
        assert_eq!(points[2], super::GridPosition { x: 2.0, y: -1.0 });
    }
}