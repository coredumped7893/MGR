use std::collections::HashSet;

use log::info;
use nannou::{App, color, Draw, Event, Frame};
use nannou::color::{ORANGERED, RED, Srgb, Srgba, STEELBLUE, WHITE};
use nannou::math::map_range;
use nannou::prelude::{Point2, pt2, PURPLE, Rect};

use mgr_map_extractor::{ApprovedHighwayType, CoordinateStats, NodeId, produce_connection_graph};
use mgr_map_extractor::graph::{Edge, Graph, Node};
use mgr_route_generator::{generate_route, generate_route_rn, GridPosition, PixRoute, Route, RouteDetails, RouteGeneratorStrategy, WxRouteDetails};
use mgr_weather::image_wrapper::{GRID_SIZE, PixelColor, produce_grid};
use tcas_adapter::{EdgeCongestionLevel, TCASAdapter};

const LINE_WEIGHT_BIAS: f32 = 0.5;

const ROUTE_COLOR_COMPONENTS: (u8, u8, u8) = (255, 0, 0);

const DRAW_WX_ROUTE_LINE: bool = false;

//Hardcoded routes for testing
// from 'szpital bielnaski' to 'policja na perzynskiego'
const ROUTE_1: RouteDetails = RouteDetails {
    starting_node: NodeId(2092909105),
    ending_node: NodeId(120225778),
};

//Metro marymont to urzad dzielnicy bielany
const ROUTE_2: RouteDetails = RouteDetails {
    starting_node: NodeId(3771159058),
    ending_node: NodeId(5784242419),
};

pub struct ModelWX {
    grid: Vec<PixelColor>,
    route: PixRoute,
}

pub struct ModelFullGraph {
    graph: Graph,
    coordinate_stats: CoordinateStats,
    render_mode: RenderMode,
    route: Option<Route>,
    tcas: TCASAdapter,
}

enum RenderMode {
    Nodes,
    Edges,
}

pub(crate) fn get_selected_route() -> RouteDetails {
    ROUTE_2
}

pub(crate) fn get_selected_wx_route_1() -> WxRouteDetails {
    WxRouteDetails {
        starting_position: GridPosition { x: 47.0, y: 40.0 },
        ending_position: GridPosition { x: 84.0, y: 50.0 },
    }
}

/// Return Render mode based on the RENDER_MODE environment variable
/// Default value is Edges
fn get_render_mode() -> RenderMode {
    match std::env::var("RENDER_MODE") {
        Ok(val) => {
            match val.as_str() {
                "nodes" => RenderMode::Nodes,
                "edges" => RenderMode::Edges,
                _ => RenderMode::Edges,
            }
        }
        Err(_) => RenderMode::Edges,
    }
}

pub fn model_wx(_app: &App) -> ModelWX {
    let grid = produce_grid();
    info!("Weather model initialized");

    let route = generate_route_rn(RouteGeneratorStrategy::PSO, &get_selected_wx_route_1(), &grid);
    info!("Route generated: {:?}", route);

    ModelWX {
        grid,
        route,
    }
}

pub fn model_graph(_app: &App) -> ModelFullGraph {
    let (graph, coordinate_stats) = produce_connection_graph();
    let graph_memory_size = std::mem::size_of_val(&graph);
    info!("Graph memory size: {} bytes", graph_memory_size);

    let tcas = TCASAdapter::new();

    let route_details = get_selected_route();

    let route = generate_route(&graph, RouteGeneratorStrategy::PSO, &route_details);

    info!("Render model initialized");

    ModelFullGraph {
        graph,
        coordinate_stats,
        render_mode: get_render_mode(),
        route: Some(route),
        tcas,
    }
}

pub fn event_wx(_app: &App, _model: &mut ModelWX, _event: Event) {}

pub fn event(_app: &App, _model: &mut ModelFullGraph, _event: Event) {}

fn map_grid_pos_to_canvas(grid_pos: &GridPosition, boundary: &Rect) -> Point2 {
    let grid_cell_size = (boundary.x.end - boundary.x.start).abs() / GRID_SIZE as f32;
    let cell_pos_x = map_range(grid_pos.x * grid_cell_size, 0.0, (GRID_SIZE as f32) * grid_cell_size, boundary.left(), boundary.right());
    let cell_pos_y = map_range(grid_pos.y * grid_cell_size, 0.0, (GRID_SIZE as f32) * grid_cell_size, boundary.top(), boundary.bottom());
    pt2(cell_pos_x, cell_pos_y)
}

