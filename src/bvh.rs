use std::collections::VecDeque;

use crate::{
    aabb::AABB,
    traits::{Bounded, Intersect},
    tree::{iter_types, ChildSide, Node, Tree, TreeIterator},
};

#[derive(Default)]
pub struct BuildBvhOption {
    pub depth_control: DepthControl,
    pub split_method: SplitMethod,
}

#[derive(Clone, Copy)]
pub enum DepthControl {
    MaxDepth(usize),
    MinPrimitives(usize),
}

impl Default for DepthControl {
    fn default() -> Self {
        DepthControl::MaxDepth(20)
    }
}

#[derive(Clone, Copy, Default)]
pub enum SplitMethod {
    #[default]
    Mid,
    Average,
    // SAH,
}

#[derive(Debug)]
pub struct BVHNodeData<const D: usize, P> {
    aabb: AABB<D>,
    primitives: Option<Vec<P>>,
}

impl<const D: usize, P> BVHNodeData<D, P> {
    fn new_node_data(aabb: AABB<D>) -> Self {
        BVHNodeData {
            aabb,
            primitives: None,
        }
    }

    fn new_leaf_data(aabb: AABB<D>, primitives: Vec<P>) -> Self {
        BVHNodeData {
            aabb,
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

pub struct BVHNode<'a, const D: usize, P> {
    pub parent: usize,
    pub depth: usize,
    pub left: Option<usize>,
    pub right: Option<usize>,
    pub aabb: AABB<D>,
    pub primitives: Option<&'a [P]>,
}

impl<'a, const D: usize, P> BVHNode<'a, D, P> {
    fn from_node(node: &'a Node<BVHNodeData<D, P>>) -> Self {
        BVHNode {
            parent: node.parent,
            depth: node.depth,
            left: node.left,
            right: node.right,
            aabb: node.data.aabb,
            primitives: node.data.primitives.as_ref().map(|v| v.as_slice()),
        }
    }
}

#[cfg(test)]
mod test_bvhnode {
    use super::*;

    #[test]
    fn test_from_node() {
        let node = Node::<BVHNodeData<3, usize>> {
            depth: 0,
            parent: 0,
            left: None,
            right: None,
            data: BVHNodeData::<3, usize> {
                aabb: AABB::new(),
                primitives: Some(vec![0, 1, 2]),
            },
        };

        let bvh_node = BVHNode::from_node(&node);
        assert_eq!(bvh_node.parent, 0);
        assert_eq!(bvh_node.depth, 0);
        assert_eq!(bvh_node.left, None);
        assert_eq!(bvh_node.right, None);
        assert_eq!(bvh_node.aabb.max, node.data.aabb.max);
        assert_eq!(bvh_node.aabb.min, node.data.aabb.min);
        assert_eq!(bvh_node.primitives, Some(&[0, 1, 2][..]));
    }
}

pub struct BVHIter<'a, const D: usize, P, IT: iter_types::IterType> {
    tree_iter: TreeIterator<'a, BVHNodeData<D, P>, IT>,
}

impl<'a, const D: usize, P> Iterator for BVHIter<'a, D, P, iter_types::PushOrder> {
    type Item = (BVHNode<'a, D, P>, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let (node, index) = self.tree_iter.next()?;
        Some((BVHNode::from_node(node), index))
    }
}

impl<'a, const D: usize, P> Iterator for BVHIter<'a, D, P, iter_types::Bfs> {
    type Item = (BVHNode<'a, D, P>, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let (node, index) = self.tree_iter.next()?;
        Some((BVHNode::from_node(node), index))
    }
}

impl<'a, const D: usize, P> Iterator for BVHIter<'a, D, P, iter_types::Dfs> {
    type Item = (BVHNode<'a, D, P>, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let (node, index) = self.tree_iter.next()?;
        Some((BVHNode::from_node(node), index))
    }
}

#[derive(Debug)]
pub struct BVH<const D: usize, P> {
    tree: Tree<BVHNodeData<D, P>>,
}

impl<const D: usize, P> BVH<D, P> {
    pub fn max_depth(&self) -> usize {
        self.tree.max_depth
    }

