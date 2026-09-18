use std::{hint::black_box, time::Duration};

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use tagtree::TagTree;

const SIZES: [usize; 2] = [1_000, 10_000];

fn shared_tags(item: u64) -> [String; 2] {
    [
        format!("area/{}/topic/{}", item % 16, (item / 16) % 32),
        format!("status/{}", item % 8),
    ]
}

fn build_shared_tree(size: usize) -> TagTree<u64> {
    let mut tree = TagTree::new();
    for item in 0..size as u64 {
        tree.set_tags(&item, shared_tags(item)).unwrap();
    }
    tree
}

fn build_wide_tree(size: usize) -> TagTree<u64> {
    let mut tree = TagTree::new();
    for item in 0..size as u64 {
        tree.set_tags(&item, [format!("record/{item:08}")]).unwrap();
    }
    tree
}

fn construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("construction");
    for size in SIZES {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("shared_taxonomy", size),
            &size,
            |b, &size| {
                b.iter(|| black_box(build_shared_tree(size)));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("wide_taxonomy", size),
            &size,
            |b, &size| {
                b.iter(|| black_box(build_wide_tree(size)));
            },
        );
    }
    group.finish();
}

fn queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("queries");
    for size in SIZES {
        let shared = build_shared_tree(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("items_under_root", size), &size, |b, _| {
            b.iter(|| black_box(shared.items_under(black_box("")).unwrap()));
        });
        group.throughput(Throughput::Elements(size.div_ceil(16) as u64));
        group.bench_with_input(
            BenchmarkId::new("items_under_branch", size),
            &size,
            |b, _| {
                b.iter(|| black_box(shared.items_under(black_box("area/0")).unwrap()));
            },
        );

        let wide = build_wide_tree(size);
        let item = (size / 2) as u64;
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::new("tags_for_wide", size), &size, |b, _| {
            b.iter(|| black_box(wide.tags_for(black_box(&item))));
        });
    }
    group.finish();
}

fn updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("updates");
    for size in SIZES {
        let mut tree = build_wide_tree(size);
        let item = (size / 2) as u64;
        let mut use_first_path = false;

        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::new("replace_existing_item", size),
            &size,
            |b, _| {
                b.iter(|| {
                    use_first_path = !use_first_path;
                    let path = if use_first_path {
                        "record/reclassified/a"
                    } else {
                        "record/reclassified/b"
                    };
                    tree.set_tags(black_box(&item), black_box([path])).unwrap();
                });
            },
        );
    }
    group.finish();
}

fn snapshots(c: &mut Criterion) {
    let mut group = c.benchmark_group("snapshots");
    for size in SIZES {
        let shared = build_shared_tree(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("shared_taxonomy", size), &size, |b, _| {
            b.iter(|| black_box(shared.summary()));
        });

        let wide = build_wide_tree(size);
        group.bench_with_input(BenchmarkId::new("wide_taxonomy", size), &size, |b, _| {
            b.iter(|| black_box(wide.summary()));
        });
    }
    group.finish();
}

fn churn(c: &mut Criterion) {
    let mut group = c.benchmark_group("churn");
    for size in SIZES {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("reclassify_single_item", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    let mut tree = TagTree::new();
                    for revision in 0..size {
                        tree.set_tags(black_box(&0_u64), [format!("revision/{revision:08}")])
                            .unwrap();
                    }
                    black_box(tree)
                });
            },
        );
    }
    group.finish();
}

fn subtree_mutations(c: &mut Criterion) {
    let mut group = c.benchmark_group("subtree_mutations");
    for size in SIZES {
        group.throughput(Throughput::Elements(size.div_ceil(16) as u64));
        group.bench_with_input(
            BenchmarkId::new("move_and_merge", size),
            &size,
            |b, &size| {
                b.iter_batched(
                    || build_shared_tree(size),
                    |mut tree| {
                        black_box(tree.move_subtree("area/0", "archive/0").unwrap());
                    },
                    BatchSize::LargeInput,
                );
            },
        );
        group.bench_with_input(BenchmarkId::new("move_to_root", size), &size, |b, &size| {
            b.iter_batched(
                || build_shared_tree(size),
                |mut tree| {
                    black_box(tree.move_subtree("area/0", "").unwrap());
                },
                BatchSize::LargeInput,
            );
        });
        group.bench_with_input(BenchmarkId::new("remove", size), &size, |b, &size| {
            b.iter_batched(
                || build_shared_tree(size),
                |mut tree| {
                    black_box(tree.remove_subtree("area/0").unwrap());
                },
                BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

fn benchmark_config() -> Criterion {
    Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3))
        .sample_size(30)
}

criterion_group! {
    name = benches;
    config = benchmark_config();
    targets = construction, queries, updates, snapshots, churn, subtree_mutations
}
criterion_main!(benches);
