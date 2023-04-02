use glam::Vec3;
use std::collections::HashMap;
use uuid::Uuid;

use crate::bv::AABB;

mod half_edge_mesh;
#[cfg(test)]
mod tests;
#[derive(Debug)]
pub struct Vertex {
    pub(crate) geometry: Vec3,
    next_half_edge: Uuid,
    pub(crate) uuid: Uuid,
}

impl Vertex {
    fn new(geometry: Vec3) -> Self {
        Vertex {
            geometry,
            next_half_edge: Uuid::nil(),
            uuid: Uuid::new_v4(),
        }
    }
}

#[derive(Debug)]
pub struct HalfEdge {
    next_vertex: Uuid,
    next_half_edge: Uuid,
    pre_half_edge: Uuid,
    pair_half_edge: Uuid,
    face: Uuid,
    uuid: Uuid,
}

impl HalfEdge {
    fn new(next_vertex: &Vertex) -> Self {
        HalfEdge {
            next_vertex: next_vertex.uuid,
            next_half_edge: Uuid::nil(),
            pre_half_edge: Uuid::nil(),
            pair_half_edge: Uuid::nil(),
            face: Uuid::nil(),
            uuid: Uuid::new_v4(),
        }
    }
}

#[derive(Debug)]
pub struct Face {
    edge: Uuid,
    normal: Vec3,
    uuid: Uuid,
}

impl Face {
    fn new(edge: &HalfEdge, normal: Vec3) -> Self {
        Face {
            edge: edge.uuid,
            normal,
            uuid: Uuid::new_v4(),
        }
    }
}

#[derive(Debug)]
pub struct HalfEdgeMesh {
    pub vertices: HashMap<Uuid, Vertex>,
    pub half_edges: HashMap<Uuid, HalfEdge>,
    pub faces: HashMap<Uuid, Face>,
}
