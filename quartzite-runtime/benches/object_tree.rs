//! Benchmarks for object-tree traversal and runtime operations.

use criterion::{Criterion, criterion_group, criterion_main};
use quartzite_core::{
    AsObject, ObjectBase, Value,
    id::ConnectionId,
    meta::{
        MetaObject, noop_lookup_enum, noop_lookup_method, noop_lookup_property, noop_lookup_signal,
    },
    signal::ConnectionType,
    traits::{Object, SignalCallback},
};
use quartzite_runtime::ObjectTree;
use std::hint::black_box;

struct BenchObject {
    base: ObjectBase,
}

impl BenchObject {
    fn named(name: &str) -> Box<dyn Object> {
        Box::new(Self {
            base: ObjectBase::named(name),
        })
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

static BENCH_META: MetaObject = MetaObject::new(
    "BenchObject",
    &[],
    &[],
    &[],
    &[],
    noop_lookup_property,
    noop_lookup_signal,
    noop_lookup_method,
    noop_lookup_enum,
);

impl Object for BenchObject {
    fn meta_object(&self) -> &'static MetaObject {
        &BENCH_META
    }

    fn read_property(&self, _: &str) -> Option<Value> {
        None
    }

    fn write_property(&mut self, _: &str, _: Value) -> bool {
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

    fn emit_signal(&mut self, _: &str, _: &[Value]) -> Option<()> {
        None
    }
}

/// Builds a depth-3 tree with branching factor 4 (21 nodes total).
///
/// Node naming:
/// - Root: `"root"`
/// - Level-1 children: `"l1-{i}"`  (4 nodes)
/// - Level-2 leaves:   `"l2-{i}-{j}"` (16 nodes)
///
/// Returns `(tree, root_id)`.
fn build_bench_tree() -> (ObjectTree, quartzite_core::ObjectId) {
    let mut tree = ObjectTree::new();
    let root = tree.insert(BenchObject::named("root"), None);
    for i in 0..4u32 {
        let l1 = tree.insert(BenchObject::named(&format!("l1-{i}")), Some(root));
        for j in 0..4u32 {
            tree.insert(BenchObject::named(&format!("l2-{i}-{j}")), Some(l1));
        }
    }
    (tree, root)
}

fn bench_object_tree_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("object_tree_lookup");

    group.bench_function("find_by_name_hit", |b| {
        let (tree, _root) = build_bench_tree();
        b.iter(|| black_box(tree.find_by_name(black_box("l2-3-3"))));
    });

    group.bench_function("find_by_name_miss", |b| {
        let (tree, _root) = build_bench_tree();
        b.iter(|| black_box(tree.find_by_name(black_box("absent"))));
    });

    group.bench_function("find_by_name_in_hit", |b| {
        let (tree, root) = build_bench_tree();
        b.iter(|| black_box(tree.find_by_name_in(black_box(root), black_box("l2-3-3"))));
    });

    group.bench_function("find_by_name_in_miss", |b| {
        let (tree, root) = build_bench_tree();
        b.iter(|| black_box(tree.find_by_name_in(black_box(root), black_box("absent"))));
    });

    group.finish();
}

criterion_group!(benches, bench_object_tree_lookup);
criterion_main!(benches);
