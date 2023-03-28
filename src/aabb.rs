#[derive(Debug, Clone, Copy)]
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

    pub(crate) fn intersect(&self, other: &Self) -> bool {
        for i in 0..D {
            if other.max[i] <= self.min[i] || self.max[i] <= other.min[i] {
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
        assert!(!a.intersect(&empty));
        assert!(!b.intersect(&empty));
        assert!(!c.intersect(&empty));
        assert!(a.intersect(&b));
        assert!(b.intersect(&c));
        assert!(!a.intersect(&c));
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
