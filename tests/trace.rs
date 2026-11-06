use mincost_flow_potentials::{generators, potentials::solve_with_trace, svg, trace::Phase, *};
#[test]
fn trace_replays_reverse_augmentation() {
    let n = generators::reassignment();
    let (s, t) = solve_with_trace(&n).unwrap();
    assert_eq!(t.frames.len(), 5);
    let before = &t.frames[3];
    let after = &t.frames[4];
    assert_eq!(before.phase, Phase::BeforeAugmentation);
    assert!(before.path.iter().any(|id| id % 2 == 1));
    assert_eq!(before.incremental_cost, 3);
    for pair in t.frames[1..].chunks_exact(2) {
        let (a, b) = (&pair[0], &pair[1]);
        let mut flows = a.edge_flows.clone();
        let mut vertex = n.source;
        for &id in &a.path {
            let e = &n.edges[id / 2];
            let (from, to) = if id % 2 == 0 {
                (e.from, e.to)
            } else {
                (e.to, e.from)
            };
            assert_eq!(vertex, from);
            vertex = to;
            if id % 2 == 0 {
                flows[id / 2] += a.amount;
            } else {
                flows[id / 2] -= a.amount;
            }
        }
        assert_eq!(vertex, n.sink);
        assert_eq!(flows, b.edge_flows);
        assert_eq!(a.cost + a.incremental_cost, b.cost);
        for (v, d) in a.distances.iter().enumerate() {
            assert_eq!(b.potentials[v], a.potentials[v] + d.unwrap_or(0));
        }
        for (i, e) in n.edges.iter().enumerate() {
            for reverse in [false, true] {
                let (u, v, cap, cost) = if reverse {
                    (e.to, e.from, b.edge_flows[i], -(e.cost as i128))
                } else {
                    (e.from, e.to, e.capacity - b.edge_flows[i], e.cost as i128)
                };
                if cap > 0 && a.distances[u].is_some() {
                    assert!(cost + b.potentials[u] - b.potentials[v] >= 0);
                }
            }
        }
    }
    assert_eq!(after.edge_flows, s.edge_flows);
    let plain = solve(&n).unwrap();
    assert_eq!(plain.cost, s.cost);
    let svg = svg::render(&t, before).unwrap();
    assert!(svg.contains("e2R r=1"));
    assert!(svg.contains("<svg"));
}
#[test]
fn trace_records_unreachable_termination() {
    let mut n = generators::reassignment();
    n.requested_flow = 3;
    let (_, t) = solve_with_trace(&n).unwrap();
    assert_eq!(t.frames.last().unwrap().phase, Phase::Terminated);
    assert_eq!(t.frames.last().unwrap().distances[n.sink], None);
    n.requested_flow = 0;
    assert_eq!(solve_with_trace(&n).unwrap().1.frames.len(), 1);
}
