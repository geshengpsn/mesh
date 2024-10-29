#[cfg(test)]
mod tests;

pub mod build_options;

use std::collections::VecDeque;

use crate::{
    bv::{BoundingVolume, AABB},
    traits::{Bounded, Intersect},
    tree::{iter_types, ChildSide, Node, Tree, TreeIterator},
};

use self::build_options::{BuildBvhOption, DepthControl, SplitMethod};

#[derive(Debug)]
pub struct BvhNodeData<const D: usize, BV, P>
where
    BV: BoundingVolume<D>,
{
    bv: BV,
    primitives: Option<Vec<P>>,
}

impl<const D: usize, BV: BoundingVolume<D>, P> BvhNodeData<D, BV, P> {
    fn new_node_data(bv: BV) -> Self {
        BvhNodeData {
            bv,
            primitives: None,
        }
    }

    fn new_leaf_data(bv: BV, primitives: Vec<P>) -> Self {
        BvhNodeData {
            bv,
            primitives: Some(primitives),
        }
    }

    fn is_node(&self) -> bool {
        self.primitives.is_none()
    }

    fn is_leaf(&self) -> bool {
        !self.is_node()
    }
}

pub struct BvhNode<'a, const D: usize, BV: BoundingVolume<D>, P> {
    pub parent: usize,
    pub depth: usize,
    pub left: Option<usize>,
    pub right: Option<usize>,
    pub bv: BV,
    pub primitives: Option<&'a [P]>,
}

impl<'a, const D: usize, BV: BoundingVolume<D>, P> BvhNode<'a, D, BV, P> {
    pub fn is_leaf(&self) -> bool {
        self.primitives.is_some()
    }

    pub fn is_node(&self) -> bool {
        !self.is_leaf()
    }

    fn from_node(node: &'a Node<BvhNodeData<D, BV, P>>) -> Self {
        BvhNode {
            parent: node.parent,
            depth: node.depth,
            left: node.left,
            right: node.right,
            bv: node.data.bv,
            primitives: node.data.primitives.as_deref(),
        }
    }
}

pub struct BvhIter<'a, const D: usize, BV: BoundingVolume<D>, P, IT: iter_types::IterType> {
    tree_iter: TreeIterator<'a, BvhNodeData<D, BV, P>, IT>,
}

impl<'a, const D: usize, BV: BoundingVolume<D>, P> Iterator
    for BvhIter<'a, D, BV, P, iter_types::PushOrder>
{
    type Item = (BvhNode<'a, D, BV, P>, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let (node, index) = self.tree_iter.next()?;
        Some((BvhNode::from_node(node), index))
    }
}

impl<'a, const D: usize, BV: BoundingVolume<D>, P> Iterator
    for BvhIter<'a, D, BV, P, iter_types::Bfs>
{
    type Item = (BvhNode<'a, D, BV, P>, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let (node, index) = self.tree_iter.next()?;
        Some((BvhNode::from_node(node), index))
    }
}

impl<'a, const D: usize, BV: BoundingVolume<D>, P> Iterator
    for BvhIter<'a, D, BV, P, iter_types::Dfs>
{
    type Item = (BvhNode<'a, D, BV, P>, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let (node, index) = self.tree_iter.next()?;
        Some((BvhNode::from_node(node), index))
    }
}

#[derive(Debug)]
pub struct Bvh<const D: usize, BV: BoundingVolume<D>, P> {
    tree: Tree<BvhNodeData<D, BV, P>>,
}

impl<const D: usize, BV: BoundingVolume<D>, P> Bvh<D, BV, P> {
    pub fn max_depth(&self) -> usize {
        self.tree.max_depth
    }

