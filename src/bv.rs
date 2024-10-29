use std::fmt::Debug;

pub trait BoundingVolume<const D: usize>: Default + Copy + PartialEq + Debug {
    fn merge(&mut self, other: &Self);
    fn grow(&mut self, point: &[f32; D]);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bsphere<const D: usize> {
    pub center: [f32; D],
    pub radius: f32,
}

impl<const D: usize> Default for Bsphere<D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const D: usize> BoundingVolume<D> for Bsphere<D> {
    fn merge(&mut self, other: &Self) {
        let distance = self.center_distance(other);
        let self_contain_other = distance + other.radius <= self.radius;
        let other_contain_self = distance + self.radius <= other.radius;
        if self_contain_other {
            return;
        }
        if other_contain_self {
            self.center = other.center;
            self.radius = other.radius;
            return;
        }

        let mut normal_vec = [0.; D];
        let mut center = [0.; D];
        for i in 0..D {
            normal_vec[i] = (self.center[i] - other.center[i]) / distance;
            let a = self.center[i] + normal_vec[i] * self.radius;
            let b = other.center[i] - normal_vec[i] * other.radius;
            center[i] = (a + b) / 2.;
        }
        let radius = (distance + self.radius + other.radius) / 2.;
        self.center = center.into();
        self.radius = radius;
    }

    fn grow(&mut self, point: &[f32; D]) {
        let distance = self
            .center
            .iter()
            .zip(point)
            .fold(0., |init, (a, b)| init + (a - b) * (a - b))
            .sqrt();
        if self.radius < 0. {
            self.center = point.clone();
            self.radius = 0.;
        }
        if distance > self.radius {
            let mut normal_vec = [0.; D];
            let mut center = [0.; D];
            for i in 0..D {
                normal_vec[i] = (self.center[i] - point[i]) / distance;
                let a = self.center[i] + normal_vec[i] * self.radius;
                center[i] = (a + point[i]) / 2.;
            }
            let radius = (distance + self.radius) / 2.;
            self.center = center.into();
            self.radius = radius;
        }
    }
}

impl<const D: usize> Bsphere<D> {
    fn new() -> Self {
        Bsphere {
            center: [0.; D],
            radius: f32::NEG_INFINITY,
        }
    }

    fn center_distance(&self, other: &Self) -> f32 {
        self.center
            .iter()
            .zip(other.center)
            .fold(0., |init, (a, b)| init + (a - b) * (a - b))
            .sqrt()
    }

    fn intersect(&self, other: &Self) -> bool {
        let distance = self.center_distance(other);
        distance <= self.radius + other.radius
    }

    fn contain(&self, other: &Self) -> bool {
        let distance = self.center_distance(other);
        distance + other.radius <= self.radius
    }
}

#[cfg(test)]
mod bsphere_test {
    use super::*;
    #[test]
    fn test_bsphere_new() {
        let new_sphere1 = Bsphere::<3>::new();
        let new_sphere2 = Bsphere::<3>::default();
        assert_eq!(new_sphere1.center, new_sphere2.center);
        assert_eq!(new_sphere1.radius, new_sphere2.radius);
    }

    #[test]
    fn test_bsphere_grow() {
        let mut new_sphere = Bsphere::<3>::new();
        new_sphere.grow(&[0.; 3]);
        assert_eq!(new_sphere.center, [0.; 3]);
        assert_eq!(new_sphere.radius, 0.);
        new_sphere.grow(&[1., 0., 0.]);
        assert_eq!(new_sphere.center, [0.5, 0., 0.]);
        assert_eq!(new_sphere.radius, 0.5);
    }

    #[test]
    fn test_center_distance() {
        let sphere1 = Bsphere::<3> {
            center: [0., 0., 0.],
            radius: 0.,
        };
        let sphere2 = Bsphere::<3> {
            center: [1., 1., 1.],
            radius: 0.,
        };
        assert_eq!(sphere1.center_distance(&sphere2), 3f32.sqrt());
    }

