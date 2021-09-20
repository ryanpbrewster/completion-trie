use std::{
    collections::{hash_set, BTreeMap, BinaryHeap, HashSet},
    hash::Hash,
};

type Score = i32;
pub struct Key {
    pub bytes: Vec<u8>,
    pub score: Score,
}
pub trait Completable: Eq + Clone + Hash {
    fn keys(&self) -> Vec<Key>;
}

pub struct CompletionTree<T> {
    items: HashSet<T>,
    children: BTreeMap<u8, CompletionTree<T>>,
    max_score: Score,
}
impl<T> Default for CompletionTree<T> {
    fn default() -> Self {
        Self {
            items: Default::default(),
            children: Default::default(),
            // TODO: change a tree to be a { node: Option<Node> } struct
            // so that we don't have to populate a synthetic max_score.
            max_score: 0,
        }
    }
}

impl<T> CompletionTree<T>
where
    T: Completable,
{
    pub fn put(&mut self, item: T) {
        for key in item.keys() {
            self.put_key(key, item.clone());
        }
    }

    fn put_key(&mut self, key: Key, item: T) {
        let mut cur = self;
        for b in key.bytes {
            cur.max_score = std::cmp::max(key.score, cur.max_score);
            cur = cur.children.entry(b).or_default();
        }
        cur.max_score = std::cmp::max(key.score, cur.max_score);
        cur.items.insert(item);
    }

    pub fn search(&self, prefix: &[u8]) -> impl Iterator<Item = &T> {
        match self.descendent(prefix) {
            None => CompletionIter::empty(),
            Some(node) => CompletionIter::from(node),
        }
    }

    fn descendent(&self, path: &[u8]) -> Option<&Self> {
        let mut cur = self;
        for b in path {
            cur = cur.children.get(b)?;
        }
        Some(cur)
    }
}

struct ExploreMarker<'a, T>(&'a CompletionTree<T>);
impl<'a, T> Ord for ExploreMarker<'a, T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.max_score.cmp(&other.0.max_score)
    }
}
impl<'a, T> PartialOrd for ExploreMarker<'a, T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<'a, T> PartialEq for ExploreMarker<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.max_score == other.0.max_score
    }
}
impl<'a, T> Eq for ExploreMarker<'a, T> {}
struct CompletionIter<'a, T> {
    queue: BinaryHeap<ExploreMarker<'a, T>>,
    cur: Option<hash_set::Iter<'a, T>>,
}
impl<'a, T> CompletionIter<'a, T> {
    fn empty() -> Self {
        Self {
            queue: BinaryHeap::new(),
            cur: None,
        }
    }
    fn from(node: &'a CompletionTree<T>) -> Self {
        let mut queue = BinaryHeap::new();
        queue.push(ExploreMarker(node));
        Self {
            queue,
            cur: Some(node.items.iter()),
        }
    }

    fn poll_item(&mut self) -> Option<&'a T> {
        if let Some(iter) = self.cur.as_mut() {
            if let Some(v) = iter.next() {
                return Some(v);
            } else {
                self.cur.take();
            }
        }
        None
    }
}
impl<'a, T> Iterator for CompletionIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(item) = self.poll_item() {
                return Some(item);
            }
            match self.queue.pop() {
                None => return None,
                Some(ExploreMarker(node)) => {
                    for child in node.children.values() {
                        self.queue.push(ExploreMarker(child));
                    }
                    self.cur = Some(node.items.iter());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Completable, CompletionTree, Key};

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct TestItem(String, i32);
    impl TestItem {
        fn new(name: &str, score: i32) -> TestItem {
            TestItem(name.to_owned(), score)
        }
    }
    impl Completable for TestItem {
        fn keys(&self) -> Vec<Key> {
            vec![Key {
                bytes: self.0.as_bytes().to_owned(),
                score: self.1,
            }]
        }
    }

    #[test]
    fn smoke_test() {
        let alice = TestItem::new("alice", 1);
        let alex = TestItem::new("alex", 4);
        let adam = TestItem::new("adam", -3);

        let mut tree = CompletionTree::default();
        tree.put(alice.clone());
        tree.put(alex.clone());
        tree.put(adam.clone());
        assert_eq!(
            tree.search(b"").collect::<Vec<_>>(),
            vec![&alex, &alice, &adam]
        )
    }
}