    pub fn get_root<'a>(&'a self) -> BVHNode<'a, D, P> {
        self.get_node(0).unwrap()
    }

    pub fn get_node<'a>(&'a self, index: usize) -> Option<BVHNode<'a, D, P>> {
        let node = self.tree.get_node(index)?;
        Some(BVHNode::from_node(node))
    }

    pub fn iter_rand<'a>(&'a self, from: usize) -> BVHIter<'a, D, P, iter_types::PushOrder> {
        BVHIter {
            tree_iter: self.tree.iter::<iter_types::PushOrder>(from),
        }
    }

    pub fn iter_bfs<'a>(&'a self, from: usize) -> BVHIter<'a, D, P, iter_types::Bfs> {
        BVHIter {
            tree_iter: self.tree.iter::<iter_types::Bfs>(from),
        }
    }

    pub fn iter_dfs<'a>(&'a self, from: usize) -> BVHIter<'a, D, P, iter_types::Dfs> {
        BVHIter {
            tree_iter: self.tree.iter::<iter_types::Dfs>(from),
        }
    }
}

impl<const D: usize, P> BVH<D, P> {
    pub(crate) fn transfrom_by<T, F>(self, f: F) -> BVH<D, T>
    where
        F: Fn(P) -> T,
    {
        let max_depth = self.tree.max_depth;
        let data = self
            .tree
            .data
            .into_iter()
            .map(|node| Node::<BVHNodeData<D, T>> {
                depth: node.depth,
                data: BVHNodeData::<D, T> {
                    aabb: node.data.aabb,
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
        BVH {
            tree: Tree { data, max_depth },
        }
    }

    pub(crate) fn transfrom<T: From<P>>(self) -> BVH<D, T> {
        self.transfrom_by(|p| T::from(p))
    }
}

impl<const D: usize, P> BVH<D, P>
where
    P: Bounded<D>,
{
    pub(crate) fn build(option: BuildBvhOption, triangles: Vec<P>) -> Self {
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
        let helper_aabb = triangles.iter().fold(AABB::new(), |mut aabb, v| {
            aabb.grow(&v.center());
            aabb
        });
        let aabb = triangles.iter().fold(AABB::new(), |mut aabb, v| {
            aabb.grow_from_aabb(&v.aabb());
            aabb
        });
        queue.push_back((
            helper_aabb,
            aabb,
            triangles,
            0usize,
            ChildSide::Left,
            0usize,
        ));
        loop {
            if queue.is_empty() {
                break;
            }
            let (helper_aabb, aabb, primitives, depth, side, parent_index) =
                queue.pop_front().unwrap();
            if should_stop(depth, primitives.len()) {
                let leaf_data =
                    BVHNodeData::new_leaf_data(aabb, primitives.into_iter().collect::<Vec<_>>());
                tree.add_child(parent_index, side, leaf_data).unwrap();
                continue;
            }
            let node_data = BVHNodeData::new_node_data(aabb);
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
            let left_aabb = left.iter().fold(AABB::new(), |mut aabb, v| {
                aabb.grow_from_aabb(&v.aabb());
                aabb
            });
            let right_aabb = right.iter().fold(AABB::new(), |mut aabb, v| {
                aabb.grow_from_aabb(&v.aabb());
                aabb
            });
            queue.push_back((
                helper_left_aabb,
                left_aabb,
                left,
                depth + 1,
                ChildSide::Left,
                current_index,
            ));
            queue.push_back((
                helper_right_aabb,
                right_aabb,
                right,
                depth + 1,
                ChildSide::Right,
                current_index,
            ));
        }
        BVH { tree }
    }

    fn split_triangles(
        aabb: AABB<D>,
        primitives: Vec<P>,
        split_axis: usize,
        split_method: SplitMethod,
    ) -> (Vec<P>, Vec<P>) {
        match split_method {
            SplitMethod::Mid => Self::split_triangles_mid(aabb, primitives, split_axis),
            SplitMethod::Average => Self::split_triangles_average(primitives, split_axis),
            // SplitMethod::SAH => Self::split_triangles_sah(),
        }
    }

    fn split_triangles_mid(
        aabb: AABB<D>,
        primitives: Vec<P>,
        split_axis: usize,
    ) -> (Vec<P>, Vec<P>) {
        let mid = aabb.center()[split_axis];
        primitives
            .into_iter()
            .partition::<Vec<_>, _>(|v| v.center_at_axis(split_axis) < mid)
    }

    fn split_triangles_average(mut primitives: Vec<P>, split_axis: usize) -> (Vec<P>, Vec<P>) {
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

impl<const D: usize, P> BVH<D, P> {
    pub fn intersect_by<F1, F2, I>(&self, intersecter: I, fi: F1, faabb: F2) -> Vec<&P>
    where
        F1: Fn(&I, &P) -> bool,
        F2: Fn(&I, &AABB<D>) -> bool,
    {
        let mut queue = VecDeque::new();
        let mut res = Vec::new();
        queue.push_back(self.tree.get_node(0).unwrap());
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
                if faabb(&intersecter, &node.data.aabb) {
                    queue.push_back(node);
                }
            }

            if let Some(right) = node.right {
                let node = self.tree.get_node(right).unwrap();
                if faabb(&intersecter, &node.data.aabb) {
                    queue.push_back(node);
                }
            }
        }
        res
    }

    // return all intersected primitives
    pub fn intersect<I: Intersect<AABB<D>> + Intersect<P>>(&self, intersecter: I) -> Vec<&P> {
        self.intersect_by(
            intersecter,
            |i, p| i.intersect(p),
            |i, aabb| i.intersect(aabb),
        )
    }
}

#[cfg(test)]
mod test_bvh {
    use super::*;
    use glam::Vec2;
    use glam::Vec3;
    use rand::{distributions::Uniform, prelude::Distribution};

    #[test]
    fn test_split_triangles_mid() {
        let mut aabb = AABB::new();
        let primitives = vec![
            (
                0,
                [
                    Vec3::new(0.0, 1.0, 0.0),
                    Vec3::new(0.0, 1.0, 1.0),
                    Vec3::new(0.0, 0.0, 1.0),
                ],
            ),
            (
                1,
                [
                    Vec3::new(1.0, 1.0, 0.0),
                    Vec3::new(1.0, 1.0, 1.0),
                    Vec3::new(1.0, 0.0, 1.0),
                ],
            ),
            (
                2,
                [
                    Vec3::new(2.0, 1.0, 0.0),
                    Vec3::new(2.0, 1.0, 1.0),
                    Vec3::new(2.0, 0.0, 1.0),
                ],
            ),
            (
                3,
                [
                    Vec3::new(3.0, 1.0, 0.0),
                    Vec3::new(3.0, 1.0, 1.0),
                    Vec3::new(3.0, 0.0, 1.0),
                ],
            ),
        ];
        for (_, g) in primitives.iter() {
            aabb.grow_from_aabb(&g.aabb());
        }

        let (v1, v2) = BVH::split_triangles_mid(aabb, primitives, 0);
        assert_eq!(v1.len(), 2);
        assert_eq!(v2.len(), 2);
        assert!(v1.iter().all(|(_, v)| v[0].x < 1.5));
        assert!(v2.iter().all(|(_, v)| v[0].x >= 1.5));

        for axis in 0..3 {
            let mut aabb = AABB::new();
            let mut primitives = vec![];
            let mut rng = rand::thread_rng();
            let low_u = Uniform::new(-10., 0.);
            let high_u = Uniform::new(0., 10.);
            for i in 0..1000 {
                let p = (
                    i,
                    if i < 500 {
                        [
                            Vec3::new(
                                low_u.sample(&mut rng),
                                low_u.sample(&mut rng),
                                low_u.sample(&mut rng),
                            ),
                            Vec3::new(
                                low_u.sample(&mut rng),
                                low_u.sample(&mut rng),
                                low_u.sample(&mut rng),
                            ),
                            Vec3::new(
                                low_u.sample(&mut rng),
                                low_u.sample(&mut rng),
                                low_u.sample(&mut rng),
                            ),
                        ]
                    } else {
                        [
                            Vec3::new(
                                high_u.sample(&mut rng),
                                high_u.sample(&mut rng),
                                high_u.sample(&mut rng),
                            ),
                            Vec3::new(
                                high_u.sample(&mut rng),
                                high_u.sample(&mut rng),
                                high_u.sample(&mut rng),
                            ),
                            Vec3::new(
                                high_u.sample(&mut rng),
                                high_u.sample(&mut rng),
                                high_u.sample(&mut rng),
                            ),
                        ]
                    },
                );
                aabb.grow_from_aabb(&p.1.aabb());
                primitives.push(p);
            }
            let (v1, v2) = BVH::split_triangles_mid(aabb, primitives, axis);
            assert_eq!(v1.len(), 500);
            assert_eq!(v2.len(), 500);
        }
    }

    #[test]
    fn test_split_triangles_average() {
        for axis in 0..3 {
            let mut aabb = AABB::<3>::new();
            let mut primitives = vec![];
            let mut rng = rand::thread_rng();
            let dist = Uniform::new(-10., 10.);
            for _ in 0..1000 {
                let p = (
                    (),
                    [
                        Vec3::new(
                            dist.sample(&mut rng),
                            dist.sample(&mut rng),
                            dist.sample(&mut rng),
                        ),
                        Vec3::new(
                            dist.sample(&mut rng),
                            dist.sample(&mut rng),
                            dist.sample(&mut rng),
                        ),
                        Vec3::new(
                            dist.sample(&mut rng),
                            dist.sample(&mut rng),
                            dist.sample(&mut rng),
                        ),
                    ],
                );
                aabb.grow_from_aabb(&p.1.aabb());
                primitives.push(p);
            }
            let (v1, v2) = BVH::split_triangles_average(primitives, axis);
            assert_eq!(v1.len(), 500);
            assert_eq!(v2.len(), 500);
        }
    }

    #[test]
    fn test_build_bvh() {
        let triangles = vec![
            (
                1,
                [
                    Vec2::new(2.0, 1.0),
                    Vec2::new(1.0, 2.0),
                    Vec2::new(1.0, 1.0),
                ],
            ),
            (
                2,
                [
                    Vec2::new(-2.0, 1.0),
                    Vec2::new(-1.0, 2.0),
                    Vec2::new(-1.0, 1.0),
                ],
            ),
            (
                4,
                [
                    Vec2::new(2.0, -1.0),
                    Vec2::new(1.0, -2.0),
                    Vec2::new(1.0, -1.0),
                ],
            ),
            (
                3,
                [
                    Vec2::new(-2.0, -1.0),
                    Vec2::new(-1.0, -2.0),
                    Vec2::new(-1.0, -1.0),
                ],
            ),
        ];
        let _bvh = BVH::build(BuildBvhOption::default(), triangles.clone());
        let _bvh = BVH::build(
            BuildBvhOption {
                split_method: SplitMethod::Average,
                ..Default::default()
            },
            triangles.clone(),
        );
        // dont panic
    }

    impl Intersect<AABB<2>> for Vec2 {
        fn intersect(&self, p: &AABB<2>) -> bool {
            for i in 0..2 {
                if self[i] < p.min[i] || self[i] > p.max[i] {
                    return false;
                }
            }
            true
        }
    }

    impl Intersect<(i32, [Vec2; 3])> for Vec2 {
        fn intersect(&self, p: &(i32, [Vec2; 3])) -> bool {
            <Vec2 as Intersect<[Vec2; 3]>>::intersect(self, &p.1)
        }
    }

    #[test]
    fn test_intersect() {
        let triangles = vec![
            (
                1,
                [
                    Vec2::new(2.0, 1.0),
                    Vec2::new(1.0, 2.0),
                    Vec2::new(1.0, 1.0),
                ],
            ),
            (
                2,
                [
                    Vec2::new(-2.0, 1.0),
                    Vec2::new(-1.0, 2.0),
                    Vec2::new(-1.0, 1.0),
                ],
            ),
            (
                3,
                [
                    Vec2::new(-2.0, -1.0),
                    Vec2::new(-1.0, -2.0),
                    Vec2::new(-1.0, -1.0),
                ],
            ),
            (
                4,
                [
                    Vec2::new(2.0, -1.0),
                    Vec2::new(1.0, -2.0),
                    Vec2::new(1.0, -1.0),
                ],
            ),
        ];

        let bvh = BVH::build(BuildBvhOption::default(), triangles.clone());
        let res = bvh.intersect(Vec2::new(0.0, 0.0));
        assert!(res.is_empty());
        let res = bvh.intersect(Vec2::new(1.1, 1.1));
        assert_eq!(res[0].0, 1);

        let bvh = bvh.transfrom_by(|(i, _)| i);
        println!("{:?}", bvh);
    }
}
