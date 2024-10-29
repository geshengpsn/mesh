use std::fs;

use super::*;
use crate::bv::Bsphere;
use crate::IndexMesh;
use glam::Vec2;
use glam::Vec3;
use rand::{distributions::Uniform, prelude::Distribution};

#[test]
fn test_from_node() {
    let node = Node::<BvhNodeData<3, AABB<3>, usize>> {
        depth: 0,
        parent: 0,
        left: None,
        right: None,
        data: BvhNodeData::<3, AABB<3>, usize> {
            bv: AABB::new(),
            primitives: Some(vec![0, 1, 2]),
        },
    };

    let bvh_node = BvhNode::from_node(&node);
    assert_eq!(bvh_node.parent, 0);
    assert_eq!(bvh_node.depth, 0);
    assert_eq!(bvh_node.left, None);
    assert_eq!(bvh_node.right, None);
    assert_eq!(bvh_node.bv.max, node.data.bv.max);
    assert_eq!(bvh_node.bv.min, node.data.bv.min);
    assert_eq!(bvh_node.primitives, Some(&[0, 1, 2][..]));
}

#[test]
fn test_bvh_iter_max_depth_get_roor() {
    let mesh = IndexMesh::from_stl(&mut fs::File::open("assets/bunny.stl").unwrap()).unwrap();
    let bvh = mesh.build_aabb_bvh(BuildBvhOption::default());
    bvh.iter_rand(0).for_each(|_| {});
    bvh.iter_bfs(0).for_each(|_| {});
    bvh.iter_dfs(0).for_each(|_| {});
    assert_eq!(bvh.max_depth(), 21);
    assert_eq!(bvh.get_root().bv, bvh.get_node(0).unwrap().bv);
}

#[test]
fn test_build_options() {
    let mesh = IndexMesh::from_stl(&mut fs::File::open("assets/bunny.stl").unwrap()).unwrap();
    let option = BuildBvhOption {
        depth_control: DepthControl::MinPrimitives(8),
        ..Default::default()
    };
    let bvh = mesh.build_aabb_bvh(option);
    assert!(bvh.max_depth() <= 20);
    bvh.iter_rand(0)
        .filter(|(a, _)| a.is_leaf())
        .for_each(|(n, _)| {
            assert!(n.primitives.unwrap().len() <= 8);
        });
}

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
        aabb.grow_from_aabb(&g.bv());
    }

    let (v1, v2) = Bvh::<3, AABB<3>, _>::split_triangles_mid(aabb, primitives, 0);
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
            aabb.grow_from_aabb(&p.1.bv());
            primitives.push(p);
        }
        let (v1, v2) = Bvh::<3, AABB<3>, _>::split_triangles_mid(aabb, primitives, axis);
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
            aabb.grow_from_aabb(&p.1.bv());
            primitives.push(p);
        }
        let (v1, v2) = Bvh::<3, AABB<3>, _>::split_triangles_average(primitives, axis);
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
    let _bvh = Bvh::<2, AABB<2>, _>::build(BuildBvhOption::default(), triangles.clone());
    let _bvh = Bvh::<2, AABB<2>, _>::build(
        BuildBvhOption {
            split_method: SplitMethod::Average,
            ..Default::default()
        },
        triangles.clone(),
    );

    let _bvh = Bvh::<2, Bsphere<2>, _>::build(BuildBvhOption::default(), triangles.clone());
}

impl Intersect<AABB<2>> for Vec2 {
    fn intersect(&self, p: &AABB<2>, err: f32) -> bool {
        for i in 0..2 {
            if self[i] < p.min[i] || self[i] > p.max[i] {
                return false;
            }
        }
        true
    }
}

impl Intersect<(i32, [Vec2; 3])> for Vec2 {
    fn intersect(&self, p: &(i32, [Vec2; 3]), err: f32) -> bool {
        <Vec2 as Intersect<[Vec2; 3]>>::intersect(self, &p.1, err)
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

    let bvh = Bvh::<2, AABB<2>, _>::build(BuildBvhOption::default(), triangles.clone());
    let res = bvh.intersect(Vec2::new(0.0, 0.0), 1e-7);
    assert!(res.is_empty());
    let res = bvh.intersect(Vec2::new(1.1, 1.1), 1e-7);
    assert_eq!(res[0].0, 1);

    // let bvh = bvh.transfrom_by(|(i, _)| i);
    println!("{:?}", bvh);
}

#[cfg(test)]
mod test_bvh {
    use crate::bv::Bsphere;

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
            aabb.grow_from_aabb(&g.bv());
        }

        let (v1, v2) = Bvh::<3, AABB<3>, _>::split_triangles_mid(aabb, primitives, 0);
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
                aabb.grow_from_aabb(&p.1.bv());
                primitives.push(p);
            }
            let (v1, v2) = Bvh::<3, AABB<3>, _>::split_triangles_mid(aabb, primitives, axis);
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
                aabb.grow_from_aabb(&p.1.bv());
                primitives.push(p);
            }
            let (v1, v2) = Bvh::<3, AABB<3>, _>::split_triangles_average(primitives, axis);
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
        let _bvh = Bvh::<2, AABB<2>, _>::build(BuildBvhOption::default(), triangles.clone());
        let _bvh = Bvh::<2, AABB<2>, _>::build(
            BuildBvhOption {
                split_method: SplitMethod::Average,
                ..Default::default()
            },
            triangles.clone(),
        );

        let _bvh = Bvh::<2, Bsphere<2>, _>::build(BuildBvhOption::default(), triangles.clone());
    }

    // impl Intersect<AABB<2>> for Vec2 {
    //     fn intersect(&self, p: &AABB<2>) -> bool {
    //         for i in 0..2 {
    //             if self[i] < p.min[i] || self[i] > p.max[i] {
    //                 return false;
    //             }
    //         }
    //         true
    //     }
    // }

    // impl Intersect<(i32, [Vec2; 3])> for Vec2 {
    //     fn intersect(&self, p: &(i32, [Vec2; 3])) -> bool {
    //         <Vec2 as Intersect<[Vec2; 3]>>::intersect(self, &p.1)
    //     }
    // }

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

        let bvh = Bvh::<2, AABB<2>, _>::build(BuildBvhOption::default(), triangles.clone());
        let res = bvh.intersect(Vec2::new(0.0, 0.0), 1e-7);
        assert!(res.is_empty());
        let res = bvh.intersect(Vec2::new(1.1, 1.1), 1e-7);
        assert_eq!(res[0].0, 1);

        // let Bvh = Bvh.transfrom_by(|(i, _)| i);
        println!("{:?}", bvh);
    }
}
