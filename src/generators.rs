use crate::{Edge, Network};

fn edge(from: usize, to: usize, capacity: u64, cost: u64) -> Edge {
    Edge {
        from,
        to,
        capacity,
        cost,
    }
}
pub fn reassignment() -> Network {
    Network {
        vertices: 6,
        source: 0,
        sink: 5,
        requested_flow: 2,
        edges: vec![
            edge(0, 1, 1, 0),
            edge(0, 2, 1, 0),
            edge(1, 3, 1, 1),
            edge(1, 4, 1, 2),
            edge(2, 3, 1, 2),
            edge(2, 4, 1, 100),
            edge(3, 5, 1, 0),
            edge(4, 5, 1, 0),
        ],
    }
}

/// SplitMix64 with specified wrapping arithmetic: stable across platforms.
pub struct Generator {
    state: u64,
}
impl Generator {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
    pub fn network(&mut self, tiny: bool) -> Network {
        let vertices = 2 + (self.next() % if tiny { 4 } else { 11 }) as usize;
        let count = self.next() % if tiny { 8 } else { 33 };
        let mut edges = Vec::new();
        for _ in 0..count {
            let from = self.next() as usize % vertices;
            let mut to = self.next() as usize % (vertices - 1);
            if to >= from {
                to += 1;
            }
            edges.push(edge(
                from,
                to,
                self.next() % if tiny { 3 } else { 6 },
                self.next() % 12,
            ));
        }
        Network {
            vertices,
            source: 0,
            sink: vertices - 1,
            requested_flow: self.next() % 10,
            edges,
        }
    }
}

/// Three warehouses, three destinations, supplies/demands of 2, 3, and 2.
pub fn transportation(seed: u64) -> Network {
    let mut rng = Generator::new(seed);
    let mut edges = Vec::new();
    for (i, supply) in [2, 3, 2].into_iter().enumerate() {
        edges.push(edge(0, 1 + i, supply, 0));
    }
    for i in 0..3 {
        for j in 0..3 {
            edges.push(edge(1 + i, 4 + j, 4, 1 + rng.next() % 20));
        }
    }
    for (j, demand) in [2, 3, 2].into_iter().enumerate() {
        edges.push(edge(4 + j, 7, demand, 0));
    }
    Network {
        vertices: 8,
        source: 0,
        sink: 7,
        requested_flow: 7,
        edges,
    }
}

pub fn allocation(seed: u64) -> Network {
    let mut rng = Generator::new(seed);
    let mut edges = Vec::new();
    for i in 0..4 {
        edges.push(edge(0, 1 + i, 1, 0));
        for j in 0..4 {
            edges.push(edge(1 + i, 5 + j, 1, rng.next() % 25));
        }
        edges.push(edge(5 + i, 9, 1, 0));
    }
    Network {
        vertices: 10,
        source: 0,
        sink: 9,
        requested_flow: 4,
        edges,
    }
}
