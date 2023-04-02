use glam::{mat2, Vec2, Vec3};

use crate::bv::{BoundingVolume, AABB};

#[cfg(test)]
mod tests;

pub trait Bounded<const D: usize, BV> {
    fn bv(&self) -> BV;
    fn center(&self) -> [f32; D];
    fn center_at_axis(&self, axis: usize) -> f32;
}

impl<P, BV: BoundingVolume<3>> Bounded<3, BV> for (P, [Vec3; 3]) {
    fn bv(&self) -> BV {
        self.1.bv()
    }

    fn center(&self) -> [f32; 3] {
        <[Vec3; 3] as Bounded<3, BV>>::center(&self.1)
    }

    fn center_at_axis(&self, axis: usize) -> f32 {
        <[Vec3; 3] as Bounded<3, BV>>::center_at_axis(&self.1, axis)
    }
}

impl<P, BV: BoundingVolume<2>> Bounded<2, BV> for (P, [Vec2; 3]) {
    fn bv(&self) -> BV {
        self.1.bv()
    }

    fn center(&self) -> [f32; 2] {
        <[Vec2; 3] as Bounded<2, BV>>::center(&self.1)
    }

    fn center_at_axis(&self, axis: usize) -> f32 {
        <[Vec2; 3] as Bounded<2, BV>>::center_at_axis(&self.1, axis)
    }
}

impl<BV: BoundingVolume<2>> Bounded<2, BV> for [Vec2; 3] {
    fn bv(&self) -> BV {
        let mut bv = BV::default();
        bv.grow(&self[0].to_array());
        bv.grow(&self[1].to_array());
        bv.grow(&self[2].to_array());
        bv
    }

    fn center(&self) -> [f32; 2] {
        let res = (self[0] + self[1] + self[2]) / 3.;
        res.to_array()
    }

    fn center_at_axis(&self, axis: usize) -> f32 {
        self.iter().map(|v| v[axis]).sum::<f32>() / 3.0
    }
}

impl<BV: BoundingVolume<3>> Bounded<3, BV> for [Vec3; 3] {
    fn bv(&self) -> BV {
        let mut bv = BV::default();
        bv.grow(&self[0].to_array());
        bv.grow(&self[1].to_array());
        bv.grow(&self[2].to_array());
        bv
    }

    fn center(&self) -> [f32; 3] {
        let res = (self[0] + self[1] + self[2]) / 3.;
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

impl<const D:usize> Intersect<AABB<D>> for AABB<D> {
    fn intersect(&self, p: &Self) -> bool {
        self.intersect_aabb(p)
    }
}
