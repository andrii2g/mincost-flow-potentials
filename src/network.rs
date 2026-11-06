use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub capacity: u64,
    pub cost: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Network {
    pub vertices: usize,
    pub source: usize,
    pub sink: usize,
    pub requested_flow: u64,
    pub edges: Vec<Edge>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidNetwork(String),
    Overflow,
    Invariant(String),
    BudgetExceeded,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
pub(crate) fn add(a: i128, b: i128) -> Result<i128, Error> {
    a.checked_add(b).ok_or(Error::Overflow)
}
pub(crate) fn sub(a: i128, b: i128) -> Result<i128, Error> {
    a.checked_sub(b).ok_or(Error::Overflow)
}
pub(crate) fn mul(a: i128, b: u64) -> Result<i128, Error> {
    a.checked_mul(b as i128).ok_or(Error::Overflow)
}

impl Network {
    pub fn validate(&self) -> Result<(), Error> {
        if !(2..=12).contains(&self.vertices)
            || self.source >= self.vertices
            || self.sink >= self.vertices
            || self.source == self.sink
        {
            return Err(Error::InvalidNetwork(
                "expected 2–12 vertices and distinct valid terminals".into(),
            ));
        }
        for e in &self.edges {
            if e.from >= self.vertices || e.to >= self.vertices || e.from == e.to {
                return Err(Error::InvalidNetwork(
                    "invalid edge endpoints or self-loop".into(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Status {
    Satisfied,
    InsufficientCapacity,
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct Metrics {
    pub augmentations: u64,
    pub heap_pushes: u64,
    pub heap_pops: u64,
    pub stale_pops: u64,
    pub relaxations: u64,
    pub residual_edges_examined: u64,
    pub potential_update_rounds: u64,
    pub vertices_updated: u64,
    pub runtime_ns: u128,
}
#[derive(Clone, Debug, Serialize)]
pub struct Solution {
    pub status: Status,
    pub flow: u64,
    pub cost: i128,
    pub edge_flows: Vec<u64>,
    pub metrics: Metrics,
}

#[derive(Clone, Debug)]
pub(crate) struct Arc {
    pub from: usize,
    pub to: usize,
    pub capacity: u64,
    pub cost: i128,
}
pub(crate) struct Residual {
    pub arcs: Vec<Arc>,
    pub adjacency: Vec<Vec<usize>>,
}
impl Residual {
    pub fn new(n: &Network) -> Self {
        let mut r = Self {
            arcs: Vec::new(),
            adjacency: vec![vec![]; n.vertices],
        };
        for e in &n.edges {
            let id = r.arcs.len();
            r.arcs.push(Arc {
                from: e.from,
                to: e.to,
                capacity: e.capacity,
                cost: e.cost as i128,
            });
            r.arcs.push(Arc {
                from: e.to,
                to: e.from,
                capacity: 0,
                cost: -(e.cost as i128),
            });
            r.adjacency[e.from].push(id);
            r.adjacency[e.to].push(id ^ 1);
        }
        r
    }
    pub fn flows(&self) -> Vec<u64> {
        self.arcs.chunks_exact(2).map(|p| p[1].capacity).collect()
    }
}
