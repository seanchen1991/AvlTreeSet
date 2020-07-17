#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

use std::cmp;
use std::mem;
use std::cmp::{Ord, Ordering};
use std::iter::FromIterator;

#[derive(Debug, PartialEq)]
pub struct AvlNode<T: Ord> {
    pub value: T,
    pub left: AvlTree<T>,
    pub right: AvlTree<T>,
    pub height: usize,
}

pub type AvlTree<T> = Option<Box<AvlNode<T>>>;

struct AvlTreeSet<T: Ord> {
    root: AvlTree<T>,
}

#[derive(Debug)]
struct AvlTreeSetIter<'a, T: Ord> {
    prev_nodes: Vec<&'a AvlNode<T>>,
    current_tree: &'a AvlTree<T>,
}

impl<'a, T: 'a + Ord> AvlNode<T> {
    pub fn left_height(&self) -> usize {
        self.left.as_ref().map_or(0, |left| left.height)
    }

    pub fn right_height(&self) -> usize {
        self.right.as_ref().map_or(0, |right| right.height)
    }
    
    pub fn update_height(&mut self) {
        self.height = cmp::max(self.left_height(), self.right_height()) + 1;
    }

    pub fn balance_factor(&self) -> i8 {
        let left_height = self.left_height();
        let right_height = self.right_height();

        if left_height >= right_height {
            (left_height - right_height) as i8
        } else {
            -((right_height - left_height) as i8)
        }
    }

    pub fn rotate_left(&mut self) -> bool {
        if self.right.is_none() { 
            return false;
        }

        let right_node = self.right.as_mut().unwrap();
        let right_left_tree = right_node.left.take();
        let right_right_tree = right_node.right.take();

        let mut new_left_tree = mem::replace(&mut self.right, right_right_tree);
        mem::swap(&mut self.value, &mut new_left_tree.as_mut().unwrap().value);

        let left_tree = self.left.take();
        let new_left_node = new_left_tree.as_mut().unwrap();

        new_left_node.right = right_left_tree;
        new_left_node.left = left_tree;
        self.left = new_left_tree;

        if let Some(node) = self.left.as_mut() {
            node.update_height();
        }

        self.update_height();

        true
    }

    pub fn rotate_right(&mut self) -> bool {
        if self.left.is_none() {
            return false;
        }

        let left_node = self.left.as_mut().unwrap();
        let left_left_tree = left_node.left.take();
        let left_right_tree = left_node.right.take();

        let mut new_right_tree = mem::replace(&mut self.left, left_left_tree);
        mem::swap(&mut self.value, &mut new_right_tree.as_mut().unwrap().value);

        let right_tree = self.right.take();
        let new_right_node = new_right_tree.as_mut().unwrap();

        new_right_node.left = left_right_tree;
        new_right_node.right = right_tree;
        self.right = new_right_tree;

        if let Some(node) = self.right.as_mut() {
            node.update_height();
        }

        self.update_height();

        true
    }

    pub fn rebalance(&mut self) -> bool {
        match self.balance_factor() {
            -2 => {
                let right_node = self.right.as_mut().unwrap();

                if right_node.balance_factor() == 1 {
                    right_node.rotate_right();
                }

                self.rotate_left();

                true
            },
            2 => {
                let left_node = self.left.as_mut().unwrap();

                if left_node.balance_factor() == -1 {
                    left_node.rotate_left();
                }

                self.rotate_right();
                
                true
            },
            _ => false,
        }
    }
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
            height: 0,
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
