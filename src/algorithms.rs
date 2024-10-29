use glam::{DMat3, DVec2, DVec3, Vec3};

use crate::AABB;

#[cfg(test)]
mod algo_tests;

pub(crate) enum TriTriIntersectStatus {
    NotIntersect,
    Intersect(IPoint, Option<IPoint>),
    Coplanar(Vec<IPoint>),
}

pub(crate) struct IPoint {
    p: DVec3,
    t1: IntersectTopo,
    t2: IntersectTopo,
}

#[derive(Clone, Copy)]
pub(crate) enum IntersectTopo {
    Edge(u8, u8),
    Vertex(u8),
    Face,
}

// TODO reference: https://web.stanford.edu/class/cs277/resources/papers/Moller1997b.pdf
/// include aabb check, no need to check aabb in the caller
pub(crate) fn tri_tri_intersect(
    tri1: [DVec3; 3],
    tri2: [DVec3; 3],
    err: f64,
) -> TriTriIntersectStatus {
    use TriTriIntersectStatus::{Coplanar, NotIntersect};

    // AABB test
    let mut aabb1 = AABB::new();
    let mut aabb2 = AABB::new();

    for i in 0..3 {
        aabb1.grow(&tri1[i].as_vec3().to_array());
        aabb2.grow(&tri2[i].as_vec3().to_array());
    }

    if !aabb1.intersect_aabb(&aabb2, err as f32) {
        return NotIntersect;
    }

    // plane test
    let tri2_plane_normal = (tri2[0] - tri2[1]).cross(tri2[0] - tri2[2]).normalize();
    let d_tri1_0 = tri2_plane_normal.dot(tri1[0] - tri2[0]);
    let d_tri1_1 = tri2_plane_normal.dot(tri1[1] - tri2[0]);
    let d_tri1_2 = tri2_plane_normal.dot(tri1[2] - tri2[0]);

    let has_same_sign =
        (d_tri1_0.is_sign_negative() && d_tri1_1.is_sign_negative() && d_tri1_2.is_sign_negative())
            || (d_tri1_0.is_sign_positive()
                && d_tri1_1.is_sign_positive()
                && d_tri1_2.is_sign_positive());
    let on_plane_status = (
        d_tri1_0.abs() < err,
        d_tri1_1.abs() < err,
        d_tri1_2.abs() < err,
    );
    if has_same_sign {
        // if vertex on plane
        match on_plane_status {
            (true, true, true) => {
                // co-planer case
                todo!("co-planer case")
            }
            (true, true, false) => {
                // edge on plane
                return edge_on_plane(&tri1, &tri2, (0, 1));
            }
            (true, false, true) => {
                // edge on plane
                return edge_on_plane(&tri1, &tri2, (0, 2));
            }
            (true, false, false) => {
                // vertex on plane
                return vertex_on_plane(&tri2, tri1[0], 0);
            }
            (false, true, true) => {
                // edge on plane
                return edge_on_plane(&tri1, &tri2, (1, 2));
            }
            (false, true, false) => {
                // vertex on plane
                return vertex_on_plane(&tri2, tri1[1], 1);
            }
            (false, false, true) => {
                // vertex on plane
                return vertex_on_plane(&tri2, tri1[2], 2);
            }
            (false, false, false) => return NotIntersect,
        }
    }

    let alone_vertex = 0;
    let pair_vertex = (0, 0);

    // l_dir * t + O = l
    let tri1_plane_normal = (tri1[0] - tri1[1]).cross(tri1[0] - tri1[2]).normalize();
    let l_dir = tri1_plane_normal.cross(tri2_plane_normal);
    if d_tri1_0.is_sign_negative() {}
    todo!()
}

enum EdgeOnPlaneStatus {
    NotIntersect,
    IntersectVertex(u8),
    IntersectVertexEdge(u8, (u8, u8, DVec3)),
    IntersectEdgeEdge((u8, u8, DVec3), (u8, u8, DVec3)),
    IntersectVertexVertex(u8, u8),
}

