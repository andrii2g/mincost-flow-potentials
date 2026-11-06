use mincost_flow_potentials::{bellman_ford, exhaustive, generators, verification, *};
use std::{collections::BTreeMap, error::Error as StdError, path::PathBuf};

const HELP: &str = "mincost-flow-potentials\n\n  demo <reassignment|transportation|allocation> [--seed N] [--out DIR]\n  solve <network.json> [--out DIR]\n  verify [--seed N] [--cases N] [--budget N]\n\nDefaults: seed=42, cases=1000, exhaustive budget=100000 assignments.\n";
fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), Box<dyn StdError>> {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        print!("{HELP}");
        return Ok(());
    };
    if command == "--help" || command == "-h" {
        print!("{HELP}");
        return Ok(());
    }
    if !["demo", "solve", "verify"].contains(&command.as_str()) {
        return Err(format!("unknown command: {command}\n{HELP}").into());
    }
    let target = if command != "verify" {
        Some(args.next().ok_or("missing demo name or input file")?)
    } else {
        None
    };
    let mut flags = BTreeMap::new();
    while let Some(key) = args.next() {
        let allowed: &[&str] = match command.as_str() {
            "demo" => &["--seed", "--out"],
            "solve" => &["--out"],
            _ => &["--seed", "--cases", "--budget"],
        };
        if !allowed.contains(&key.as_str()) {
            return Err(format!("unknown option: {key}").into());
        }
        let value = args.next().ok_or("missing option value")?;
        if flags.insert(key.clone(), value).is_some() {
            return Err(format!("duplicate option: {key}").into());
        }
    }
    let number = |key: &str, default: u64| -> Result<u64, Box<dyn StdError>> {
        Ok(match flags.get(key) {
            Some(v) => v.parse()?,
            None => default,
        })
    };
    let seed = number("--seed", 42)?;
    if command == "verify" {
        let cases = number("--cases", 1000)?;
        if cases == 0 {
            return Err("cases must be positive".into());
        }
        let budget = number("--budget", 100000)?;
        let mut rng = generators::Generator::new(seed);
        let (mut checked, mut skipped) = (0u64, 0u64);
        for case in 0..cases {
            let n = rng.network(case % 2 == 0);
            let result = (|| -> Result<bool, Box<dyn StdError>> {
                let a = solve(&n)?;
                let b = bellman_ford::solve(&n)?;
                verification::check(&n, &a)?;
                verification::check(&n, &b)?;
                if (a.flow, a.cost) != (b.flow, b.cost) {
                    return Err("reference mismatch".into());
                }
                match exhaustive::solve(&n, budget) {
                    Ok(c) => {
                        verification::check(&n, &c)?;
                        if (a.flow, a.cost) != (c.flow, c.cost) {
                            return Err("exhaustive mismatch".into());
                        }
                        Ok(true)
                    }
                    Err(Error::BudgetExceeded) => Ok(false),
                    Err(e) => Err(e.into()),
                }
            })();
            match result {
                Ok(true) => checked += 1,
                Ok(false) => skipped += 1,
                Err(e) => {
                    return Err(format!(
                        "seed={seed}, case={case}: {e}\n{}",
                        serde_json::to_string_pretty(&n)?
                    )
                    .into());
                }
            }
        }
        println!(
            "Verified {cases} networks (seed {seed}); exhaustive checked={checked}, skipped={skipped} (budget {budget})."
        );
        return Ok(());
    }
    let target = target.unwrap();
    let n = if command == "demo" {
        match target.as_str() {
            "reassignment" => generators::reassignment(),
            "transportation" => generators::transportation(seed),
            "allocation" => generators::allocation(seed),
            _ => return Err(format!("unknown demo: {target}").into()),
        }
    } else {
        serde_json::from_slice::<Network>(&std::fs::read(&target)?)?
    };
    let solution = solve(&n)?;
    verification::check(&n, &solution)?;
    let reference = bellman_ford::solve(&n)?;
    if (solution.flow, solution.cost) != (reference.flow, reference.cost) {
        return Err("reference mismatch".into());
    }
    let out = flags.get("--out").map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from("artifacts").join(if command == "demo" {
            target.as_str()
        } else {
            "solve"
        })
    });
    std::fs::create_dir_all(&out)?;
    std::fs::write(out.join("network.json"), serde_json::to_vec_pretty(&n)?)?;
    std::fs::write(
        out.join("solution.json"),
        serde_json::to_vec_pretty(&solution)?,
    )?;
    println!(
        "{:?}: flow={}/{}, cost={}, augmentations={}\nOutput: {}",
        solution.status,
        solution.flow,
        n.requested_flow,
        solution.cost,
        solution.metrics.augmentations,
        out.display()
    );
    Ok(())
}
