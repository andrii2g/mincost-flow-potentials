//! Standalone SVG rendering. Edge IDs connect the diagram to an exact numeric ledger.
use crate::{
    Error,
    trace::{Frame, Phase, Trace},
};
use std::{fmt::Write, path::Path};

pub fn render(trace: &Trace, frame: &Frame) -> Result<String, Error> {
    let n = &trace.network;
    n.validate()?;
    if frame.edge_flows.len() != n.edges.len()
        || frame.potentials.len() != n.vertices
        || frame.distances.len() != n.vertices
        || frame.path.iter().any(|&id| id / 2 >= n.edges.len())
        || frame
            .edge_flows
            .iter()
            .zip(&n.edges)
            .any(|(&f, e)| f > e.capacity)
    {
        return Err(Error::Invariant("invalid SVG frame".into()));
    }
    let height = 850usize.max(240 + n.edges.len() * 38);
    let mut s = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="1500" height="{height}" viewBox="0 0 1500 {height}">
<defs><marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0 0L10 5L0 10Z" fill="context-stroke"/></marker></defs>
<style>text{{font-family:Consolas,monospace;fill:#dce6f5;white-space:pre}}.small{{font-size:13px}}.label{{font-size:12px;paint-order:stroke;stroke:#101b2c;stroke-width:5px;stroke-linejoin:round}}.head{{font-size:16px;font-weight:bold}}</style>
<rect width="1500" height="{height}" fill="#0b1321"/><rect x="24" y="130" width="862" height="660" rx="18" fill="#101b2c"/>
"##
    );
    writeln!(s,r#"<text x="32" y="42" font-size="25" font-weight="bold">Minimum-cost flow / residual network</text>"#).unwrap();
    writeln!(s,r#"<text x="32" y="76" class="head">{:?} / augmentation {} / flow {} of {} / cost {}</text>"#,frame.phase,frame.augmentation,frame.flow,n.requested_flow,frame.cost).unwrap();
    writeln!(s,r#"<text x="32" y="105" class="small">Send {} / path unit cost {} / incremental cost {}</text>"#,frame.amount,frame.unit_cost,frame.incremental_cost).unwrap();
    let points: Vec<_> = (0..n.vertices)
        .map(|v| {
            let angle = std::f64::consts::TAU * v as f64 / n.vertices as f64 + std::f64::consts::PI;
            (455.0 + 295.0 * angle.cos(), 450.0 + 235.0 * angle.sin())
        })
        .collect();
    for (i, e) in n.edges.iter().enumerate() {
        let pair = (e.from.min(e.to), e.from.max(e.to));
        let rank = n.edges[..i]
            .iter()
            .filter(|x| (x.from.min(x.to), x.from.max(x.to)) == pair)
            .count();
        for reverse in [false, true] {
            let id = i * 2 + usize::from(reverse);
            let selected = frame.path.contains(&id);
            let remaining = if reverse {
                frame.edge_flows[i]
            } else {
                e.capacity - frame.edge_flows[i]
            };
            if reverse && remaining == 0 && !selected {
                continue;
            }
            let (u, v) = if reverse {
                (e.to, e.from)
            } else {
                (e.from, e.to)
            };
            let (x1, y1) = points[u];
            let (x2, y2) = points[v];
            let (ax, ay) = points[pair.0];
            let (bx, by) = points[pair.1];
            let length = (bx - ax).hypot(by - ay);
            let bend = (32.0 + rank as f64 * 48.0) * if reverse { -1.0 } else { 1.0 };
            let (cx, cy) = (
                (ax + bx) / 2.0 - (by - ay) / length * bend,
                (ay + by) / 2.0 + (bx - ax) / length * bend,
            );
            let l1 = (cx - x1).hypot(cy - y1);
            let l2 = (cx - x2).hypot(cy - y2);
            let (sx, sy) = (x1 + (cx - x1) / l1 * 32.0, y1 + (cy - y1) / l1 * 32.0);
            let (tx, ty) = (x2 + (cx - x2) / l2 * 36.0, y2 + (cy - y2) / l2 * 36.0);
            let color = if selected {
                "#ffcb6b"
            } else if remaining == 0 {
                "#445269"
            } else if reverse {
                "#c792ea"
            } else {
                "#6cb6e9"
            };
            let dash = if reverse { "7 5" } else { "none" };
            writeln!(s,r#"<path d="M{sx:.1} {sy:.1} Q{cx:.1} {cy:.1} {tx:.1} {ty:.1}" fill="none" stroke="{color}" stroke-width="{}" stroke-dasharray="{dash}" marker-end="url(#arrow)"/>"#,if selected { 4 } else { 2 }).unwrap();
            let (lx, ly) = ((sx + 2.0 * cx + tx) / 4.0, (sy + 2.0 * cy + ty) / 4.0);
            writeln!(s,r#"<text x="{lx:.1}" y="{ly:.1}" text-anchor="middle" class="label">e{i}{} r={remaining}</text>"#,if reverse { "R" } else { "" }).unwrap();
        }
    }
    for (v, &(x, y)) in points.iter().enumerate() {
        let color = if v == n.source {
            "#235849"
        } else if v == n.sink {
            "#634c26"
        } else {
            "#263c58"
        };
        writeln!(s,r#"<circle cx="{x:.1}" cy="{y:.1}" r="29" fill="{color}" stroke="rgb(182,201,223)" stroke-width="2"/><text x="{x:.1}" y="{:.1}" text-anchor="middle" font-size="19">{v}</text>"#,y+6.0).unwrap();
        let distance = frame.distances[v]
            .map(|d| d.to_string())
            .unwrap_or_else(|| "unreached".into());
        writeln!(s,r#"<text x="{x:.1}" y="{:.1}" text-anchor="middle" class="label">pi={} d={distance}</text>"#,y+48.0,frame.potentials[v]).unwrap();
    }
    writeln!(s,r#"<text x="920" y="154" class="head">Original edges / current residual capacities</text><text x="920" y="185" class="small">ID  u&gt;v    flow/cap    cost    fwd / rev</text>"#).unwrap();
    for (i, e) in n.edges.iter().enumerate() {
        let f = frame.edge_flows[i];
        let selected = frame.path.iter().any(|id| id / 2 == i);
        let color = if selected { "#ffcb6b" } else { "#dce6f5" };
        writeln!(s,r#"<text x="920" y="{}" class="small" style="fill:{color}">e{:<3} {}&gt;{:<3} {:>3}/{:<3} {:>7} {:>5} / {}</text>"#,220+i*38,i,e.from,e.to,f,e.capacity,e.cost,e.capacity-f,f).unwrap();
    }
    let footer = height - 36;
    writeln!(s,r#"<text x="32" y="{footer}" class="small">Gold: selected path | Dashed purple: reverse residual (cost = -original cost) | r: remaining capacity</text>"#).unwrap();
    let note = if frame.phase == Phase::AfterAugmentation {
        "Potentials updated; distances belong to the preceding search. Highlight shows the path just used."
    } else {
        "Distances use the displayed potentials. R means traversal reverses an original edge. Source/sink are green/gold."
    };
    writeln!(
        s,
        r#"<text x="32" y="{}" class="small">{note}</text></svg>"#,
        footer - 24
    )
    .unwrap();
    Ok(s)
}

pub fn write_frames(trace: &Trace, directory: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(directory)?;
    for (i, frame) in trace.frames.iter().enumerate() {
        std::fs::write(
            directory.join(format!("frame-{i:03}.svg")),
            render(trace, frame)?,
        )?;
    }
    // Remove only surplus files in our numbered-frame namespace on a shorter rerun.
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if let Some(digits) = name
            .strip_prefix("frame-")
            .and_then(|s| s.strip_suffix(".svg"))
            && !digits.is_empty()
            && digits.bytes().all(|b| b.is_ascii_digit())
            && digits
                .parse::<usize>()
                .is_ok_and(|index| index >= trace.frames.len())
            && entry.file_type()?.is_file()
        {
            std::fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}