    #[test]
    fn test_intersect() {
        let sphere1 = Bsphere::<3> {
            center: [0., 0., 0.],
            radius: 1.,
        };
        let sphere2 = Bsphere::<3> {
            center: [1., 1., 1.],
            radius: 1.,
        };
        assert_eq!(sphere1.intersect(&sphere2), true);
        let sphere1 = Bsphere::<3> {
            center: [0., 0., 0.],
            radius: 1.,
        };
        let sphere2 = Bsphere::<3> {
            center: [2., 0., 0.],
            radius: 1.,
        };
        assert_eq!(sphere1.intersect(&sphere2), true);
        let sphere3 = Bsphere::<3> {
            center: [2., 2., 2.],
            radius: 1.,
        };
        assert_eq!(sphere1.intersect(&sphere3), false);
    }

    #[test]
    fn test_contain() {
        let sphere1 = Bsphere::<3> {
            center: [0., 0., 0.],
            radius: 1.,
        };
        let sphere2 = Bsphere::<3> {
            center: [0., 0., 0.],
            radius: 0.5,
        };
        assert_eq!(sphere1.contain(&sphere2), true);
        let sphere3 = Bsphere::<3> {
            center: [0., 0., 0.0001],
            radius: 1.,
        };
        assert_eq!(sphere1.contain(&sphere3), false);
    }

