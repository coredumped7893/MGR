use std::collections::HashMap;

use mgr_map_extractor::graph::Edge;
use mgr_map_extractor::NodeId;

pub type DroneId = u32;

#[derive(Debug, Clone, Default)]
pub struct CongestionStats {
    //Holds info about which drones are currently on which edges
    congestion: HashMap<(NodeId, NodeId), Vec<DroneId>>,
}


impl CongestionStats {
    pub fn new() -> Self {
        CongestionStats::default()
    }

    ///Generate random congestion stats
    pub fn new_random() -> Self {
        Self {
            //@TODO
            congestion: HashMap::new(),
        }
    }

    pub fn add_congestion(&mut self, edge: Edge, drone_id: DroneId) {
        let drones = self.congestion.entry((edge.from, edge.to)).or_default();
        drones.push(drone_id);
    }
    pub fn congestion(&self) -> &HashMap<(NodeId, NodeId), Vec<DroneId>> {
        &self.congestion
    }

    //Remove congestion entry for drone_id in given edge
    pub fn remove_congestion(&mut self, drone_id: DroneId, edge: Edge) {
        let drones = self.congestion.get_mut(&(edge.from, edge.to)).unwrap();
        drones.retain(|&x| x != drone_id);
    }
}
