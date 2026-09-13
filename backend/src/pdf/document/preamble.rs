use super::super::payload::{ExportPayload, LineSpec, LineStyle};

pub(crate) fn preamble() -> &'static str {
    r##"
#import "/mitex/prelude.typ": *
#import "/mitex/standard.typ": package
#let mitex-scope = package.scope
// box(): inline math must never break across lines
#let mi(res) = box(eval("$" + res + "$", scope: mitex-scope))
#let mm(res) = eval(mode: "markup", "$ " + res + " $", scope: mitex-scope)

#let qblock(h, body) = block(
  width: 100%, height: h, breakable: false,
  above: 2mm, below: 0mm,
  inset: (top: 1mm, bottom: 2mm),
  stroke: (bottom: (paint: rgb("#b9bcc4"), thickness: 0.6pt, dash: (2pt, 2pt))),
  body,
)

// Shrink option columns until no single option would overflow its column
// (print page `fitOptions`).
#let fitopts(target, avail, opts) = context {
  let ladder = if target == 4 { (4, 2, 1) } else if target == 2 { (2, 1) } else { (1,) }
  let widths = opts.map(o => measure(box(o), width: 100000pt).width)
  let cols = ladder.find(c => {
    let colw = (avail - (c - 1) * 2mm) / c
    widths.all(w => w <= colw)
  })
  let cols = if cols == none { 1 } else { cols }
  grid(columns: (1fr,) * cols, column-gutter: 2mm, row-gutter: 4.5mm, ..opts)
}

#let hf-row(slots, .., bindLeft: false) = {
  let s = if bindLeft { slots } else { (slots.at(2), slots.at(1), slots.at(0)) }
  grid(
    columns: (1fr, auto, 1fr),
    align: (left, center, right),
    text(size: 0.71em, fill: rgb("#999999"), s.at(0)),
    text(size: 0.71em, fill: rgb("#999999"), s.at(1)),
    text(size: 0.71em, fill: rgb("#999999"), s.at(2)),
  )
}

#let anslabel(label) = {
  v(1.4mm)
  // below spacing collapses against the next block (measured: nominal
  // 1.2mm rendered as ~0.6mm) — the label sat glued to its body text.
  // 3.5mm nominal yields a comfortable ~2mm visible gap.
  block(below: 3.5mm, text(size: 1em, fill: rgb("#666666"), weight: 600, tracking: 1.5pt, label))
}

#let ansbox(body) = block(
  width: 100%, radius: 4.5pt,
  // outset(y: 1.5mm) draws the frame 1.5mm OUTSIDE the content box, i.e.
  // into the above/below spacing — and the qblock's bottom inset eats
  // more. Measured: nominal 4mm left ~0 visible gap under a tall display
  // fraction; 8mm above yields a comfortable ~3.3mm visible gap.
  above: 8mm, below: 5mm,
  stroke: 1pt + rgb("#b9bcc4"),
  outset: (y: 1.5mm), inset: (x: 2mm, y: 2mm),
  // pad() carries the real inner clearance: tall structures (fractions,
  // cases, matrices, big operators) paint past the plain inset and touch
  // the border otherwise (measured 2-3px overlap). ~2.5mm visible padding.
  align(center, pad(y: 3mm, text(size: 1.2em, weight: 700, body))),
)

#let quoteblock(body) = block(
  width: 100%, above: 2.5mm, below: 2mm,
  inset: (left: 2mm, y: 1mm),
  stroke: (left: 1pt + rgb("#cccccc")),
  text(fill: rgb("#666666"), body),
)

#let fig(key, fmt) = align(center, image(sys.inputs.at(key), format: fmt, width: 75%))
"##
}

pub(crate) fn dash_of(spec: &LineSpec) -> String {
    match spec.style {
        LineStyle::Dotted => format!("({g:.2}pt, {g:.2}pt)", g = spec.gap.max(0.5)),
        LineStyle::Dashed => format!("(2pt, {g:.2}pt)", g = spec.gap.max(0.5)),
        _ => "\"solid\"".to_string(),
    }
}

pub(crate) fn double_lines(spec: &LineSpec, dy1: f64, dy2: f64) -> String {
    let (w1, w2) = spec.double_widths();
    format!(
        "place(bottom + left, dy: {dy1}mm, line(length: 100%, stroke: (paint: rgb(\"#b9bcc4\"), thickness: {w1:.2}pt))) \
         + place(bottom + left, dy: {dy2}mm, line(length: 100%, stroke: (paint: rgb(\"#b9bcc4\"), thickness: {w2:.2}pt)))"
    )
}

pub(crate) fn qblock_override(payload: &ExportPayload) -> String {
    let spec = &payload.sep_q;
    let flow = "#let qblock-flow(body) = block(\n  width: 100%, breakable: true,\n  above: 2mm, below: 0mm,\n  // bottom inset collapses too (nominal 2mm rendered ~1.3mm): 3mm keeps\n  // the separator clear of the last content line\n  inset: (top: 1mm, bottom: 3mm),\n";
    let mut out = String::new();
    if spec.is_none() {
        out.push_str(&format!("{flow}  body,\n)\n"));
    } else if spec.is_double() {
        out.push_str(&format!(
            "{flow}  body + ({rules}),\n)\n",
            rules = double_lines(spec, 1.6, 0.0)
        ));
    } else {
        out.push_str(&format!(
            "{flow}  stroke: (bottom: (paint: rgb(\"#b9bcc4\"), thickness: {w:.2}pt, dash: {dash})),\n  body,\n)\n",
            w = spec.width,
            dash = dash_of(spec),
        ));
    }
    let open = "#let qblock(h, body) = block(\n  width: 100%, height: h, breakable: false,\n  above: 2mm, below: 0mm,\n  inset: (top: 1mm, bottom: 3mm),\n";
    if spec.is_none() {
        out.push_str(&format!("{open}  body,\n)\n"));
        return out;
    }
    if spec.is_double() {
        out.push_str(&format!(
            "{open}  body + ({rules}),\n)\n",
            rules = double_lines(spec, 1.6, 0.0)
        ));
        return out;
    }
    out.push_str(&format!(
        "{open}  stroke: (bottom: (paint: rgb(\"#b9bcc4\"), thickness: {w:.2}pt, dash: {dash})),\n  body,\n)\n",
        w = spec.width,
        dash = dash_of(spec),
    ));
    out
}