    pub fn get_root<'a>(&'a self) -> BvhNode<'a, D, BV, P> {
        self.get_node(0).unwrap()
    }

    pub fn get_node<'a>(&'a self, index: usize) -> Option<BvhNode<'a, D, BV, P>> {
        let node = self.tree.get_node(index)?;
        Some(BvhNode::from_node(node))
    }

    pub fn iter_rand<'a>(&'a self, from: usize) -> BvhIter<'a, D, BV, P, iter_types::PushOrder> {
        BvhIter {
            tree_iter: self.tree.iter::<iter_types::PushOrder>(from),
        }
    }

    pub fn iter_bfs<'a>(&'a self, from: usize) -> BvhIter<'a, D, BV, P, iter_types::Bfs> {
        BvhIter {
            tree_iter: self.tree.iter::<iter_types::Bfs>(from),
        }
    }

    pub fn iter_dfs<'a>(&'a self, from: usize) -> BvhIter<'a, D, BV, P, iter_types::Dfs> {
        BvhIter {
            tree_iter: self.tree.iter::<iter_types::Dfs>(from),
        }
    }
}

impl<const D: usize, BV: BoundingVolume<D>, P> Bvh<D, BV, P> {
    pub(crate) fn transfrom_by<T, F>(self, f: F) -> Bvh<D, BV, T>
    where
        F: Fn(P) -> T,
    {
        let max_depth = self.tree.max_depth;
        let data = self
            .tree
            .data
            .into_iter()
            .map(|node| Node::<BvhNodeData<D, BV, T>> {
                depth: node.depth,
                data: BvhNodeData::<D, BV, T> {
                    bv: node.data.bv,
                    primitives: if node.data.primitives.is_some() {
                        Some(
                            node.data
                                .primitives
                                .unwrap()
                                .into_iter()
                                .map(|p| f(p))
                                .collect::<Vec<_>>(),
                        )
                    } else {
                        None
                    },
                },
                parent: node.parent,
                left: node.left,
                right: node.right,
            })
            .collect::<Vec<_>>();
        Bvh {
            tree: Tree { data, max_depth },
        }
    }

    pub(crate) fn transfrom<T: From<P>>(self) -> Bvh<D, BV, T> {
        self.transfrom_by(|p| T::from(p))
    }
}

impl<const D: usize, BV: BoundingVolume<D>, P> Bvh<D, BV, P> {
    pub(crate) fn build(option: BuildBvhOption, primitives: Vec<P>) -> Self
    where
        P: Bounded<D, BV>,
    {
        let should_stop = |depth: usize, prim: usize| {
            if prim <= 1 {
                return true;
            }
            match option.depth_control {
                DepthControl::MaxDepth(max_depth) => depth >= max_depth,
                DepthControl::MinPrimitives(min_prim) => prim <= min_prim,
            }
        };
        let mut tree = Tree::new_empty();
        let mut queue = VecDeque::new();
        let helper_aabb = primitives.iter().fold(AABB::new(), |mut aabb, v| {
            aabb.grow(&v.center());
            aabb
        });
        let bv = primitives.iter().fold(BV::default(), |mut bv, v| {
            bv.merge(&v.bv());
            bv
        });
        queue.push_back((helper_aabb, bv, primitives, 0usize, ChildSide::Left, 0usize));
        loop {
            if queue.is_empty() {
                break;
            }
            let (helper_aabb, aabb, primitives, depth, side, parent_index) =
                queue.pop_front().unwrap();
            if should_stop(depth, primitives.len()) {
                let leaf_data =
                    BvhNodeData::new_leaf_data(aabb, primitives.into_iter().collect::<Vec<_>>());
                tree.add_child(parent_index, side, leaf_data).unwrap();
                continue;
            }
            let node_data = BvhNodeData::new_node_data(aabb);
            let current_index = tree.add_child(parent_index, side, node_data).unwrap();
            let longest_axis = helper_aabb.longest_dimension();
            let (left, right) =
                Self::split_triangles(helper_aabb, primitives, longest_axis, option.split_method);
            let helper_left_aabb = left.iter().fold(AABB::new(), |mut aabb, v| {
                aabb.grow(&v.center());
                aabb
            });
            let helper_right_aabb = right.iter().fold(AABB::new(), |mut aabb, v| {
                aabb.grow(&v.center());
                aabb
            });
            let left_bv = left.iter().fold(BV::default(), |mut bv, v| {
                bv.merge(&v.bv());
                bv
            });
            let right_bv = right.iter().fold(BV::default(), |mut bv, v| {
                bv.merge(&v.bv());
                bv
            });
            queue.push_back((
                helper_left_aabb,
                left_bv,
                left,
                depth + 1,
                ChildSide::Left,
                current_index,
            ));
            queue.push_back((
                helper_right_aabb,
                right_bv,
                right,
                depth + 1,
                ChildSide::Right,
                current_index,
            ));
        }
        Bvh { tree }
    }

