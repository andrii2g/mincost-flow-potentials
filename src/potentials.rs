use crate::network::*;
use crate::trace::{Frame, Phase, Trace};
use std::{cmp::Reverse, collections::BinaryHeap, time::Instant};

pub fn solve(n: &Network) -> Result<Solution, Error> {
    run(n, None)
}

pub fn solve_with_trace(n: &Network) -> Result<(Solution, Trace), Error> {
    let mut frames = Vec::new();
    let solution = run(n, Some(&mut frames))?;
    Ok((
        solution,
        Trace {
            network: n.clone(),
            frames,
        },
    ))
}

fn run(n: &Network, mut frames: Option<&mut Vec<Frame>>) -> Result<Solution, Error> {
    n.validate()?;
    let started = Instant::now();
    let mut r = Residual::new(n);
    let mut pi = vec![0i128; n.vertices];
    let (mut flow, mut cost) = (0u64, 0i128);
    let mut metrics = Metrics::default();
    if let Some(frames) = frames.as_mut() {
        frames.push(Frame {
            phase: Phase::Initial,
            augmentation: 0,
            flow,
            cost,
            edge_flows: r.flows(),
            potentials: pi.clone(),
            distances: vec![None; n.vertices],
            path: vec![],
            amount: 0,
            unit_cost: 0,
            incremental_cost: 0,
        });
    }
    while flow < n.requested_flow {
        let mut dist = vec![None; n.vertices];
        let mut prev = vec![None; n.vertices];
        let mut heap = BinaryHeap::new();
        dist[n.source] = Some(0i128);
        heap.push(Reverse((0i128, n.source)));
        metrics.heap_pushes += 1;
        while let Some(Reverse((d, u))) = heap.pop() {
            metrics.heap_pops += 1;
            if dist[u] != Some(d) {
                metrics.stale_pops += 1;
                continue;
            }
            for &id in &r.adjacency[u] {
                metrics.residual_edges_examined += 1;
                let e = &r.arcs[id];
                if e.capacity == 0 {
                    continue;
                }
                let reduced = sub(add(e.cost, pi[u])?, pi[e.to])?;
                if reduced < 0 {
                    return Err(Error::Invariant("negative reachable reduced cost".into()));
                }
                let candidate = add(d, reduced)?;
                if dist[e.to].is_none_or(|old| candidate < old) {
                    dist[e.to] = Some(candidate);
                    prev[e.to] = Some(id);
                    heap.push(Reverse((candidate, e.to)));
                    metrics.heap_pushes += 1;
                    metrics.relaxations += 1;
                }
            }
        }
        if dist[n.sink].is_none() {
            if let Some(frames) = frames.as_mut() {
                frames.push(Frame {
                    phase: Phase::Terminated,
                    augmentation: metrics.augmentations,
                    flow,
                    cost,
                    edge_flows: r.flows(),
                    potentials: pi.clone(),
                    distances: dist.clone(),
                    path: vec![],
                    amount: 0,
                    unit_cost: 0,
                    incremental_cost: 0,
                });
            }
            break;
        }
        let mut path = Vec::new();
        let mut v = n.sink;
        let mut amount = n.requested_flow - flow;
        let mut unit_cost = 0;
        while v != n.source {
            let id = prev[v].ok_or_else(|| Error::Invariant("missing predecessor".into()))?;
            amount = amount.min(r.arcs[id].capacity);
            unit_cost = add(unit_cost, r.arcs[id].cost)?;
            path.push(id);
            v = r.arcs[id].from;
        }
        path.reverse();
        let incremental_cost = mul(unit_cost, amount)?;
        if let Some(frames) = frames.as_mut() {
            frames.push(Frame {
                phase: Phase::BeforeAugmentation,
                augmentation: metrics.augmentations + 1,
                flow,
                cost,
                edge_flows: r.flows(),
                potentials: pi.clone(),
                distances: dist.clone(),
                path: path.clone(),
                amount,
                unit_cost,
                incremental_cost,
            });
        }
        for (v, d) in dist.iter().enumerate() {
            if let Some(d) = d {
                pi[v] = add(pi[v], *d)?;
                metrics.vertices_updated += 1;
            }
        }
        metrics.potential_update_rounds += 1;
        cost = add(cost, incremental_cost)?;
        for &id in &path {
            r.arcs[id].capacity -= amount;
            r.arcs[id ^ 1].capacity = r.arcs[id ^ 1]
                .capacity
                .checked_add(amount)
                .ok_or(Error::Overflow)?;
        }
        flow += amount;
        metrics.augmentations += 1;
        if let Some(frames) = frames.as_mut() {
            frames.push(Frame {
                phase: Phase::AfterAugmentation,
                augmentation: metrics.augmentations,
                flow,
                cost,
                edge_flows: r.flows(),
                potentials: pi.clone(),
                distances: dist,
                path,
                amount,
                unit_cost,
                incremental_cost,
            });
        }
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
        edge_flows: r.flows(),
        metrics,
    })
}
