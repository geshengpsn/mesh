use glam::{mat2, Vec2, Vec3};

use crate::aabb::AABB;

pub(crate) trait Primitive<const D: usize>: Bounded<D> {}

pub trait Bounded<const D: usize> {
    fn aabb(&self) -> AABB<D>;
    fn center(&self) -> [f32; D];
    fn center_at_axis(&self, axis: usize) -> f32;
}

impl<P> Bounded<3> for (P, [Vec3; 3]) {
    fn aabb(&self) -> AABB {
        self.1.aabb()
    }

    fn center(&self) -> [f32; 3] {
        self.1.center()
    }

    fn center_at_axis(&self, axis: usize) -> f32 {
        self.1.center_at_axis(axis)
    }
}

impl Bounded<3> for [Vec3; 3] {
    fn aabb(&self) -> AABB {
        let mut aabb = AABB::new();
        for v in self.iter() {
            aabb.grow(&v.to_array());
        }
        aabb
    }

    fn center(&self) -> [f32; 3] {
        let res = (self[0] + self[1] + self[2]) / 3.;
        res.to_array()
    }

    fn center_at_axis(&self, axis: usize) -> f32 {
        self.iter().map(|v| v[axis]).sum::<f32>() / 3.0
    }
}

impl<P> Bounded<2> for (P, [Vec2; 3]) {
    fn aabb(&self) -> AABB<2> {
        self.1.aabb()
    }

    fn center(&self) -> [f32; 2] {
        self.1.center()
    }

    fn center_at_axis(&self, axis: usize) -> f32 {
        self.1.center_at_axis(axis)
    }
}

impl Bounded<2> for [Vec2; 3] {
    fn aabb(&self) -> AABB<2> {
        let mut aabb = AABB::new();
        for v in self.iter() {
            aabb.grow(&v.to_array());
        }
        aabb
    }

    fn center(&self) -> [f32; 2] {
        let res = (self[0] + self[1]) / 3.;
        res.to_array()
    }

    fn center_at_axis(&self, axis: usize) -> f32 {
        self.iter().map(|v| v[axis]).sum::<f32>() / 3.0
    }
}

pub trait Intersect<I> {
    fn intersect(&self, p: &I) -> bool;
}

impl Intersect<[Vec2; 3]> for Vec2 {
    fn intersect(&self, p: &[Vec2; 3]) -> bool {
        let triangle = p;
        let v0 = triangle[0];
        let v1 = triangle[1];
        let v2 = triangle[2];
        let e1 = v1 - v0;
        let e2 = v2 - v0;
        let m = mat2(e1, e2);
        if m.determinant() == 0.0 {
            return false;
        }
        let m_inv = m.inverse();
        let p = *self - v0;
        let res = m_inv.mul_vec2(p);
        if res[0] < 0.0 || res[0] > 1.0 || res[1] < 0.0 || res[1] > 1.0 {
            return false;
        }
        return true;
    }
}

impl Intersect<Vec2> for AABB<2> {
    fn intersect(&self, p: &Vec2) -> bool {
        self.min[0] <= p[0] && p[0] <= self.max[0] && self.min[1] <= p[1] && p[1] <= self.max[1]
    }
}
