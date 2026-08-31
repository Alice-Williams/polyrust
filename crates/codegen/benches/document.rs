use std::{hint::black_box, time::Instant};

use portable_codegen::{Document, FinalNewline, RenderOptions, render_with_stats};

fn text(value: impl Into<String>) -> Document {
    Document::text(value).expect("benchmark text has no raw controls")
}

fn representative_document(count: usize) -> Document {
    let declarations = (0..count).map(|index| {
        Document::concat([
            text(format!("declaration_{index}")),
            Document::line(),
            text("="),
            Document::line(),
            text(format!("value_{index}")),
        ])
        .group()
    });
    Document::join(Document::hard_line(), declarations)
}

fn main() {
    let document = representative_document(10_000);
    let options = RenderOptions {
        width: 88,
        final_newline: FinalNewline::Always,
        ..RenderOptions::default()
    };
    let mut best = None;
    let mut final_render = None;
    for _ in 0..10 {
        let started = Instant::now();
        let rendered = render_with_stats(black_box(&document), options).expect("benchmark renders");
        let elapsed = started.elapsed();
        best = Some(best.map_or(elapsed, |current| elapsed.min(current)));
        final_render = Some(rendered);
    }
    let rendered = final_render.expect("benchmark loop executes");
    println!(
        "document benchmark: count=10000 best_us={} output_bytes={} peak_output_capacity_bytes={} peak_pending_frames={} nodes_visited={}",
        best.expect("benchmark timing exists").as_micros(),
        rendered.stats.output_bytes,
        rendered.stats.peak_output_capacity_bytes,
        rendered.stats.peak_pending_frames,
        rendered.stats.nodes_visited,
    );
}
