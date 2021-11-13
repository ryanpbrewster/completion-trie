use std::{
    collections::{BTreeMap, BinaryHeap},
    hash::Hash,
};

type Score = i32;
struct Scored<T> {
    pub item: T,
    pub score: Score,
}
impl<T> Ord for Scored<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score.cmp(&other.score)
    }
}
impl<T> PartialOrd for Scored<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<T> PartialEq for Scored<T> {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}
impl<T> Eq for Scored<T> {}
pub struct Key {
    pub bytes: Vec<u8>,
    pub score: Score,
}
pub trait Completable: Eq + Clone + Hash {
    fn keys(&self) -> Vec<Key>;
}

pub struct CompletionTree<T>(Option<Node<T>>);
impl<T> Default for CompletionTree<T> {
    fn default() -> Self {
        Self(None)
    }
}
impl<T> CompletionTree<T>
where
    T: Completable,
{
    pub fn put(&mut self, item: T) {
        for key in item.keys() {
            self.0
                .get_or_insert_with(|| Node::new(key.score))
                .put_key(key, item.clone());
        }
    }

    pub fn search(&self, prefix: &[u8]) -> impl Iterator<Item = &T> {
        match self.descendent(prefix) {
            None => CompletionIter::empty(),
            Some(node) => CompletionIter::from(node),
        }
    }

    fn descendent(&self, prefix: &[u8]) -> Option<&Node<T>> {
        self.0.as_ref()?.descendent(prefix)
    }
}

struct Node<T> {
    items: Vec<Scored<T>>,
    children: BTreeMap<u8, Node<T>>,
    max_score: Score,
}
impl<T> Node<T>
where
    T: Completable,
{
    fn new(max_score: Score) -> Self {
        Self {
            items: Default::default(),
            children: Default::default(),
            max_score,
        }
    }
    fn put_key(&mut self, key: Key, item: T) {
        let score = key.score;
        let mut cur = self;
        for b in key.bytes {
            cur.max_score = std::cmp::max(score, cur.max_score);
            cur = cur.children.entry(b).or_insert_with(|| Node::new(score));
        }
        cur.max_score = std::cmp::max(score, cur.max_score);
        cur.items.push(Scored { item, score });
    }

    fn descendent(&self, path: &[u8]) -> Option<&Self> {
        let mut cur = self;
        for b in path {
            cur = cur.children.get(b)?;
        }
        Some(cur)
    }
}

enum ExploreMarker<'a, T> {
    Item(&'a T),
    Node(&'a Node<T>),
}
struct CompletionIter<'a, T> {
    queue: BinaryHeap<Scored<ExploreMarker<'a, T>>>,
}
impl<'a, T> CompletionIter<'a, T> {
    fn empty() -> Self {
        Self {
            queue: BinaryHeap::new(),
        }
    }
    fn from(node: &'a Node<T>) -> Self {
        let mut queue = BinaryHeap::new();
        queue.push(Scored {
            item: ExploreMarker::Node(node),
            score: node.max_score,
        });
        Self { queue }
    }
}
impl<'a, T> Iterator for CompletionIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(cur) = self.queue.pop() {
            match cur.item {
                ExploreMarker::Item(item) => return Some(item),
                ExploreMarker::Node(node) => {
                    for item in &node.items {
                        self.queue.push(Scored {
                            item: ExploreMarker::Item(&item.item),
                            score: item.score,
                        });
                    }
                    for child in node.children.values() {
                        self.queue.push(Scored {
                            item: ExploreMarker::Node(child),
                            score: child.max_score,
                        });
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{Completable, CompletionTree, Key};
    use itertools::Itertools;

    impl Completable for (&str, i32) {
        fn keys(&self) -> Vec<Key> {
            let mut buf = Vec::new();
            let mut s = self.0;
            loop {
                buf.push(Key {
                    bytes: s.as_bytes().to_vec(),
                    score: self.1,
                });
                match s.find(' ') {
                    Some(idx) => s = &s[idx + 1..],
                    None => return buf,
                }
            }
        }
    }
    macro_rules! make_tree {
        ($($key:expr => $score:expr, )*) => {{
            let mut tree = CompletionTree::default();
            $(tree.put(($key, $score));)*
            tree
        }};
    }

    #[test]
    fn smoke_test() {
        let tree = make_tree!(
            "alice" => 1,
            "alex" => 4,
            "adam" => -3,
        );
        assert_eq!(
            tree.search(b"").map(|r| r.0).collect::<Vec<_>>(),
            ["alex", "alice", "adam"]
        );
    }

    #[test]
    fn empty_results() {
        let tree = make_tree!(
            "alice" => 1,
            "alex" => 4,
            "adam" => -3,
        );
        assert_eq!(tree.search(b"z").count(), 0);
    }

    #[test]
    fn exploration_prioritizes_shallower_items() {
        let tree = make_tree!(
            "a" => 1,
            "aa" => 0,
            "aaa" => 1,
        );
        assert_eq!(
            tree.search(b"").map(|r| r.0).collect::<Vec<_>>(),
            ["a", "aaa", "aa"]
        );
    }

    #[test]
    fn subsequences_are_not_matched() {
        // Honestly this is kind of unfortunate, I wish it worked.
        let tree = make_tree!(
            "jeffrey smith" => 1,
        );
        assert_eq!(tree.search(b"jeff smith").count(), 0);
    }

    #[test]
    fn multikey_items_example() {
        let tree = make_tree!(
            "hello world" => 1,
            "goodbye world" => 0,
        );
        assert_eq!(
            tree.search(b"h").map(|r| r.0).collect::<Vec<_>>(),
            ["hello world"]
        );
        assert_eq!(
            tree.search(b"g").map(|r| r.0).collect::<Vec<_>>(),
            ["goodbye world"]
        );
        assert_eq!(
            tree.search(b"world").map(|r| r.0).collect::<Vec<_>>(),
            ["hello world", "goodbye world"]
        );
        assert_eq!(
            tree.search(b"").map(|r| r.0).collect::<Vec<_>>(),
            // Both "hello"s come before both "goodbyes"
            [
                "hello world",
                "hello world",
                "goodbye world",
                "goodbye world"
            ]
        );

        assert_eq!(
            tree.search(b"").unique().map(|r| r.0).collect::<Vec<_>>(),
            ["hello world", "goodbye world"]
        );
    }
}
