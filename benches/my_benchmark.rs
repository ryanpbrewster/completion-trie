use completion_trie::{Completable, CompletionTree, Key};
use criterion::{criterion_group, criterion_main, Criterion};
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

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("construct 1k", |b| {
        b.iter(|| {
            let mut tree = CompletionTree::default();
            let mut prng = SmallRng::seed_from_u64(42);
            for _ in 0..1_000 {
                let name = Alphanumeric.sample_string(&mut prng, 10);
                let score = prng.gen_range(0..1_000);
                tree.put(BenchItem(name, score));
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
