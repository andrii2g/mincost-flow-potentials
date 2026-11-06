//! Enumerates original-edge assignments, independently of residual solvers.
use crate::network::*;
use std::time::Instant;

pub fn solve(n: &Network, budget: u64) -> Result<Solution, Error> {
    n.validate()?;
    let started = Instant::now();
    let states = n.edges.iter().try_fold(1u64, |p, e| {
        p.checked_mul(e.capacity.checked_add(1).ok_or(Error::BudgetExceeded)?)
            .filter(|&s| s <= budget)
            .ok_or(Error::BudgetExceeded)
    })?;
    if states > budget {
        return Err(Error::BudgetExceeded);
    }
    let mut current = vec![0; n.edges.len()];
    let mut best = Solution {
        status: Status::InsufficientCapacity,
        flow: 0,
        cost: 0,
        edge_flows: current.clone(),
        metrics: Metrics::default(),
    };
    loop {
        let mut balance = vec![0i128; n.vertices];
        for (e, &f) in n.edges.iter().zip(&current) {
            balance[e.from] = add(balance[e.from], f as i128)?;
            balance[e.to] = sub(balance[e.to], f as i128)?;
        }
        let value = balance[n.source];
        if value >= 0
            && value <= n.requested_flow as i128
            && balance[n.sink] == -value
            && balance
                .iter()
                .enumerate()
                .all(|(v, &b)| v == n.source || v == n.sink || b == 0)
        {
            let mut cost = 0;
            for (e, &f) in n.edges.iter().zip(&current) {
                cost = add(cost, mul(e.cost as i128, f)?)?;
            }
            if value as u64 > best.flow || (value as u64 == best.flow && cost < best.cost) {
                best.flow = value as u64;
                best.cost = cost;
                best.edge_flows.clone_from(&current);
            }
        }
        let mut index = 0;
        while index < current.len() && current[index] == n.edges[index].capacity {
            current[index] = 0;
            index += 1;
        }
        if index == current.len() {
            break;
        }
        current[index] += 1;
    }
    best.status = if best.flow == n.requested_flow {
        Status::Satisfied
    } else {
        Status::InsufficientCapacity
    };
    best.metrics.runtime_ns = started.elapsed().as_nanos();
    Ok(best)
}
