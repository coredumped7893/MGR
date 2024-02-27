use log::info;

use mgr_map_extractor::produce_connection_graph;
use mgr_route_generator::{generate_route, generate_route_rn, RouteGeneratorStrategy};
use mgr_weather::image_wrapper::produce_grid;

use crate::renderer::{event, event_wx, get_selected_wx_route_1, model_graph, model_wx, view_graph_route, view_weather_route};

mod renderer;

enum RenderTarget {
    Weather,
    Graph,
}

const ENABLE_GUI: bool = true;

const RENDER_TARGET: RenderTarget = RenderTarget::Graph;

fn main() {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339(std::time::SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        // Add blanket level filter -
        .level(log::LevelFilter::Debug)
        // - and per-module overrides
        .level_for("wgpu_core", log::LevelFilter::Warn)
        .level_for("wgpu_hal", log::LevelFilter::Warn)
        .level_for("naga", log::LevelFilter::Warn)
        // Output to stdout, files, and other Dispatch configurations
        .chain(std::io::stdout())
        // Apply globally
        .apply().unwrap();

    info!("starting up");

    if ENABLE_GUI {
        match RENDER_TARGET {
            RenderTarget::Weather => {
                nannou::app(model_wx)
                    .event(event_wx)
                    .simple_window(view_weather_route)
                    .run();
            }
            RenderTarget::Graph => {
                nannou::app(model_graph)
                    // .update(update)
                    .event(event)
                    .simple_window(view_graph_route)
                    .run();
            }
        }
    } else {
        info!("GUI disabled - Benchmark mode");
        match RENDER_TARGET {
            RenderTarget::Weather => {
                let grid = produce_grid();
                info!("Weather model initialized");
                let route = generate_route_rn(RouteGeneratorStrategy::PSO, &get_selected_wx_route_1(), &grid);
                info!("Route generated, number of waypoints: {:?}", route.0.len());
            }
            RenderTarget::Graph => {
                let route_details = crate::renderer::get_selected_route();
                let (graph, _) = produce_connection_graph();
                info!("Graph initialized - loaded: {:?} nodes", graph.nodes().len());
                info!("Generating route");
                let route = generate_route(&graph, RouteGeneratorStrategy::ACO, &route_details);
                info!("Route generated. Number of edges: {}", route.edges().len());
            }
        }

        // info!("Route: {:?}", route.edges());
        // stdin().read_line(&mut String::new()).unwrap();
    }
}
