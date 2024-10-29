use std::{
    collections::VecDeque,
    marker::PhantomData,
    ops::{Index, IndexMut},
};

#[derive(Clone, Copy)]
pub enum ChildSide {
    Left,
    Right,
}

#[derive(Debug)]
pub struct Node<T> {
    pub(crate) data: T,
    pub(crate) depth: usize,
    pub(crate) parent: usize,
    pub(crate) left: Option<usize>,
    pub(crate) right: Option<usize>,
}

impl<T> Node<T> {
    fn new(data: T, depth: usize, parent: usize) -> Self {
        Node {
            data,
            depth,
            parent,
            left: None,
            right: None,
        }
    }
}

impl<T> IndexMut<ChildSide> for Node<T> {
    fn index_mut(&mut self, index: ChildSide) -> &mut Self::Output {
        match index {
            ChildSide::Left => &mut self.left,
            ChildSide::Right => &mut self.right,
        }
    }
}

impl<T> Index<ChildSide> for Node<T> {
    type Output = Option<usize>;

    fn index(&self, index: ChildSide) -> &Option<usize> {
        match index {
            ChildSide::Left => &self.left,
            ChildSide::Right => &self.right,
        }
    }
}

#[cfg(test)]
mod node_test {
    use super::{ChildSide, Node};

    #[test]
    fn test_node_new() {
        let node = Node::new(1, 1, 1);
        assert_eq!(node.data, 1);
        assert_eq!(node.depth, 1);
        assert_eq!(node.parent, 1);
        assert_eq!(node.left, None);
        assert_eq!(node.right, None);
    }

    #[test]
    fn test_node_index() {
        let node = Node::new(1, 1, 0);
        assert_eq!(node[ChildSide::Left], None);
        assert_eq!(node[ChildSide::Right], None);
    }

    #[test]
    fn test_node_index_mut() {
        let mut node = Node::new(1, 0, 0);
        node[ChildSide::Left] = Some(2);
        node[ChildSide::Right] = Some(3);
        assert_eq!(node.left, Some(2));
        assert_eq!(node.right, Some(3));
    }
}

#[derive(Debug)]
pub struct Tree<T> {
    pub(crate) data: Vec<Node<T>>,
    pub(crate) max_depth: usize,
}

impl<T> Tree<T> {
    pub(crate) fn new_empty() -> Self {
        Tree {
            data: vec![],
            max_depth: 0,
        }
    }

    pub(crate) fn new_root(data: T) -> Self {
        Tree {
            data: vec![Node::new(data, 1, 0)],
            max_depth: 1,
        }
    }

    pub(crate) fn get_node(&self, index: usize) -> Option<&Node<T>> {
        self.data.get(index)
    }

    pub(crate) fn get_node_mut(&mut self, index: usize) -> Option<&mut Node<T>> {
        self.data.get_mut(index)
    }

    pub(crate) fn len(&self) -> usize {
        self.data.len()
    }

    /// Add a child to the parent node, if the parent node already has a child on the side, return an error.
    pub(crate) fn add_child(
        &mut self,
        parent_index: usize,
        side: ChildSide,
        data: T,
    ) -> Result<usize, ()> {
        let index = self.len();
        // if the tree is empty, add the root node
        if self.len() == 0 {
            self.data.push(Node::new(data, 1, 0));
            self.max_depth = 1;
            return Ok(index);
        }

        match self.data[parent_index][side] {
            Some(_) => Err(()),
            None => {
                let depth = self.data[parent_index].depth + 1;
                if depth > self.max_depth {
                    self.max_depth = depth;
                }
                self.data[parent_index][side] = Some(index);
                self.data.push(Node::new(data, depth, parent_index));

                Ok(index)
            }
        }
    }

    // TODO test need
    pub(crate) fn merge(self, other: Self, root: T) -> Self {
        let mut vec = Vec::with_capacity(1 + self.len() + other.len());
        vec.push(Node::new(root, 1, 0));
        let max_depth = self.max_depth.max(other.max_depth) + 1;
        let bias = self.len() + 1;
        vec.extend(self.data.into_iter().map(|node| Node {
            data: node.data,
            depth: node.depth + 1,
            parent: node.parent + 1,
            left: node.left.map(|x| x + 1),
            right: node.right.map(|x| x + 1),
        }));
        vec.extend(other.data.into_iter().map(|node| Node {
            data: node.data,
            depth: node.depth + 1,
            parent: node.parent + bias,
            left: node.left.map(|x| x + bias),
            right: node.right.map(|x| x + bias),
        }));
        vec[1].parent = 0;
        vec[bias].parent = 0;

        Tree {
            data: vec,
            max_depth,
        }
    }

    pub fn iter<IT: iter_types::IterType>(&self, root: usize) -> TreeIterator<T, IT> {
        TreeIterator::new(self, root)
    }
}

#[cfg(test)]
mod tree_test {
    use super::{iter_types, ChildSide, Tree};

    #[test]
    fn test_tree_new_root() {
        let tree = Tree::new_root(1.0);
        assert_eq!(tree.data[0].data, 1.0);
    }

