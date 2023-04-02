use glam::{Vec3, Vec2};

use crate::AABB;

use super::{Bounded, Intersect};

#[test]
fn test_bounded() {
    let triangle = [Vec3::X, Vec3::Y, Vec3::Z];
    let aabb: AABB<3> = triangle.bv();
    assert_eq!(aabb.min, [0.; 3]);
    assert_eq!(aabb.max, [1.; 3]);
    let center: [f32; 3] = <[Vec3; 3] as Bounded<3, AABB<3>>>::center(&triangle);
    assert_eq!(center, [1. / 3.; 3]);
    let center_at_x = <[Vec3; 3] as Bounded<3, AABB<3>>>::center_at_axis(&triangle, 0);
    assert_eq!(center_at_x, 1. / 3.);
}

#[test]
fn test_intersect_vec2_with_aabb() {
    let point = Vec2::new(0.1, 0.1);
    let triangle = [Vec2::X, Vec2::Y, Vec2::ZERO];
    assert!(point.intersect(&triangle));

    let point = Vec2::new(0.1, 0.1);
    let triangle = [Vec2::X, Vec2::Y, Vec2::X];
    assert!(!point.intersect(&triangle));

    let point = Vec2::new(1., 1.);
    let triangle = [Vec2::X, Vec2::Y, Vec2::ZERO];
    assert!(!point.intersect(&triangle));

    let aabb = AABB::<2> {
        min: [0., 0.],
        max: [1., 1.],
    };

    let aabb2 = AABB::<2> {
        min: [0., 0.],
        max: [2., 2.],
    };
    assert!(aabb.intersect(&Vec2::new(1., 1.)));
    assert!(!aabb.intersect(&Vec2::new(1.1, 1.)));
    assert!(aabb.intersect(&aabb2));
}
