use mincost_flow_potentials::*;
fn edge(from: usize, to: usize, capacity: u64, cost: u64) -> Edge {
    Edge {
        from,
        to,
        capacity,
        cost,
    }
}
fn reassignment() -> Network {
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
#[test]
fn revises_assignment() {
    let s = solve(&reassignment()).unwrap();
    assert_eq!((s.flow, s.cost), (2, 4));
    assert_eq!(&s.edge_flows[2..6], &[0, 1, 1, 0]);
}
#[test]
fn partial_and_zero() {
    let mut n = reassignment();
    n.requested_flow = 3;
    let s = solve(&n).unwrap();
    assert_eq!(s.status, Status::InsufficientCapacity);
    assert_eq!((s.flow, s.cost), (2, 4));
    n.requested_flow = 0;
    assert_eq!(solve(&n).unwrap().cost, 0);
}
#[test]
fn rejects_invalid() {
    let mut n = reassignment();
    n.edges[0].to = 0;
    assert!(solve(&n).is_err());
}
#[test]
fn overflow_is_reported() {
    let n = Network {
        vertices: 2,
        source: 0,
        sink: 1,
        requested_flow: u64::MAX,
        edges: vec![edge(0, 1, u64::MAX, u64::MAX)],
    };
    assert_eq!(solve(&n).unwrap_err(), Error::Overflow);
}