    #[test]
    fn test_tree_add_chlid() {
        let mut tree = Tree::new_root(1.0);
        let node1 = tree.add_child(0, ChildSide::Left, 1.0);
        let node2 = tree.add_child(0, ChildSide::Left, 2.0);
        let node3 = tree.add_child(0, ChildSide::Right, 3.0);
        let node4 = tree.add_child(node3.unwrap(), ChildSide::Left, 4.0);
        assert_eq!(node1, Ok(1));
        assert_eq!(node2, Err(()));
        assert_eq!(node3, Ok(2));
        assert_eq!(tree.data.len(), 4);
        assert_eq!(tree.data[0].left, Some(1));
        assert_eq!(tree.data[0].right, Some(2));
        assert_eq!(tree.data[1].parent, 0);
        assert_eq!(tree.data[2].parent, 0);
        assert_eq!(tree.data[1].data, 1.0);
        assert_eq!(tree.data[2].data, 3.0);
        assert_eq!(node4, Ok(3));
        assert_eq!(tree.data[3].parent, 2);
    }

    #[test]
    fn test_tree_iter() {
        let mut tree = Tree::new_root(3);
        let node1 = tree.add_child(0, ChildSide::Left, 9);
        tree.add_child(0, ChildSide::Right, 20).unwrap();
        tree.add_child(node1.unwrap(), ChildSide::Left, 15).unwrap();
        tree.add_child(node1.unwrap(), ChildSide::Right, 7).unwrap();
        let iter = tree.iter::<iter_types::Dfs>(0);
        assert_eq!(iter.stack, vec![0]);
    }
}

pub mod iter_types {
    pub trait IterType {}

    pub struct PushOrder;
    // pub struct PreOrder;
    // pub struct InOrder;
    // pub struct PostOrder;
    pub struct Dfs;
    pub struct Bfs;

    impl IterType for PushOrder {}
    // impl IterType for PreOrder {}
    // impl IterType for InOrder {}
    // impl IterType for PostOrder {}
    impl IterType for Dfs {}
    impl IterType for Bfs {}
}

pub struct TreeIterator<'a, T, IT: iter_types::IterType> {
    tree: &'a Tree<T>,
    stack: VecDeque<usize>,
    iter_type: PhantomData<IT>,
}

impl<'a, T, IT: iter_types::IterType> TreeIterator<'a, T, IT> {
    fn new(tree: &'a Tree<T>, root_index: usize) -> Self {
        TreeIterator {
            tree,
            stack: vec![root_index].into(),
            iter_type: PhantomData,
        }
    }
}

impl<'a, T> Iterator for TreeIterator<'a, T, iter_types::PushOrder> {
    type Item = (&'a Node<T>, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.stack[0];
        let a = self.tree.data.get(index)?;
        self.stack[0] += 1;
        Some((a, index))
    }
}

impl<'a, T> Iterator for TreeIterator<'a, T, iter_types::Dfs> {
    type Item = (&'a Node<T>, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() {
            return None;
        }
        let index = self.stack.pop_back().unwrap();
        let node = &self.tree.data[index];
        if node.right.is_some() {
            self.stack.push_back(node.right.unwrap());
        }
        if node.left.is_some() {
            self.stack.push_back(node.left.unwrap());
        }
        Some((node, index))
    }
}

impl<'a, T> Iterator for TreeIterator<'a, T, iter_types::Bfs> {
    type Item = (&'a Node<T>, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() {
            return None;
        }
        let index = self.stack.pop_front().unwrap();
        let node = &self.tree.data[index];
        if node.left.is_some() {
            self.stack.push_back(node.left.unwrap());
        }
        if node.right.is_some() {
            self.stack.push_back(node.right.unwrap());
        }
        Some((node, index))
    }
}

#[cfg(test)]
mod test_tree_iter {
    use super::{iter_types, ChildSide, Tree};
    fn construct_tree() -> Tree<f32> {
        let mut tree = Tree::new_root(3.0);
        let node1 = tree.add_child(0, ChildSide::Left, 9.0);
        tree.add_child(0, ChildSide::Right, 20.0).unwrap();
        tree.add_child(node1.unwrap(), ChildSide::Left, 15.0)
            .unwrap();
        tree.add_child(node1.unwrap(), ChildSide::Right, 7.0)
            .unwrap();
        tree
    }

    #[test]
    fn test_tree_max_depth_test() {
        let tree = construct_tree();
        assert_eq!(tree.max_depth, 3);
    }

    #[test]
    fn test_tree_pushorder_iter() {
        let tree = construct_tree();
        let vec = tree
            .iter::<iter_types::PushOrder>(0)
            .map(|(n, _)| n.data)
            .collect::<Vec<_>>();
        assert_eq!(vec, vec![3.0, 9.0, 20.0, 15.0, 7.0]);
    }

    #[test]
    fn test_tree_dfs_iter() {
        let tree = construct_tree();
        let vec = tree
            .iter::<iter_types::Dfs>(0)
            .map(|(n, _)| n.data)
            .collect::<Vec<_>>();
        assert_eq!(vec, vec![3.0, 9.0, 15.0, 7.0, 20.0]);
    }

    #[test]
    fn test_tree_bfs_iter() {
        let tree = construct_tree();
        let vec = tree
            .iter::<iter_types::Bfs>(0)
            .map(|(n, _)| n.data)
            .collect::<Vec<_>>();
        assert_eq!(vec, vec![3.0, 9.0, 20.0, 15.0, 7.0]);
    }
}
