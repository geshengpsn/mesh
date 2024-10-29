use std::{
    collections::HashMap,
    ops::{BitAnd, BitOr, Sub},
};
use uuid::Uuid;

use crate::{half_edge::HalfEdgeMesh, tree::Tree};

#[cfg(test)]
mod csg_tests;

struct ClusterdHalfEdgeMesh {
    origin_mesh: HalfEdgeMesh,
    origin_cluster_uuid: Uuid,
    /// cluster id -> vec of face id, rest belong to origin cluster
    clusters: HashMap<Uuid, Vec<Uuid>>,
    /// edge id -> ((cluster id, vec<half edge id>), (cluster id, vec<half edge id>)),
    /// vec of half edge id should be closed
    edges: HashMap<Uuid, ((Uuid, Vec<Uuid>), (Uuid, Vec<Uuid>))>,
}

// https://arxiv.org/pdf/2205.14151.pdf
// https://arxiv.org/pdf/1308.4434.pdf
impl ClusterdHalfEdgeMesh {
    fn new(mesh: HalfEdgeMesh) -> Self {
        ClusterdHalfEdgeMesh {
            origin_mesh: mesh,
            origin_cluster_uuid: Uuid::new_v4(),
            clusters: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    fn split(&mut self, other: &mut Self) {
        // TODO
        if !self.origin_mesh.is_mesh_self_intersect()
            || self.origin_mesh.is_mesh_manifold()
            || !other.origin_mesh.is_mesh_self_intersect()
            || other.origin_mesh.is_mesh_manifold()
        {
            panic!("mesh need to be non self-intersect and manifold");
        }
        let bvh1 = self.origin_mesh.bvh.as_ref().unwrap();
        let bvh2 = other.origin_mesh.bvh.as_ref().unwrap();
        // find intersect face pairs
        
    }

    fn union(&self, other: &Self) -> Self {
        // TODO
        // intersect test
        // mesh need to be non self-intersect
        // if

        todo!()
    }

    fn intersection(&self, other: &Self) -> Self {
        // TODO
        todo!()
    }

    fn subtraction(&self, other: &Self) -> Self {
        // TODO
        todo!()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CsgOperation {
    Union,
    Intersection,
    Difference,
}

struct CsgNode {
    mesh: Option<ClusterdHalfEdgeMesh>,
    operation: Option<CsgOperation>,
}

impl CsgNode {
    fn new(origin_mesh: HalfEdgeMesh) -> Self {
        CsgNode {
            mesh: Some(ClusterdHalfEdgeMesh::new(origin_mesh)),
            operation: None,
        }
    }
}

pub struct CsgSolid {
    tree: Tree<CsgNode>,
}

impl CsgSolid {
    pub fn new(origin_mesh: HalfEdgeMesh) -> Self {
        CsgSolid {
            tree: Tree::new_root(CsgNode::new(origin_mesh)),
        }
    }
}

impl CsgSolid {
    fn merge(self, other: Self, operation: CsgOperation) -> Self {
        let tree = self.tree.merge(
            other.tree,
            CsgNode {
                mesh: None,
                operation: Some(operation),
            },
        );
        Self { tree }
    }

    pub fn union(self, mesh: Self) -> Self {
        self.merge(mesh, CsgOperation::Union)
    }

    pub fn intersection(self, mesh: Self) -> Self {
        self.merge(mesh, CsgOperation::Intersection)
    }

    pub fn difference(self, mesh: Self) -> Self {
        self.merge(mesh, CsgOperation::Difference)
    }

    pub fn construct(&mut self) {
        let root = self.tree.get_node(0).unwrap();
        if let Some(_) = &root.data.mesh {
            // no need to do any thing
            return;
        }

        // init works
        let mut vec = vec![0usize];
        let mut current_pointer = 0;
        while let Some(node_index) = vec.get(current_pointer) {
            let node = self.tree.get_node(*node_index).unwrap();
            if node.data.mesh.is_none() {
                if node.left.is_none() {
                    vec.push(node.left.unwrap());
                }
                if node.right.is_none() {
                    vec.push(node.right.unwrap());
                }
            }
            current_pointer += 1;
        }

        // do works in reverse order

        for node_index in vec.iter().rev() {
            let node = self.tree.get_node(*node_index).unwrap();
            // must have left and right
            let left_mesh = self
                .tree
                .get_node(node.left.unwrap())
                .unwrap()
                .data
                .mesh
                .as_ref()
                .unwrap();
            let right_mesh = self
                .tree
                .get_node(node.right.unwrap())
                .unwrap()
                .data
                .mesh
                .as_ref()
                .unwrap();

            let operation = node.data.operation.unwrap();
            let new_mesh = match operation {
                CsgOperation::Union => left_mesh.union(right_mesh),
                CsgOperation::Intersection => left_mesh.intersection(right_mesh),
                CsgOperation::Difference => left_mesh.subtraction(right_mesh),
            };
            self.tree.get_node_mut(*node_index).unwrap().data.mesh = Some(new_mesh);
        }
    }
}

impl BitAnd for CsgSolid {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

impl BitOr for CsgSolid {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl Sub for CsgSolid {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.difference(rhs)
    }
}
