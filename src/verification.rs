//! Certificate checks independent of solver state, including global negative cycles.
use crate::network::*;

pub fn check(n: &Network, s: &Solution) -> Result<(), Error> {
    n.validate()?;
    let fail = |message: &str| Error::Invariant(message.into());
    if s.edge_flows.len() != n.edges.len() || s.flow > n.requested_flow {
        return Err(fail("invalid flow shape or value"));
    }
    let mut balances = vec![0i128; n.vertices];
    let mut cost = 0;
    let mut arcs = Vec::new();
    for (e, &f) in n.edges.iter().zip(&s.edge_flows) {
        if f > e.capacity {
            return Err(fail("capacity exceeded"));
        }
        balances[e.from] = add(balances[e.from], f as i128)?;
        balances[e.to] = sub(balances[e.to], f as i128)?;
        cost = add(cost, mul(e.cost as i128, f)?)?;
        if f < e.capacity {
            arcs.push((e.from, e.to, e.cost as i128));
        }
        if f > 0 {
            arcs.push((e.to, e.from, -(e.cost as i128)));
        }
    }
    for (v, &b) in balances.iter().enumerate() {
        let expected = if v == n.source {
            s.flow as i128
        } else if v == n.sink {
            -(s.flow as i128)
        } else {
            0
        };
        if b != expected {
            return Err(fail("flow conservation failed"));
        }
    }
    if cost != s.cost {
        return Err(fail("cost mismatch"));
    }
    if (s.status == Status::Satisfied) != (s.flow == n.requested_flow) {
        return Err(fail("status mismatch"));
    }
    let mut distance = vec![0i128; n.vertices];
    for round in 0..n.vertices {
        let mut changed = false;
        for &(u, v, c) in &arcs {
            let candidate = add(distance[u], c)?;
            if candidate < distance[v] {
                if round == n.vertices - 1 {
                    return Err(fail("negative residual cycle: nonoptimal flow"));
                }
                distance[v] = candidate;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    if s.flow < n.requested_flow {
        let mut reached = vec![false; n.vertices];
        reached[n.source] = true;
        for _ in 0..n.vertices {
            for &(u, v, _) in &arcs {
                if reached[u] {
                    reached[v] = true;
                }
            }
        }
        if reached[n.sink] {
            return Err(fail("partial flow is not maximum"));
        }
    }
    Ok(())
}
