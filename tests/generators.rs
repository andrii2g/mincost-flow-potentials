use mincost_flow_potentials::{bellman_ford, generators, verification, *};
#[test]
fn seeded_generators_are_stable_and_solvers_agree() {
    let mut rng = generators::Generator::new(42);
    let mut twin = generators::Generator::new(42);
    for i in 0..500 {
        let n = rng.network(i % 2 == 0);
        assert_eq!(
            serde_json::to_string(&n).unwrap(),
            serde_json::to_string(&twin.network(i % 2 == 0)).unwrap()
        );
        let a = solve(&n).unwrap();
        let b = bellman_ford::solve(&n).unwrap();
        assert_eq!((a.flow, a.cost), (b.flow, b.cost));
        verification::check(&n, &a).unwrap();
    }
    for seed in 0..20 {
        for n in [
            generators::transportation(seed),
            generators::allocation(seed),
        ] {
            let a = solve(&n).unwrap();
            let b = bellman_ford::solve(&n).unwrap();
            assert_eq!(a.status, Status::Satisfied);
            assert_eq!((a.flow, a.cost), (b.flow, b.cost));
            verification::check(&n, &a).unwrap();
        }
    }
}
