#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

use std::cmp::{Ord, Ordering};
use std::iter::FromIterator;

#[derive(Debug)]
struct AvlNode<T: Ord> {
    value: T,
    left: AvlTree<T>,
    right: AvlTree<T>,
}

type AvlTree<T> = Option<Box<AvlNode<T>>>;

struct AvlTreeSet<T: Ord> {
    root: AvlTree<T>,
}

#[derive(Debug)]
struct AvlTreeSetIter<'a, T: Ord> {
    prev_nodes: Vec<&'a AvlNode<T>>,
    current_tree: &'a AvlTree<T>,
}

impl<T: Ord> AvlTreeSet<T> {
    fn new() -> Self {
        Self { root: None }
    }

    fn insert(&mut self, value: T) -> bool {
        let mut current_tree = &mut self.root;

        while let Some(current_node) = current_tree {
            match current_node.value.cmp(&value) {
                Ordering::Less => current_tree = &mut current_node.right,
                Ordering::Equal => { return false; }
                Ordering::Greater => current_tree = &mut current_node.left,
            }
        }

        *current_tree = Some(Box::new(AvlNode {
            value,
            left: None,
            right: None,
        }));

        true
    }
}

impl<'a, T: 'a + Ord> Iterator for AvlTreeSetIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match *self.current_tree {
                Some(ref current_node) => {
                    if current_node.left.is_some() {
                        self.prev_nodes.push(&current_node);
                        self.current_tree = &current_node.left;
                        continue;
                    }

                    if current_node.right.is_some() {
                        self.current_tree = &current_node.right;
                        return Some(&current_node.value);
                    }

                    self.current_tree = &None;
                    return Some(&current_node.value);
                },
                None => match self.prev_nodes.pop() {
                    Some(ref prev_node) => {
                        self.current_tree = &prev_node.right;
                        return Some(&prev_node.value);
                    },
                    None => { return None; },
                }
            }
        }
    }
}

impl<'a, T: 'a + Ord> AvlTreeSet<T> {
    fn iter(&'a self) -> AvlTreeSetIter<'a, T> {
        AvlTreeSetIter {
            prev_nodes: Vec::new(),
            current_tree: &self.root,
        }
    }
}

impl<T: Ord> FromIterator<T> for AvlTreeSet<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
        let mut set = Self::new();

        for i in iter {
            set.insert(i);
        }

        set
    }
}

#[cfg(test)]
mod properties {
    use super::*;
    use itertools::equal;
    use std::collections::BTreeSet;

    #[quickcheck]
    fn iterator_parity(input: Vec<usize>) -> bool {
        let avl_set = input.iter().cloned().collect::<AvlTreeSet<_>>();
        let btree_set = input.iter().cloned().collect::<BTreeSet<_>>();

        equal(avl_set.iter(), btree_set.iter())
    }

    #[quickcheck]
    fn insert_parity(mut bt: BTreeSet<u8>, x: u8) -> bool {
        let mut avl_set = bt.iter().cloned().collect::<AvlTreeSet<_>>();

        avl_set.insert(x) == bt.insert(x)
    }
}
