//! Benchmarks for signal emission and property read/write operations.

use criterion::{Criterion, criterion_group, criterion_main};
use quartzite_core::{
    AsObject, ObjectBase, Signal, Value,
    id::ConnectionId,
    meta::{
        MetaObject, ParamMeta, PropertyFlag, PropertyMeta, SignalMeta, noop_lookup_enum,
        noop_lookup_method,
    },
    signal::ConnectionType,
    traits::{Object, SignalCallback},
};
use std::hint::black_box;

struct BenchObject {
    base: ObjectBase,
    count: i64,
    sig: Signal<(i32,)>,
}

impl BenchObject {
    fn new() -> Self {
        Self {
            base: ObjectBase::new(),
            count: 0,
            sig: Signal::new(),
        }
    }
}

impl AsObject for BenchObject {
    fn object_base(&self) -> &ObjectBase {
        &self.base
    }

    fn object_base_mut(&mut self) -> &mut ObjectBase {
        &mut self.base
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn core::any::Any {
        self
    }
}

fn lookup_property(name: &str) -> Option<PropertyMeta> {
    match name {
        "count" => Some(PropertyMeta::new(
            "count",
            "i64",
            PropertyFlag::read_write(),
        )),
        _ => None,
    }
}

fn lookup_signal(name: &str) -> Option<SignalMeta> {
    static SIG_PARAMS: [ParamMeta; 1] = [ParamMeta::new("value", "i32")];
    match name {
        "sig" => Some(SignalMeta::new("sig", &SIG_PARAMS)),
        _ => None,
    }
}

static BENCH_META: MetaObject = MetaObject::new(
    "BenchObject",
    &[PropertyMeta::new(
        "count",
        "i64",
        PropertyFlag::read_write(),
    )],
    &[SignalMeta::new("sig", &[ParamMeta::new("value", "i32")])],
    &[],
    &[],
    lookup_property,
    lookup_signal,
    noop_lookup_method,
    noop_lookup_enum,
);

impl Object for BenchObject {
    fn meta_object(&self) -> &'static MetaObject {
        &BENCH_META
    }

    fn read_property(&self, name: &str) -> Option<Value> {
        (name == "count").then(|| Value::Int(self.count))
    }

    fn write_property(&mut self, name: &str, val: Value) -> bool {
        if name == "count"
            && let Value::Int(v) = val
        {
            self.count = v;
            return true;
        }
        false
    }

    fn invoke_method(&mut self, _: &str, _: &[Value]) -> Option<Value> {
        None
    }

    fn connect_signal(
        &mut self,
        _: &str,
        _: SignalCallback,
        _: ConnectionType,
    ) -> Option<ConnectionId> {
        None
    }

    fn emit_signal(&mut self, name: &str, _args: &[Value]) -> Option<()> {
        if name == "sig" {
            self.sig.emit_unconditionally(&(0,));
            Some(())
        } else {
            None
        }
    }
}

fn bench_signal_emit(c: &mut Criterion) {
    let mut group = c.benchmark_group("signal_emit");

    group.bench_function("typed_no_slots", |b| {
        let mut obj = BenchObject::new();
        b.iter(|| obj.sig.emit_unconditionally(&(black_box(1i32),)));
    });

    group.bench_function("typed_one_slot", |b| {
        let mut obj = BenchObject::new();
        obj.sig.connect(|args| {
            let _ = black_box(args.0);
        });
        b.iter(|| obj.sig.emit_unconditionally(&(black_box(1i32),)));
    });

    group.bench_function("emit_macro_one_slot", |b| {
        let mut obj = BenchObject::new();
        obj.sig.connect(|args| {
            let _ = black_box(args.0);
        });
        b.iter(|| quartzite_core::emit!(obj.sig, &(black_box(1i32),)));
    });

    group.bench_function("dynamic_emit_signal", |b| {
        let mut obj = BenchObject::new();
        obj.sig.connect(|args| {
            let _ = black_box(args.0);
        });
        let args = [Value::Int(1)];
        b.iter(|| {
            let _ = black_box(obj.emit_signal(black_box("sig"), black_box(args.as_slice())));
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

    group.bench_function("write", |b| {
        let mut obj = BenchObject::new();
        b.iter(|| black_box(obj.write_property(black_box("count"), Value::Int(42))));
    });

    group.finish();
}

criterion_group!(benches, bench_signal_emit, bench_property_rw);
criterion_main!(benches);
