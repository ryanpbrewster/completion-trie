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
        let descendent = self.0.as_ref().and_then(|root| root.descendent(prefix));
        match descendent {
            None => CompletionIter::empty(),
            Some(node) => CompletionIter::from(node),
        }
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
    macro_rules! make_tree {
        ($($e:expr),*) => {{
            let mut tree = CompletionTree::default();
            $(tree.put($e.clone());)*
            tree
        }};
    }

    #[test]
    fn smoke_test() {
        let alice = TestItem::new("alice", 1);
        let alex = TestItem::new("alex", 4);
        let adam = TestItem::new("adam", -3);

        let tree = make_tree!(alice, alex, adam);
        assert_eq!(
            tree.search(b"").collect::<Vec<_>>(),
            vec![&alex, &alice, &adam]
        )
    }

    #[test]
    fn exploration_prioritizes_shallower_items() {
        let one = TestItem::new("a", 1);
        let two = TestItem::new("aa", 0);
        let three = TestItem::new("aaa", 1);

        let tree = make_tree!(one, two, three);
        assert_eq!(
            tree.search(b"").collect::<Vec<_>>(),
            vec![&one, &three, &two]
        )
    }
}
