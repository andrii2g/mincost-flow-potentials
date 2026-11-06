//! Minimum-cost flow with vertex potentials, independent oracles, and SVG traces.
//!
//! ```
//! use mincost_flow_potentials::{Edge, Network, solve, verification};
//! let network = Network {
//!     vertices: 2, source: 0, sink: 1, requested_flow: 3,
//!     edges: vec![Edge { from: 0, to: 1, capacity: 5, cost: 2 }],
//! };
//! let result = solve(&network)?;
//! verification::check(&network, &result)?;
//! assert_eq!((result.flow, result.cost), (3, 6));
//! # Ok::<(), mincost_flow_potentials::Error>(())
//! ```

pub mod network;
pub mod potentials;
pub mod svg;
pub mod trace;
pub use network::{Edge, Error, Metrics, Network, Solution, Status};
pub use potentials::solve;
pub mod bellman_ford;
pub mod exhaustive;
pub mod generators;
pub mod verification;