pub fn view_weather_route(app: &App, model: &ModelWX, frame: Frame) {
    let draw = app.draw();
    draw.background().color(WHITE);
    let boundary = app.window_rect();

    draw_wx_grid(&draw, model, &boundary);

    if DRAW_WX_ROUTE_LINE {
        draw_route_ends(&draw, &boundary);

        draw_generated_wx_route(&draw, model, &boundary);
    }

    draw.to_frame(app, &frame).unwrap();
}

//Iterate over points and connect them with lines
pub fn draw_generated_wx_route(draw: &Draw, model: &ModelWX, boundary: &Rect) {
    let points = &model.route.0;

    for i in 0..points.len() - 1 {
        draw.ellipse()
            .radius(3.0)
            .xy(map_grid_pos_to_canvas(&points[i], boundary))
            .color(ORANGERED);

        draw.line()
            .start(map_grid_pos_to_canvas(&points[i], boundary))
            .end(map_grid_pos_to_canvas(&points[i + 1], boundary))
            .color(RED)
            .weight(2.0);
    }
}

// fn rand_color() -> Srgba {
//     let r = rand::random::<f32>();
//     let g = rand::random::<f32>();
//     let b = rand::random::<f32>();
//     Srgba::from_components((r, g, b, 1.0))
// }

///Draw points that represent route start and end
fn draw_route_ends(draw: &Draw, boundary: &Rect) {
    let points = get_selected_wx_route_1();
    draw.ellipse()
        .xy(map_grid_pos_to_canvas(&points.starting_position, boundary))
        .radius(8.0)
        .color(PURPLE);

    draw.ellipse()
        .xy(map_grid_pos_to_canvas(&points.ending_position, boundary))
        .radius(8.0)
        .color(PURPLE);
}

//Draw grid that represents weather radar data
//Each grid cell is represented by a max reflectivity value within that region
fn draw_wx_grid(draw: &Draw, model: &ModelWX, boundary: &Rect) {
    let grid_cell_size = (boundary.x.end - boundary.x.start).abs() / GRID_SIZE as f32;
    // let cell_pos_offset = grid_cell_size / 2.0;
    //Grid is always square
    for row in 0..GRID_SIZE {
        for column in 0..GRID_SIZE {
            let cell_color = model.grid[row * GRID_SIZE + column];
            let cell_color_converted = Srgba::from_components((cell_color.0[0], cell_color.0[1], cell_color.0[2], cell_color.0[3]));

            let cell_pos_x = map_range((column as f32) * grid_cell_size, 0.0, (GRID_SIZE as f32) * grid_cell_size, boundary.left(), boundary.right());
            let cell_pos_y = map_range((row as f32) * grid_cell_size, 0.0, (GRID_SIZE as f32) * grid_cell_size, boundary.top(), boundary.bottom());
            draw.rect()
                .color(cell_color_converted)
                .x_y(cell_pos_x, cell_pos_y)
                .w_h(grid_cell_size, grid_cell_size);
        }
    }
}


// pub fn update(_app: &App, _model: &mut Model, _update: Update) {}
pub fn view_graph_route(app: &App, model: &ModelFullGraph, frame: Frame) {
    let draw = app.draw();
    draw.background().color(WHITE);

    let boundary = app.window_rect();

    match model.render_mode {
        RenderMode::Nodes => {
            draw_nodes(&draw, model.graph.nodes(), &model.coordinate_stats, &boundary);
        }
        RenderMode::Edges => {
            draw_edges(&draw, model.graph.edges(), model.graph.nodes(), &model.coordinate_stats, &boundary, model);
        }
    }

    draw.to_frame(app, &frame).unwrap();
}

