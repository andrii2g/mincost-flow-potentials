//! Independent reference: rebuild residual transitions from original-edge flows.
use crate::network::*;
use std::time::Instant;

pub fn solve(n: &Network) -> Result<Solution, Error> {
    n.validate()?;
    let started = Instant::now();
    let mut flows = vec![0u64; n.edges.len()];
    let (mut flow, mut cost) = (0u64, 0i128);
    let mut metrics = Metrics::default();
    while flow < n.requested_flow {
        let mut transitions = Vec::new();
        for (i, e) in n.edges.iter().enumerate() {
            if flows[i] < e.capacity {
                transitions.push((e.from, e.to, i, true, e.cost as i128));
            }
            if flows[i] > 0 {
                transitions.push((e.to, e.from, i, false, -(e.cost as i128)));
            }
        }
        let mut dist = vec![None; n.vertices];
        let mut prev = vec![None; n.vertices];
        dist[n.source] = Some(0i128);
        for _ in 0..n.vertices - 1 {
            let mut changed = false;
            for &(u, v, i, forward, c) in &transitions {
                metrics.residual_edges_examined += 1;
                if let Some(d) = dist[u] {
                    let candidate = add(d, c)?;
                    if dist[v].is_none_or(|old| candidate < old) {
                        dist[v] = Some(candidate);
                        prev[v] = Some((u, i, forward));
                        metrics.relaxations += 1;
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        if dist[n.sink].is_none() {
            break;
        }
        let mut v = n.sink;
        let mut path = Vec::new();
        let mut amount = n.requested_flow - flow;
        while v != n.source {
            if path.len() >= n.vertices {
                return Err(Error::Invariant("reference predecessor cycle".into()));
            }
            let (u, i, forward) =
                prev[v].ok_or_else(|| Error::Invariant("reference predecessor missing".into()))?;
            amount = amount.min(if forward {
                n.edges[i].capacity - flows[i]
            } else {
                flows[i]
            });
            path.push((i, forward));
            v = u;
        }
        for (i, forward) in path {
            if forward {
                flows[i] += amount;
            } else {
                flows[i] -= amount;
            }
        }
        cost = add(cost, mul(dist[n.sink].unwrap(), amount)?)?;
        flow += amount;
        metrics.augmentations += 1;
    }
    metrics.runtime_ns = started.elapsed().as_nanos();
    Ok(Solution {
        status: if flow == n.requested_flow {
            Status::Satisfied
        } else {
            Status::InsufficientCapacity
        },
        flow,
        cost,
        edge_flows: flows,
        metrics,
    })
}
