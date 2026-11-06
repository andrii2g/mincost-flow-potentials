use crate::Network;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Phase {
    Initial,
    BeforeAugmentation,
    AfterAugmentation,
    Terminated,
}

#[derive(Clone, Debug, Serialize)]
pub struct Frame {
    pub phase: Phase,
    pub augmentation: u64,
    pub flow: u64,
    pub cost: i128,
    pub edge_flows: Vec<u64>,
    /// Potentials used for the displayed Dijkstra distances (before), or updated potentials (after).
    pub potentials: Vec<i128>,
    /// Distances from this round's Dijkstra run; null means unreachable.
    pub distances: Vec<Option<i128>>,
    /// Residual IDs in source-to-sink order: 2*i is forward, 2*i+1 is reverse.
    pub path: Vec<usize>,
    pub amount: u64,
    pub unit_cost: i128,
    pub incremental_cost: i128,
}

#[derive(Clone, Debug, Serialize)]
pub struct Trace {
    pub network: Network,
    pub frames: Vec<Frame>,
}