    fn split_triangles(
        aabb: AABB<D>,
        primitives: Vec<P>,
        split_axis: usize,
        split_method: SplitMethod,
    ) -> (Vec<P>, Vec<P>)
    where
        P: Bounded<D, BV>,
        BV: BoundingVolume<D>,
    {
        match split_method {
            SplitMethod::Mid => Self::split_triangles_mid(aabb, primitives, split_axis),
            SplitMethod::Average => Self::split_triangles_average(primitives, split_axis),
            // SplitMethod::SAH => split_triangles_sah(),
        }
    }

    fn split_triangles_mid(aabb: AABB<D>, primitives: Vec<P>, split_axis: usize) -> (Vec<P>, Vec<P>)
    where
        P: Bounded<D, BV>,
        BV: BoundingVolume<D>,
    {
        let mid = aabb.center()[split_axis];
        primitives
            .into_iter()
            .partition::<Vec<_>, _>(|v| v.center_at_axis(split_axis) < mid)
    }

    fn split_triangles_average(mut primitives: Vec<P>, split_axis: usize) -> (Vec<P>, Vec<P>)
    where
        P: Bounded<D, BV>,
        BV: BoundingVolume<D>,
    {
        let half_len = primitives.len() / 2;
        primitives.sort_by(|a, b| {
            let a_center = a.center_at_axis(split_axis);
            let b_center = b.center_at_axis(split_axis);
            a_center.partial_cmp(&b_center).unwrap()
        });
        let right = primitives.split_off(half_len);
        (primitives, right)
    }

    // TODO SAH split method support later
    // fn split_triangles_sah<'a, G: 'a + Iterator<Item = &'a [f32; D]>>() -> (Vec<(P, G)>, Vec<(P, G)>)
    // {
    //     todo!()
    // }
}

impl<const D: usize, BV: BoundingVolume<D>, P> Bvh<D, BV, P> {
    pub fn intersect_by<F1, F2, I>(&self, intersecter: I, fi: F1, fbv: F2) -> Vec<&P>
    where
        F1: Fn(&I, &P) -> bool,
        F2: Fn(&I, &BV) -> bool,
    {
        let mut queue = VecDeque::new();
        let mut res = Vec::new();
        let node = self.tree.get_node(0).unwrap();
        if !fbv(&intersecter, &node.data.bv) {
            return vec![];
        }
        queue.push_back(node);
        loop {
            if queue.is_empty() {
                break;
            }
            let node = queue.pop_front().unwrap();
            if node.data.is_leaf() {
                if let Some(p) = node
                    .data
                    .primitives
                    .as_ref()
                    .unwrap()
                    .iter()
                    .find(|&p| fi(&intersecter, p))
                {
                    res.push(p);
                }
            }
            if let Some(left) = node.left {
                let node = self.tree.get_node(left).unwrap();
                if fbv(&intersecter, &node.data.bv) {
                    queue.push_back(node);
                }
            }

            if let Some(right) = node.right {
                let node = self.tree.get_node(right).unwrap();
                if fbv(&intersecter, &node.data.bv) {
                    queue.push_back(node);
                }
            }
        }
        res
    }

    // return all intersected primitives
    pub fn intersect<I: Intersect<BV> + Intersect<P>>(&self, intersecter: I, err: f32) -> Vec<&P> {
        self.intersect_by(
            intersecter,
            |i, p| i.intersect(p, err),
            |i, bv| i.intersect(bv, err),
        )
    }
}
