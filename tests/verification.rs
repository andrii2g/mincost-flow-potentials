use mincost_flow_potentials::{bellman_ford, exhaustive, verification, *};

#[test]
fn enumerated_networks_match_three_ways() {
    // Exhaust all costs 0..2 and capacities 0..2 for a graph with opposing edges.
    for code in 0u64..6561 {
        let mut x = code;
        let edges = [(0, 1), (1, 2), (0, 2), (1, 0)]
            .into_iter()
            .map(|(from, to)| {
                let capacity = x % 3;
                x /= 3;
                let cost = x % 3;
                x /= 3;
                Edge {
                    from,
                    to,
                    capacity,
                    cost,
                }
            })
            .collect();
        let n = Network {
            vertices: 3,
            source: 0,
            sink: 2,
            requested_flow: 3,
            edges,
        };
        let a = solve(&n).unwrap();
        let b = bellman_ford::solve(&n).unwrap();
        let c = exhaustive::solve(&n, 100).unwrap();
        assert_eq!((a.flow, a.cost), (b.flow, b.cost), "case {code}");
        assert_eq!((a.flow, a.cost), (c.flow, c.cost), "case {code}");
        for s in [&a, &b, &c] {
            verification::check(&n, s).unwrap();
        }
    }
}
#[test]
fn budget_is_explicit() {
    let n = Network {
        vertices: 2,
        source: 0,
        sink: 1,
        requested_flow: 1,
        edges: vec![Edge {
            from: 0,
            to: 1,
            capacity: u64::MAX,
            cost: 0,
        }],
    };
    assert_eq!(
        exhaustive::solve(&n, 100).unwrap_err(),
        Error::BudgetExceeded
    );
}
#[test]
fn rejects_feasible_but_nonoptimal_result() {
    let n = Network {
        vertices: 2,
        source: 0,
        sink: 1,
        requested_flow: 1,
        edges: vec![
            Edge {
                from: 0,
                to: 1,
                capacity: 1,
                cost: 10,
            },
            Edge {
                from: 0,
                to: 1,
                capacity: 1,
                cost: 1,
            },
        ],
    };
    let mut s = solve(&n).unwrap();
    s.edge_flows = vec![1, 0];
    s.cost = 10;
    assert!(verification::check(&n, &s).is_err());
}
#[test]
fn zero_cost_cycles_disconnected_and_parallel() {
    let n = Network {
        vertices: 5,
        source: 0,
        sink: 3,
        requested_flow: 5,
        edges: vec![
            Edge {
                from: 0,
                to: 1,
                capacity: 2,
                cost: 0,
            },
            Edge {
                from: 1,
                to: 0,
                capacity: 2,
                cost: 0,
            },
            Edge {
                from: 1,
                to: 3,
                capacity: 1,
                cost: 3,
            },
            Edge {
                from: 1,
                to: 3,
                capacity: 2,
                cost: 1,
            },
            Edge {
                from: 4,
                to: 2,
                capacity: 1,
                cost: 0,
            },
        ],
    };
    let a = solve(&n).unwrap();
    assert_eq!((a.flow, a.cost), (2, 2));
    verification::check(&n, &a).unwrap();
    let b = bellman_ford::solve(&n).unwrap();
    assert_eq!((a.flow, a.cost), (b.flow, b.cost));
}