fn draw_edges(draw: &Draw, edges: &[Edge], nodes: &HashSet<Node>, coordinate_stats: &CoordinateStats, boundary: &Rect, model: &ModelFullGraph) {
    edges.iter().for_each(|edge| {
        //We assume that given node id exists :)
        let (lat_from, lon_from) = nodes.get(&Node::id(edge.from)).unwrap().get_coordinates();
        let (lat_to, lon_to) = nodes.get(&Node::id(edge.to)).unwrap().get_coordinates();

        //Map to screen coordinates
        let y_from = map_range(lat_from, coordinate_stats.min_lat, coordinate_stats.max_lat, boundary.bottom(), boundary.top());
        let x_from = map_range(lon_from, coordinate_stats.min_lon, coordinate_stats.max_lon, boundary.left(), boundary.right());
        let y_to = map_range(lat_to, coordinate_stats.min_lat, coordinate_stats.max_lat, boundary.bottom(), boundary.top());
        let x_to = map_range(lon_to, coordinate_stats.min_lon, coordinate_stats.max_lon, boundary.left(), boundary.right());


        //If edge is part of the route change color
        let mut color: Srgb<u8> = STEELBLUE;
        let mut route_weight_bias = 0.0;
        if let Some(route) = &model.route {
            if route.edges().contains(edge) {
                color = Srgb::from_components(ROUTE_COLOR_COMPONENTS);

                if let Some(edge_usage) = model.tcas.read_edge_usage(&model.graph, edge.clone()) {
                    match edge_usage {
                        EdgeCongestionLevel::Low => {
                            color = color::PALEGREEN
                        }
                        EdgeCongestionLevel::Medium => {
                            color = color::LIGHTYELLOW
                        }
                        EdgeCongestionLevel::High => {
                            color = color::ORANGERED
                        }
                        _ => {}
                    }
                }


                route_weight_bias = 1.0;
            }
        }

        let line_builder = draw.line()
            .start(pt2(x_from, y_from))
            .end(pt2(x_to, y_to))
            .color(color);

        match edge.highway_type {
            ApprovedHighwayType::Motorway => {
                line_builder.weight(route_weight_bias + 2.6 + LINE_WEIGHT_BIAS);
            }
            ApprovedHighwayType::MotorwayLink => {
                line_builder.weight(route_weight_bias + 2.6 + LINE_WEIGHT_BIAS);
            }
            ApprovedHighwayType::Trunk => {
                line_builder.weight(route_weight_bias + 2.3 + LINE_WEIGHT_BIAS);
            }
            ApprovedHighwayType::TrunkLink => {
                line_builder.weight(route_weight_bias + 2.3 + LINE_WEIGHT_BIAS);
            }
            ApprovedHighwayType::Primary => {
                line_builder.weight(route_weight_bias + 2.0 + LINE_WEIGHT_BIAS);
            }
            ApprovedHighwayType::PrimaryLink => {
                line_builder.weight(route_weight_bias + 2.0 + LINE_WEIGHT_BIAS);
            }
            ApprovedHighwayType::Secondary => {
                line_builder.weight(route_weight_bias + 1.9 + LINE_WEIGHT_BIAS);
            }
            ApprovedHighwayType::SecondaryLink => {
                line_builder.weight(route_weight_bias + 1.9 + LINE_WEIGHT_BIAS);
            }
            ApprovedHighwayType::Tertiary => {
                line_builder.weight(route_weight_bias + 1.2 + LINE_WEIGHT_BIAS);
            }
            ApprovedHighwayType::TertiaryLink => {
                line_builder.weight(route_weight_bias + 1.2 + LINE_WEIGHT_BIAS);
            }
            ApprovedHighwayType::Residential => {
                line_builder.weight(route_weight_bias + 0.7 + LINE_WEIGHT_BIAS);
            }
            ApprovedHighwayType::LivingStreet => {
                line_builder.weight(route_weight_bias + 0.5 + LINE_WEIGHT_BIAS);
            }
            _ => {}
        }
    });
}

fn draw_nodes(draw: &Draw, nodes: &HashSet<Node>, coordinate_stats: &CoordinateStats, boundary: &Rect) {
    nodes.iter().for_each(|node| {
        let (lat, lon) = node.get_coordinates();
        let y = map_range(lat, coordinate_stats.min_lat, coordinate_stats.max_lat, boundary.bottom(), boundary.top());
        let x = map_range(lon, coordinate_stats.min_lon, coordinate_stats.max_lon, boundary.left(), boundary.right());
        draw.ellipse()
            .x_y(x, y)
            .radius(1.2)
            .color(STEELBLUE);
    });
}

#[cfg(test)]
mod tests {}