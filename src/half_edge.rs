use glam::Vec3;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{aabb::AABB, index_mesh::IndexMesh};

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

#[test]
fn test_vertex() {
    let v = Vertex::new(Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(v.geometry, Vec3::new(1.0, 2.0, 3.0));
    assert!(v.next_half_edge.is_nil());
    assert_ne!(v.uuid, Uuid::nil());
    assert_ne!(v.uuid, Uuid::new_v4());
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

#[test]
fn test_halfedge() {
    let v = Vertex::new(Vec3::new(1.0, 2.0, 3.0));
    let he = HalfEdge::new(&v);
    assert_eq!(he.next_vertex, v.uuid);
    assert!(he.next_half_edge.is_nil());
    assert!(he.pre_half_edge.is_nil());
    assert!(he.pair_half_edge.is_nil());
    assert!(he.face.is_nil());
}

#[derive(Debug)]
pub struct Face {
    edge: Uuid,
    aabb: AABB,
    normal: Vec3,
    uuid: Uuid,
}

impl Face {
    fn new(edge: &HalfEdge, aabb: AABB, normal: Vec3) -> Self {
        Face {
            edge: edge.uuid,
            aabb,
            normal,
            uuid: Uuid::new_v4(),
        }
    }
}

#[test]
fn test_face() {
    let aabb = AABB::new();
    let normal = Vec3::new(1.0, 0.0, 0.0);
    let e = HalfEdge::new(&Vertex::new(Vec3::new(1.0, 2.0, 3.0)));
    let face = Face::new(&e, aabb, normal);
    assert_eq!(face.edge, e.uuid);
}

#[derive(Debug)]
pub struct HalfEdgeMesh {
    pub vertices: HashMap<Uuid, Vertex>,
    pub half_edges: HashMap<Uuid, HalfEdge>,
    pub faces: HashMap<Uuid, Face>,
}

impl From<&IndexMesh> for HalfEdgeMesh {
    fn from(mesh: &IndexMesh) -> Self {
        HalfEdgeMesh::from_index_mesh(mesh)
    }
}

impl HalfEdgeMesh {
    fn new() -> Self {
        Self {
            vertices: HashMap::new(),
            half_edges: HashMap::new(),
            faces: HashMap::new(),
        }
    }

    pub fn is_mesh_manifold(&self) -> bool {
        for (_, he) in self.half_edges.iter() {
            if he.pair_half_edge.is_nil() {
                return false;
            }
        }
        true
    }

    fn insert_vertex(&mut self, v: Vertex) {
        self.vertices.insert(v.uuid, v);
    }

    fn insert_half_edge(&mut self, half_dege: HalfEdge) {
        self.half_edges.insert(half_dege.uuid, half_dege);
    }

    fn insert_face(&mut self, f: Face) {
        self.faces.insert(f.uuid, f);
    }

    // assume all vertex has inserted before
    fn make_face(&mut self, v1: &Uuid, v2: &Uuid, v3: &Uuid, map: &mut HashMap<Uuid, Vec<Uuid>>) {
        let v1 = self.vertices.get(v1).unwrap();
        let v2 = self.vertices.get(v2).unwrap();
        let v3 = self.vertices.get(v3).unwrap();

        // make halfedges
        let mut e31 = HalfEdge::new(v1);
        let mut e12 = HalfEdge::new(v2);
        let mut e23 = HalfEdge::new(v3);

        // make face
        let normal = (v2.geometry - v1.geometry)
            .cross(v3.geometry - v1.geometry)
            .normalize();
        let face = Face::new(
            &e31,
            AABB::from_triangle(&[v1.geometry.into(), v2.geometry.into(), v3.geometry.into()]),
            normal,
        );

        // find pair halfedge
        let e13_uuid = self.find_half_edge_by_outgoing_map(v1, v3, map);
        let e21_uuid = self.find_half_edge_by_outgoing_map(v2, v1, map);
        let e32_uuid = self.find_half_edge_by_outgoing_map(v3, v2, map);

        if let Some(e13_uuid) = e13_uuid {
            let e13 = self.half_edges.get(&e13_uuid).unwrap();
            e31.pair_half_edge = e13.uuid;
            unsafe {
                (e13 as *const HalfEdge as *mut HalfEdge)
                    .as_mut()
                    .unwrap()
                    .pair_half_edge = e31.uuid;
            }
        }
        if let Some(e21_uuid) = e21_uuid {
            let e21 = self.half_edges.get(&e21_uuid).unwrap();
            e12.pair_half_edge = e21.uuid;
            unsafe {
                (e21 as *const HalfEdge as *mut HalfEdge)
                    .as_mut()
                    .unwrap()
                    .pair_half_edge = e12.uuid;
            }
        }
        if let Some(e32_uuid) = e32_uuid {
            let e32 = self.half_edges.get(&e32_uuid).unwrap();
            e23.pair_half_edge = e32.uuid;
            unsafe {
                (e32 as *const HalfEdge as *mut HalfEdge)
                    .as_mut()
                    .unwrap()
                    .pair_half_edge = e23.uuid;
            }
        }

        map.entry(v1.uuid)
            .and_modify(|v| v.push(e12.uuid))
            .or_insert(vec![e12.uuid]);
        map.entry(v2.uuid)
            .and_modify(|v| v.push(e23.uuid))
            .or_insert(vec![e23.uuid]);
        map.entry(v3.uuid)
            .and_modify(|v| v.push(e31.uuid))
            .or_insert(vec![e31.uuid]);

        unsafe {
            let v1 = (v1 as *const Vertex as *mut Vertex).as_mut().unwrap();
            let v2 = (v2 as *const Vertex as *mut Vertex).as_mut().unwrap();
            let v3 = (v3 as *const Vertex as *mut Vertex).as_mut().unwrap();
            v1.next_half_edge = e12.uuid;
            v2.next_half_edge = e23.uuid;
            v3.next_half_edge = e31.uuid;
        }

        e12.next_half_edge = e23.uuid;
        e23.next_half_edge = e31.uuid;
        e31.next_half_edge = e12.uuid;

        e12.pre_half_edge = e31.uuid;
        e23.pre_half_edge = e12.uuid;
        e31.pre_half_edge = e23.uuid;

        e12.face = face.uuid;
        e23.face = face.uuid;
        e31.face = face.uuid;

        self.insert_half_edge(e12);
        self.insert_half_edge(e23);
        self.insert_half_edge(e31);
        self.insert_face(face);
    }

    fn from_index_mesh(mesh: &IndexMesh) -> Self {
        let mut half_edge_mesh = Self::new();
        let mut ids = vec![];
        for v in mesh.vertices.iter() {
            let vertex = Vertex::new(*v);
            ids.push(vertex.uuid);
            half_edge_mesh.insert_vertex(vertex);
        }
        let mut outgoing_edges = HashMap::<Uuid, Vec<Uuid>>::new();
        for t in mesh.triangles.iter() {
            half_edge_mesh.make_face(&ids[t.0], &ids[t.1], &ids[t.2], &mut outgoing_edges);
        }
        half_edge_mesh
    }
}

impl HalfEdgeMesh {
    fn find_half_edge_by_outgoing_map(
        &self,
        start: &Vertex,
        end: &Vertex,
        map: &HashMap<Uuid, Vec<Uuid>>,
    ) -> Option<Uuid> {
        let Some(edges) = map.get(&start.uuid) else {
            return None
        };
        for e_uuid in edges {
            let e = self.half_edges.get(e_uuid).unwrap();
            if e.next_vertex == end.uuid {
                return Some(e.uuid);
            }
        }
        None
    }
    pub fn find_half_edge(&self, start: &Vertex, end: &Vertex) -> Option<&HalfEdge> {
        let edges = self.find_half_edges_from_start(start);
        edges.into_iter().find(|e| e.next_vertex == end.uuid)
    }

    pub fn find_half_edges_from_start(&self, start: &Vertex) -> Vec<&HalfEdge> {
        let mut res = vec![];
        let start = match self.half_edges.get(&start.next_half_edge) {
            Some(e) => e,
            None => return vec![],
        };
        res.push(start);
        let mut left = start;
        let mut right = start;
        let mut loop_closed = false;
        loop {
            let left_uuid = self
                .half_edges
                .get(&left.pre_half_edge)
                .unwrap()
                .pair_half_edge;
            if left_uuid.is_nil() {
                break;
            }
            if left_uuid == start.uuid {
                loop_closed = true;
                break;
            }
            left = self.half_edges.get(&left_uuid).unwrap();
            res.push(left);
        }
        if loop_closed {
            return res;
        }
        loop {
            if right.pair_half_edge.is_nil() {
                break;
            }
            let right_uuid = self
                .half_edges
                .get(&right.pair_half_edge)
                .unwrap()
                .next_half_edge;
            right = self.half_edges.get(&right_uuid).unwrap();
            res.push(right);
        }
        res
    }

    pub fn find_vertex_in_face(&self, face: &Face) -> (&Vertex, &Vertex, &Vertex) {
        let e1 = self.half_edges.get(&face.edge).unwrap();
        let e2 = self.half_edges.get(&e1.next_half_edge).unwrap();
        let e3 = self.half_edges.get(&e2.next_half_edge).unwrap();
        (
            self.vertices.get(&e1.next_vertex).unwrap(),
            self.vertices.get(&e2.next_vertex).unwrap(),
            self.vertices.get(&e3.next_vertex).unwrap(),
        )
    }
}

#[cfg(test)]
mod half_edge_mesh_test {
    use std::fmt::Debug;

    use petgraph::{
        dot::{Config, Dot},
        prelude::DiGraph,
    };

    use crate::Box;

    use super::*;

    enum Node<'a> {
        Vertex(&'a Vertex),
        HalfEdge(&'a HalfEdge),
        Face(&'a Face),
    }

    impl Debug for Node<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Vertex(_) => f.write_str("Vertex"),
                Self::HalfEdge(_) => f.write_str("HalfEdge"),
                Self::Face(_) => f.write_str("Face"),
            }
        }
    }

    fn print_mesh_graph(mesh: &HalfEdgeMesh) {
        let mut g = DiGraph::<Node, ()>::new();

        let mut vertex_map = HashMap::new();
        for v in mesh.vertices.values() {
            let index = g.add_node(Node::Vertex(v));
            vertex_map.insert(v.uuid, index);
        }

        let mut edge_map = HashMap::new();
        for e in mesh.half_edges.values() {
            let index = g.add_node(Node::HalfEdge(e));
            edge_map.insert(e.uuid, index);
        }

        let mut face_map = HashMap::new();
        for f in mesh.faces.values() {
            let index = g.add_node(Node::Face(f));
            face_map.insert(f.uuid, index);
        }

        for v in mesh.vertices.values() {
            let v_index = vertex_map.get(&v.uuid).unwrap();
            let e_index = edge_map.get(&v.next_half_edge).unwrap();
            g.add_edge(*v_index, *e_index, ());
        }

        for e in mesh.half_edges.values() {
            let e_index = edge_map.get(&e.uuid).unwrap();
            let v_index = vertex_map.get(&e.next_vertex).unwrap();
            g.add_edge(*e_index, *v_index, ());
            let f_index = face_map.get(&e.face).unwrap();
            g.add_edge(*e_index, *f_index, ());

            let next_e_index = edge_map.get(&e.next_half_edge).unwrap();
            let pre_e_index = edge_map.get(&e.pre_half_edge).unwrap();
            g.add_edge(*e_index, *next_e_index, ());
            g.add_edge(*e_index, *pre_e_index, ());
            if !e.pair_half_edge.is_nil() {
                let pair_e_index = edge_map.get(&e.pair_half_edge).unwrap();
                g.add_edge(*e_index, *pair_e_index, ());
            }
        }

        for f in mesh.faces.values() {
            let f_index = face_map.get(&f.uuid).unwrap();
            let e_index = edge_map.get(&f.edge).unwrap();
            g.add_edge(*f_index, *e_index, ());
        }
        let dot = Dot::with_config(&g, &[Config::EdgeNoLabel]);
        println!("{:?}", dot);
    }

    #[test]
    fn test_from_index_mesh() {
        let mesh = Box { size: 2. }.to_mesh();
        let mesh = HalfEdgeMesh::from(&mesh);
        print_mesh_graph(&mesh);
        assert!(mesh.is_mesh_manifold());
        assert_eq!(mesh.vertices.len(), 8);
        assert_eq!(mesh.half_edges.len(), 36);
        assert_eq!(mesh.faces.len(), 12);
    }

    fn construct_half_edge_mesh() -> HalfEdgeMesh {
        let mut f = std::fs::File::open("assets/bunny.stl").unwrap();
        let mesh = IndexMesh::from_stl(&mut f).unwrap();
        HalfEdgeMesh::from(&mesh)
    }

    #[test]
    fn test_from_stl() {
        let mesh = construct_half_edge_mesh();
        assert!(mesh.is_mesh_manifold());
    }

    #[test]
    fn test_insert_vertex() {
        let mut mesh = HalfEdgeMesh::new();
        let v1 = Vertex::new(Vec3::new(1.0, 0.0, 0.0));
        let v2 = Vertex::new(Vec3::new(0.0, 1.0, 0.0));
        let v3 = Vertex::new(Vec3::new(0.0, 0.0, 1.0));
        mesh.insert_vertex(v1);
        mesh.insert_vertex(v2);
        mesh.insert_vertex(v3);
        assert_eq!(mesh.vertices.len(), 3);
    }

    #[test]
    fn test_find_half_edges_from_start() {
        let mut mesh = HalfEdgeMesh::new();
        let mut center = Vertex::new(Vec3::new(0.0, 0.0, 0.0));
        let center_uuid = center.uuid;
        let mut v1 = Vertex::new(Vec3::new(0.0, 0.0, 0.0));
        let v1_uuid = v1.uuid;
        let mut v2 = Vertex::new(Vec3::new(0.0, 0.0, 0.0));
        let mut v3 = Vertex::new(Vec3::new(0.0, 0.0, 0.0));

        // triangle 1
        let mut ec1 = HalfEdge::new(&v1);
        let mut e12 = HalfEdge::new(&v2);
        let mut e2c = HalfEdge::new(&center);
        let f1 = Face::new(&ec1, AABB::new(), Vec3::new(0.0, 0.0, 0.0));

        // triangle 2
        let mut ec2 = HalfEdge::new(&v2);
        let mut e23 = HalfEdge::new(&v3);
        let mut e3c = HalfEdge::new(&center);
        let f2 = Face::new(&ec2, AABB::new(), Vec3::new(0.0, 0.0, 0.0));

        // triangle 3
        let mut ec3 = HalfEdge::new(&v3);
        let mut e31 = HalfEdge::new(&v1);
        let mut e1c = HalfEdge::new(&center);
        let f3 = Face::new(&ec3, AABB::new(), Vec3::new(0.0, 0.0, 0.0));

        // vertex next edge
        center.next_half_edge = ec1.uuid;
        v1.next_half_edge = e12.uuid;
        v2.next_half_edge = e23.uuid;
        v3.next_half_edge = e31.uuid;

        // edge next edge
        ec1.next_half_edge = e12.uuid;
        e12.next_half_edge = e2c.uuid;
        e2c.next_half_edge = ec1.uuid;

        ec2.next_half_edge = e23.uuid;
        e23.next_half_edge = e3c.uuid;
        e3c.next_half_edge = ec2.uuid;

        ec3.next_half_edge = e31.uuid;
        e31.next_half_edge = e1c.uuid;
        e1c.next_half_edge = ec3.uuid;

        // edge pre edge
        e12.pre_half_edge = ec1.uuid;
        e2c.pre_half_edge = e12.uuid;
        ec1.pre_half_edge = e2c.uuid;

        e23.pre_half_edge = ec2.uuid;
        e3c.pre_half_edge = e23.uuid;
        ec2.pre_half_edge = e3c.uuid;

        e31.pre_half_edge = ec3.uuid;
        e1c.pre_half_edge = e31.uuid;
        ec3.pre_half_edge = e1c.uuid;

        // edge pair dege
        ec1.pair_half_edge = e1c.uuid;
        e1c.pair_half_edge = ec1.uuid;

        ec2.pair_half_edge = e2c.uuid;
        e2c.pair_half_edge = ec2.uuid;

        ec3.pair_half_edge = e3c.uuid;
        e3c.pair_half_edge = ec3.uuid;

        // edge face
        ec1.face = f1.uuid;
        e12.face = f1.uuid;
        e2c.face = f1.uuid;

        ec2.face = f2.uuid;
        e23.face = f2.uuid;
        e3c.face = f2.uuid;

        ec3.face = f3.uuid;
        e31.face = f3.uuid;
        e1c.face = f3.uuid;

        mesh.insert_vertex(v1);
        mesh.insert_vertex(v2);
        mesh.insert_vertex(v3);
        mesh.insert_vertex(center);

        let ec1_uuid = ec1.uuid;
        mesh.insert_half_edge(ec1);
        mesh.insert_half_edge(e12);
        mesh.insert_half_edge(e2c);
        mesh.insert_half_edge(ec2);
        mesh.insert_half_edge(e23);
        mesh.insert_half_edge(e3c);
        mesh.insert_half_edge(ec3);
        mesh.insert_half_edge(e31);
        mesh.insert_half_edge(e1c);

        mesh.insert_face(f1);
        mesh.insert_face(f2);
        mesh.insert_face(f3);

        let center_ref = mesh.vertices.get(&center_uuid).unwrap();
        let v1_ref = mesh.vertices.get(&v1_uuid).unwrap();
        let res = mesh.find_half_edges_from_start(center_ref);
        assert_eq!(res.len(), 3);
        let res = mesh.find_half_edge(center_ref, v1_ref);
        let ec1_ref = mesh.half_edges.get(&ec1_uuid).unwrap();
        assert!(res.is_some());
        assert_eq!(res.unwrap().uuid, ec1_ref.uuid);
    }

    #[test]
    fn test_make_face() {
        let mut mesh = HalfEdgeMesh::new();

        let mut center = Vertex::new(Vec3::new(0.0, 0.0, 0.0));
        let mut v1 = Vertex::new(Vec3::new(0.0, 0.0, 0.0));
        let mut v2 = Vertex::new(Vec3::new(0.0, 0.0, 0.0));
        let mut v3 = Vertex::new(Vec3::new(0.0, 0.0, 0.0));

        // triangle 1
        let mut ec1 = HalfEdge::new(&v1);
        let mut e12 = HalfEdge::new(&v2);
        let mut e2c = HalfEdge::new(&center);
        let f1 = Face::new(&ec1, AABB::new(), Vec3::new(0.0, 0.0, 0.0));

        // triangle 2
        let mut ec2 = HalfEdge::new(&v2);
        let mut e23 = HalfEdge::new(&v3);
        let mut e3c = HalfEdge::new(&center);
        let f2 = Face::new(&ec2, AABB::new(), Vec3::new(0.0, 0.0, 0.0));

        // triangle 3
        let mut ec3 = HalfEdge::new(&v3);
        let mut e31 = HalfEdge::new(&v1);
        let mut e1c = HalfEdge::new(&center);
        let f3 = Face::new(&ec3, AABB::new(), Vec3::new(0.0, 0.0, 0.0));

        // vertex next edge
        center.next_half_edge = ec1.uuid;
        v1.next_half_edge = e12.uuid;
        v2.next_half_edge = e23.uuid;
        v3.next_half_edge = e31.uuid;

        // edge next edge
        ec1.next_half_edge = e12.uuid;
        e12.next_half_edge = e2c.uuid;
        e2c.next_half_edge = ec1.uuid;

        ec2.next_half_edge = e23.uuid;
        e23.next_half_edge = e3c.uuid;
        e3c.next_half_edge = ec2.uuid;

        ec3.next_half_edge = e31.uuid;
        e31.next_half_edge = e1c.uuid;
        e1c.next_half_edge = ec3.uuid;

        // edge pre edge
        e12.pre_half_edge = ec1.uuid;
        e2c.pre_half_edge = e12.uuid;
        ec1.pre_half_edge = e2c.uuid;

        e23.pre_half_edge = ec2.uuid;
        e3c.pre_half_edge = e23.uuid;
        ec2.pre_half_edge = e3c.uuid;

        e31.pre_half_edge = ec3.uuid;
        e1c.pre_half_edge = e31.uuid;
        ec3.pre_half_edge = e1c.uuid;

        // edge pair dege
        ec1.pair_half_edge = e1c.uuid;
        e1c.pair_half_edge = ec1.uuid;

        ec2.pair_half_edge = e2c.uuid;
        e2c.pair_half_edge = ec2.uuid;

        ec3.pair_half_edge = e3c.uuid;
        e3c.pair_half_edge = ec3.uuid;

        // edge face
        ec1.face = f1.uuid;
        e12.face = f1.uuid;
        e2c.face = f1.uuid;

        ec2.face = f2.uuid;
        e23.face = f2.uuid;
        e3c.face = f2.uuid;

        ec3.face = f3.uuid;
        e31.face = f3.uuid;
        e1c.face = f3.uuid;

        mesh.insert_vertex(v1);
        mesh.insert_vertex(v2);
        mesh.insert_vertex(v3);
        mesh.insert_vertex(center);

        mesh.insert_half_edge(ec1);
        mesh.insert_half_edge(e12);
        mesh.insert_half_edge(e2c);
        mesh.insert_half_edge(ec2);
        mesh.insert_half_edge(e23);
        mesh.insert_half_edge(e3c);
        mesh.insert_half_edge(ec3);
        mesh.insert_half_edge(e31);
        mesh.insert_half_edge(e1c);

        mesh.insert_face(f1);
        mesh.insert_face(f2);
        mesh.insert_face(f3);

        let index_mesh = IndexMesh::from(&mesh);

        println!("{:?}", index_mesh);
    }
}
