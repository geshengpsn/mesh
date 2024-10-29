use std::ops::Add;

use glam::Vec3;

use crate::half_edge::HalfEdgeMesh;

use super::CsgSolid;

#[test]
fn test_union() {
    // TODO should crate a primtive lib for generial perpose
    let box1 = crate::Box { size: 1.0 }.to_mesh();
    let mut box2 = crate::Box { size: 1.0 }.to_mesh();
    box2.translate(Vec3::new(0.5, 0.5, 0.5));

    let csg_box1 = CsgSolid::new(box1.to_halfedge_mesh());
    let csg_box2 = CsgSolid::new(box2.to_halfedge_mesh());
    let new = csg_box1 & csg_box2;
    // let mut result = csg_box1.union(csg_box2);
    // result.construct();
}