    #[test]
    fn test_merge() {
        let mut sphere1 = Bsphere::<3> {
            center: [0., 0., 0.],
            radius: 1.,
        };
        let sphere2 = Bsphere::<3> {
            center: [0., 0., 0.],
            radius: 0.5,
        };
        sphere1.merge(&sphere2);
        assert_eq!(sphere1.center, [0., 0., 0.]);
        assert_eq!(sphere1.radius, 1.);

        let sphere1 = Bsphere::<3> {
            center: [0., 0., 0.],
            radius: 1.,
        };
        let mut sphere2 = Bsphere::<3> {
            center: [0., 0., 0.],
            radius: 0.5,
        };
        sphere2.merge(&sphere1);
        assert_eq!(sphere2.center, [0., 0., 0.]);
        assert_eq!(sphere2.radius, 1.);

        let sphere1 = Bsphere::<3> {
            center: [0., 0., 0.],
            radius: 1.,
        };
        let mut sphere2 = Bsphere::<3> {
            center: [2., 0., 0.],
            radius: 1.,
        };
        sphere2.merge(&sphere1);
        assert_eq!(sphere2.center, [1., 0., 0.]);
        assert_eq!(sphere2.radius, 2.);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AABB<const D: usize = 3> {
    pub min: [f32; D],
    pub max: [f32; D],
}

impl<const D: usize> Default for AABB<D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const D: usize> AABB<D> {
    pub(crate) fn new() -> Self {
        AABB {
            min: [f32::MAX; D],
            max: [f32::MIN; D],
        }
    }

    pub(crate) fn from_triangle(triangle: &[[f32; D]; 3]) -> Self {
        let mut aabb = AABB::new();
        aabb.grow_from_triangle(triangle);
        aabb
    }

    pub(crate) fn grow_from_triangle(&mut self, triangle: &[[f32; D]; 3]) {
        for v in triangle {
            self.grow(v);
        }
    }

    pub(crate) fn grow_from_aabb(&mut self, aabb: &AABB<D>) {
        self.grow(&aabb.min);
        self.grow(&aabb.max);
    }

    pub(crate) fn grow(&mut self, point: &[f32; D]) {
        for i in 0..D {
            self.min[i] = self.min[i].min(point[i]);
            self.max[i] = self.max[i].max(point[i]);
        }
    }

    pub(crate) fn intersect_aabb(&self, other: &Self, err: f32) -> bool {
        for i in 0..D {
            if (other.max[i] - self.min[i]).abs() < err || (self.max[i] - other.min[i]).abs() < err
            {
                return false;
            }
        }
        true
    }

    pub(crate) fn center(&self) -> [f32; D] {
        let mut res = [0.; D];
        for i in 0..D {
            res[i] = (self.min[i] + self.max[i]) / 2.0;
        }
        res
    }

    pub(crate) fn longest_dimension(&self) -> usize {
        let mut max = 0.;
        let mut a = 0;
        for i in 0..D {
            let d = self.max[i] - self.min[i];
            if d > max {
                a = i;
                max = d;
            }
        }
        a
    }
}

impl<const D: usize> BoundingVolume<D> for AABB<D> {
    fn merge(&mut self, other: &Self) {
        self.grow_from_aabb(other);
    }

    fn grow(&mut self, point: &[f32; D]) {
        self.grow(point);
    }
}

#[cfg(test)]
mod aabb_test {
    use super::AABB;

    #[test]
    fn test_new() {
        let aabb = AABB::new();
        assert_eq!(aabb.min, [f32::MAX; 3]);
        assert_eq!(aabb.max, [f32::MIN; 3]);
    }

    #[test]
    fn test_default() {
        let aabb = AABB::default();
        assert_eq!(aabb.min, [f32::MAX; 3]);
        assert_eq!(aabb.max, [f32::MIN; 3]);
    }

    #[test]
    fn test_intersect() {
        // empty aabb should not intersect with any aabb
        let empty = AABB::new();
        let a = AABB {
            min: [-1.; 3],
            max: [1.; 3],
        };
        let b = AABB {
            min: [0.; 3],
            max: [2.; 3],
        };
        let c = AABB {
            min: [1.; 3],
            max: [3.; 3],
        };
        assert!(!a.intersect_aabb(&empty, 1e-7));
        assert!(!b.intersect_aabb(&empty, 1e-7));
        assert!(!c.intersect_aabb(&empty, 1e-7));
        assert!(a.intersect_aabb(&b, 1e-7));
        assert!(b.intersect_aabb(&c, 1e-7));
        assert!(!a.intersect_aabb(&c, 1e-7));
    }

    #[test]
    fn test_grow() {
        let mut aabb = AABB::new();
        aabb.grow(&[0., 0., 0.]);
        assert_eq!(aabb.min, [0.; 3]);
        assert_eq!(aabb.max, [0.; 3]);
        aabb.grow(&[1., 1., 1.]);
        assert_eq!(aabb.min, [0.; 3]);
        assert_eq!(aabb.max, [1.; 3]);
        aabb.grow(&[2., 3., 4.]);
        assert_eq!(aabb.min, [0.; 3]);
        assert_eq!(aabb.max, [2., 3., 4.]);
        aabb.grow(&[-2., 0., 0.]);
        assert_eq!(aabb.min, [-2., 0., 0.]);
        assert_eq!(aabb.max, [2., 3., 4.]);
    }

    #[test]
    fn test_from_triangle() {
        let a = AABB::from_triangle(&[[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]]);
        assert_eq!(a.min, [0.; 3]);
        assert_eq!(a.max, [1.; 3]);
    }

    #[test]
    fn test_longest_axis() {
        let a = AABB::<3>::from_triangle(&[[1.1, 0., 0.], [0., 1., 0.], [0., 0., 1.]]);
        assert_eq!(a.longest_dimension(), 0);
        let b = AABB::<3>::from_triangle(&[[0., 1.1, 0.], [0., 0., 1.], [1., 0., 0.]]);
        assert_eq!(b.longest_dimension(), 1);
        let c = AABB::<3>::from_triangle(&[[0., 0., 1.1], [1., 0., 0.], [0., 1., 0.]]);
        assert_eq!(c.longest_dimension(), 2);
    }

    #[test]
    fn test_center() {
        let a: AABB<3> = AABB::from_triangle(&[[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]]);
        assert_eq!(a.center(), [0.5, 0.5, 0.5]);
    }
}