impl EdgeOnPlaneStatus {
    fn to_status(
        &self,
        tri1: &[DVec3; 3],
        tri2: &[DVec3; 3],
        on_plane_vertex: (u8, u8),
    ) -> TriTriIntersectStatus {
        let t1 = IntersectTopo::Edge(on_plane_vertex.0, on_plane_vertex.1);
        match self {
            Self::NotIntersect => TriTriIntersectStatus::NotIntersect,
            Self::IntersectVertex(v) => TriTriIntersectStatus::Intersect(
                IPoint {
                    p: tri2[*v as usize],
                    t1,
                    t2: IntersectTopo::Vertex(*v),
                },
                None,
            ),
            Self::IntersectVertexEdge(v, e) => TriTriIntersectStatus::Intersect(
                IPoint {
                    p: tri2[*v as usize],
                    t1,
                    t2: IntersectTopo::Vertex(*v),
                },
                Some(IPoint {
                    p: e.2,
                    t1,
                    t2: IntersectTopo::Edge(e.0, e.1),
                }),
            ),
            Self::IntersectEdgeEdge(e1, e2) => TriTriIntersectStatus::Intersect(
                IPoint {
                    p: e1.2,
                    t1,
                    t2: IntersectTopo::Edge(e1.0, e1.1),
                },
                Some(IPoint {
                    p: e2.2,
                    t1,
                    t2: IntersectTopo::Edge(e2.0, e2.1),
                }),
            ),
            Self::IntersectVertexVertex(v1, v2) => TriTriIntersectStatus::Intersect(
                IPoint {
                    p: tri1[*v1 as usize],
                    t1,
                    t2: IntersectTopo::Vertex(*v1),
                },
                Some(IPoint {
                    p: tri2[*v2 as usize],
                    t1,
                    t2: IntersectTopo::Vertex(*v2),
                }),
            ),
        }
    }
}

fn edge_on_plane(
    tri1: &[DVec3; 3],
    tri2: &[DVec3; 3],
    on_plane_vertex: (u8, u8),
) -> TriTriIntersectStatus {
    // TODO
    let s = EdgeOnPlaneStatus::NotIntersect;
    s.to_status(tri1, tri2, on_plane_vertex)
}

enum VertexOnPlaneStatus {
    NotIntersect,
    IntersectVertex(u8),
    IntersectEdge(u8, u8),
    IntersectFace,
}

impl VertexOnPlaneStatus {
    fn to_status(&self, tri: &[DVec3; 3], p: DVec3, index: u8) -> TriTriIntersectStatus {
        let t1 = IntersectTopo::Vertex(index);
        match self {
            VertexOnPlaneStatus::NotIntersect => TriTriIntersectStatus::NotIntersect,
            VertexOnPlaneStatus::IntersectVertex(v) => TriTriIntersectStatus::Intersect(
                IPoint {
                    p,
                    t1,
                    t2: IntersectTopo::Vertex(*v),
                },
                None,
            ),
            VertexOnPlaneStatus::IntersectEdge(v1, v2) => TriTriIntersectStatus::Intersect(
                IPoint {
                    p,
                    t1,
                    t2: IntersectTopo::Edge(*v1, *v2),
                },
                None,
            ),
            VertexOnPlaneStatus::IntersectFace => TriTriIntersectStatus::Intersect(
                IPoint {
                    p,
                    t1,
                    t2: IntersectTopo::Face,
                },
                None,
            ),
        }
    }
}

fn vertex_on_plane(tri: &[DVec3; 3], p: DVec3, index: u8) -> TriTriIntersectStatus {
    todo!()
}

// TODO
// test line segment intersect
fn handle_coplaner(
    tri1: &[DVec3; 3],
    tri2: &[DVec3; 3],
    tri2_plane_normal: DVec3,
) -> TriTriIntersectStatus {
    let mat = DMat3::from_cols(tri2[0] - tri2[1], tri2[0] - tri2[2], tri2_plane_normal);
    
    let intersect_points = vec![IPoint {
        p: todo!(),
        t1: todo!(),
        t2: todo!(),
    }];
    TriTriIntersectStatus::Coplanar(intersect_points)
}
