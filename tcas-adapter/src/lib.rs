use mgr_map_extractor::graph::{Edge, Graph};

use crate::data_feed::CongestionStats;

pub mod data_feed;

pub enum EdgeCongestionLevel {
    NA,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Default)]
pub struct TCASAdapter {
    congestion_stats: CongestionStats,
}

impl TCASAdapter {
    pub fn new() -> Self {
        TCASAdapter {
            congestion_stats: CongestionStats::new_random(),
        }
    }

    /// Read edge usage from the graph context
    pub fn read_edge_usage(&self, graph: &Graph, edge: Edge) -> Option<EdgeCongestionLevel> {
        graph.edge_by_node_id().get(&(edge.from, edge.to)).map(|_| {
            //Return random congestion level for now
            EdgeCongestionLevel::NA
            // match rand::random::<u8>() % 3 {
            //     0 => EdgeCongestionLevel::Low,
            //     1 => EdgeCongestionLevel::Medium,
            //     _ => EdgeCongestionLevel::High,
            // }
        })
    }
}


#[cfg(test)]
mod tests {}
