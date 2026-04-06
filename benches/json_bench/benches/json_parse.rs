mod gen_data;

use criterion::{Criterion, criterion_group, criterion_main};
use json_bench::manual_winnow;

// ── faputa ──
use faputa_derive::Parser;

#[derive(Parser)]
#[grammar("grammar.faputa")]
struct FaputaJson;

// ── pest ──
use pest::Parser as PestParser;

#[derive(pest_derive::Parser)]
#[grammar = "json.pest"]
struct PestJson;

fn bench_json(c: &mut Criterion) {
    let data = gen_data::generate_json();

    // Sanity checks
    FaputaJson::parse_json(&data).expect("faputa failed");
    manual_winnow::parse_json(&data).expect("manual winnow failed");
    PestJson::parse(Rule::json, &data).expect("pest failed");
    serde_json::from_str::<serde_json::Value>(&data).expect("serde_json failed");

    let mut group = c.benchmark_group("json_parse");

    group.bench_function("faputa", |b| {
        b.iter(|| FaputaJson::parse_json(std::hint::black_box(&data)).unwrap())
    });

    group.bench_function("winnow_manual", |b| {
        b.iter(|| manual_winnow::parse_json(std::hint::black_box(&data)).unwrap())
    });

    group.bench_function("pest", |b| {
        b.iter(|| PestJson::parse(Rule::json, std::hint::black_box(&data)).unwrap())
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::from_str::<serde_json::Value>(std::hint::black_box(&data)).unwrap())
    });

    group.finish();
}

criterion_group!(benches, bench_json);
criterion_main!(benches);
