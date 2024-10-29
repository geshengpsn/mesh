use std::collections::HashMap;

use glam::{Mat4, Quat, Vec3};
use uuid::Uuid;

use crate::{
    bv::BoundingVolume,
    bvh::{build_options::BuildBvhOption, Bvh},
    IndexMesh,
};

use super::{Face, HalfEdge, HalfEdgeMesh, Vertex};

impl<BV> From<&IndexMesh> for HalfEdgeMesh<BV>
where
    BV: BoundingVolume<3>,
{
    fn from(mesh: &IndexMesh) -> Self {
        let mut half_edge_mesh = Self::default();
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
    pub(crate) fn new() -> Self {
        Self {
            vertices: HashMap::new(),
            half_edges: HashMap::new(),
            faces: HashMap::new(),
            bvh: None,
        }
    }
}

impl<BV> HalfEdgeMesh<BV>
where
    BV: BoundingVolume<3>,
{
    pub fn is_mesh_manifold(&self) -> bool {
        self.is_mesh_watertight() && self.is_vertex_manifold() && self.is_edge_manifold()
    }

    // TODO 
    /// https://cs184.eecs.berkeley.edu/uploads/lectures/10_mesh-rep/images/slide_018.jpg
    pub fn is_vertex_manifold(&self) -> bool {
        todo!()
    }

    // TODO 
    pub fn is_edge_manifold(&self) -> bool {
        todo!()
    }

    pub fn is_mesh_watertight(&self) -> bool {
        for (_, he) in self.half_edges.iter() {
            if he.pair_half_edge.is_nil() {
                return false;
            }
        }
        true
    }

    // TODO: implement
    pub fn is_mesh_self_intersect(&mut self) -> bool {
        if self.bvh.is_none() {
            self.build_bvh();
        }
        let bvh = self.bvh.as_ref().unwrap();
        for (_, face) in self.faces.iter() {
            let intersects = bvh.intersect_by(
                face,
                |f, (face_id, b)| {
                    if face_id == &f.uuid {
                        return false;
                    }
                    let (v1 , v2, v3) = self.find_vertex_in_face(f);
                    // v1.geometry

                    todo!()
                },
                |a, b| false,
            );
        }
        todo!()
    }

    pub(super) fn insert_vertex(&mut self, v: Vertex) {
        self.vertices.insert(v.uuid, v);
    }

    pub(super) fn insert_half_edge(&mut self, half_dege: HalfEdge) {
        self.half_edges.insert(half_dege.uuid, half_dege);
    }

    pub(super) fn insert_face(&mut self, f: Face<BV>) {
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
        let face = Face::new(&e31, normal);

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

    fn to_index_mesh(&self) -> IndexMesh {
        IndexMesh::from(self)
    }

    // use this function only when mesh is under construction
    // mesh is non manifold
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

    /// make sure mesh is manifold, otherwise return fault result
    pub fn find_half_edge(&self, start: &Vertex, end: &Vertex) -> Option<&HalfEdge> {
        debug_assert!(self.is_mesh_manifold());
        let edges = self.find_half_edges_from_start(start);
        edges.into_iter().find(|e| e.next_vertex == end.uuid)
    }

    /// make sure mesh is manifold, otherwise return fault result
    pub fn find_half_edges_from_start(&self, start: &Vertex) -> Vec<&HalfEdge> {
        debug_assert!(self.is_mesh_manifold());
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

    pub fn find_vertex_in_face(&self, face: &Face<BV>) -> (&Vertex, &Vertex, &Vertex) {
        let e1 = self.half_edges.get(&face.edge).unwrap();
        let e2 = self.half_edges.get(&e1.next_half_edge).unwrap();
        let e3 = self.half_edges.get(&e2.next_half_edge).unwrap();
        (
            self.vertices.get(&e1.next_vertex).unwrap(),
            self.vertices.get(&e2.next_vertex).unwrap(),
            self.vertices.get(&e3.next_vertex).unwrap(),
        )
    }

    pub fn transfrom(&mut self, mat4: Mat4) {
        self.vertices.iter_mut().for_each(|(_, v)| {
            v.geometry = mat4.transform_point3(v.geometry.clone());
        });
    }

    pub fn translate(&mut self, translation: Vec3) {
        let mat = Mat4::from_translation(translation);
        self.transfrom(mat);
    }

    pub fn scale(&mut self, scale: Vec3) {
        let mat = Mat4::from_scale(scale);
        self.transfrom(mat);
    }

    pub fn rotate(&mut self, rotation: Quat) {
        let mat = Mat4::from_quat(rotation);
        self.transfrom(mat);
    }

    pub(crate) fn build_bvh(&mut self) {
        let p = self
            .faces
            .iter()
            .map(|(_, face)| {
                let (v1, v2, v3) = self.find_vertex_in_face(face);
                (face.uuid, [v1.geometry, v2.geometry, v3.geometry])
            })
            .collect::<Vec<_>>();
        Bvh::<3, BV, _>::build(BuildBvhOption::default(), p);
    }

    // TODO find intersect triangles
    fn find_intersect(&self, other: &Self) -> Vec<(Uuid, Uuid)> {
        todo!()
    }
}
