use completion_trie::{Completable, CompletionTree, Key};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{
    distributions::Alphanumeric, distributions::DistString, prelude::SmallRng, Rng, SeedableRng,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BenchItem(String, i32);
impl Completable for BenchItem {
    fn keys(&self) -> Vec<Key> {
        vec![Key {
            bytes: self.0.as_bytes().to_owned(),
            score: self.1,
        }]
    }
}
fn make_random_tree(prng: &mut SmallRng, n: usize) -> CompletionTree<BenchItem> {
    let mut tree = CompletionTree::default();
    for _ in 0..n {
        let name = Alphanumeric.sample_string(prng, 10);
        let score = prng.gen_range(0..1_000);
        tree.put(BenchItem(name, score));
    }
    tree
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("construct 1k", |b| {
        b.iter(|| {
            let mut prng = SmallRng::seed_from_u64(42);
            black_box(make_random_tree(&mut prng, 1_000))
        })
    });

    c.bench_function("empty query 1k", |b| {
        let tree = {
            let mut prng = SmallRng::seed_from_u64(42);
            make_random_tree(&mut prng, 1_000)
        };
        b.iter(|| black_box(tree.search(b"").nth(10)))
    });

    c.bench_function("zero match query 1k", |b| {
        let tree = {
            let mut prng = SmallRng::seed_from_u64(42);
            make_random_tree(&mut prng, 1_000)
        };
        b.iter(|| black_box(tree.search(b"blahblahgarbage").nth(10)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
