//! Benchmarks for the derive-macro `Object` implementation.
// Bench fixtures opt out of the undocumented-item diagnostic via per-block
// `undocumented = "allow"` attrs (doc prose on internal fixtures is noise).
#![allow(deprecated)]

use criterion::{Criterion, criterion_group, criterion_main};
use quartzite::core::{Object, ObjectBase, Signal, Value};
use quartzite::macros::{Extend, Object, object_impl};
use quartzite::prelude::emit;
use std::hint::black_box;

#[derive(Extend, Object)]
#[root]
#[extend(undocumented = "allow")]
#[object(undocumented = "allow")]
struct BenchObject {
    #[base]
    object_base: ObjectBase,
    #[prop(notify = count_changed)]
    pub count: i64,
    #[signal]
    pub count_changed: Signal<(i64,)>,
}

#[object_impl]
impl BenchObject {}

impl BenchObject {
    fn new() -> Self {
        Self {
            object_base: ObjectBase::new(),
            count: 0,
            count_changed: Signal::new(),
        }
    }
}

fn bench_signal_emit(c: &mut Criterion) {
    let mut group = c.benchmark_group("signal_emit");

    group.bench_function("typed_no_slots", |b| {
        let mut obj = BenchObject::new();
        b.iter(|| obj.count_changed.emit_unconditionally(&(black_box(1i64),)));
    });

    group.bench_function("typed_one_slot", |b| {
        let mut obj = BenchObject::new();
        obj.count_changed.connect(|args| {
            let _ = black_box(args.0);
        });
        b.iter(|| obj.count_changed.emit_unconditionally(&(black_box(1i64),)));
    });

    group.bench_function("emit_macro_one_slot", |b| {
        let mut obj = BenchObject::new();
        obj.count_changed.connect(|args| {
            let _ = black_box(args.0);
        });
        b.iter(|| emit!(obj.count_changed, &(black_box(1i64),)));
    });

    group.bench_function("dynamic_emit_signal", |b| {
        let mut obj = BenchObject::new();
        obj.count_changed.connect(|args| {
            let _ = black_box(args.0);
        });
        let args = [Value::Int(1)];
        b.iter(|| {
            let _ =
                black_box(obj.emit_signal(black_box("count_changed"), black_box(args.as_slice())));
        });
    });

    group.finish();
}

fn bench_property_rw(c: &mut Criterion) {
    let mut group = c.benchmark_group("property_rw");

    group.bench_function("read", |b| {
        let obj = BenchObject::new();
        b.iter(|| black_box(obj.read_property(black_box("count"))));
    });

    // write_property with #[prop(notify = count_changed)] also emits the notify signal —
    // this is intentional: the bench measures the realistic macro-generated write path.
    group.bench_function("write", |b| {
        let mut obj = BenchObject::new();
        b.iter(|| black_box(obj.write_property(black_box("count"), Value::Int(42))));
    });

    group.finish();
}

criterion_group!(benches, bench_signal_emit, bench_property_rw);
criterion_main!(benches);
