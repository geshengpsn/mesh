use super::*;

#[test]
fn test_vertex() {
    let v = Vertex::new(Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(v.geometry, Vec3::new(1.0, 2.0, 3.0));
    assert!(v.next_half_edge.is_nil());
    assert_ne!(v.uuid, Uuid::nil());
    assert_ne!(v.uuid, Uuid::new_v4());
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

#[test]
fn test_face() {
    let normal = Vec3::new(1.0, 0.0, 0.0);
    let e = HalfEdge::new(&Vertex::new(Vec3::new(1.0, 2.0, 3.0)));
    let face = Face::new(&e, normal);
    assert_eq!(face.edge, e.uuid);
}

#[cfg(test)]
mod half_edge_mesh_test {
    // use std::fmt::Debug;

    // use petgraph::{
    //     dot::{Config, Dot},
    //     prelude::DiGraph,
    // };

    use crate::{Box, IndexMesh};

    use super::*;

    // enum Node<'a> {
    //     Vertex(&'a Vertex),
    //     HalfEdge(&'a HalfEdge),
    //     Face(&'a Face),
    // }

    // impl Debug for Node<'_> {
    //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //         match self {
    //             Self::Vertex(_) => f.write_str("Vertex"),
    //             Self::HalfEdge(_) => f.write_str("HalfEdge"),
    //             Self::Face(_) => f.write_str("Face"),
    //         }
    //     }
    // }

    // fn print_mesh_graph(mesh: &HalfEdgeMesh) {
    //     let mut g = DiGraph::<Node, ()>::new();

    //     let mut vertex_map = HashMap::new();
    //     for v in mesh.vertices.values() {
    //         let index = g.add_node(Node::Vertex(v));
    //         vertex_map.insert(v.uuid, index);
    //     }

    //     let mut edge_map = HashMap::new();
    //     for e in mesh.half_edges.values() {
    //         let index = g.add_node(Node::HalfEdge(e));
    //         edge_map.insert(e.uuid, index);
    //     }

    //     let mut face_map = HashMap::new();
    //     for f in mesh.faces.values() {
    //         let index = g.add_node(Node::Face(f));
    //         face_map.insert(f.uuid, index);
    //     }

    //     for v in mesh.vertices.values() {
    //         let v_index = vertex_map.get(&v.uuid).unwrap();
    //         let e_index = edge_map.get(&v.next_half_edge).unwrap();
    //         g.add_edge(*v_index, *e_index, ());
    //     }

    //     for e in mesh.half_edges.values() {
    //         let e_index = edge_map.get(&e.uuid).unwrap();
    //         let v_index = vertex_map.get(&e.next_vertex).unwrap();
    //         g.add_edge(*e_index, *v_index, ());
    //         let f_index = face_map.get(&e.face).unwrap();
    //         g.add_edge(*e_index, *f_index, ());

    //         let next_e_index = edge_map.get(&e.next_half_edge).unwrap();
    //         let pre_e_index = edge_map.get(&e.pre_half_edge).unwrap();
    //         g.add_edge(*e_index, *next_e_index, ());
    //         g.add_edge(*e_index, *pre_e_index, ());
    //         if !e.pair_half_edge.is_nil() {
    //             let pair_e_index = edge_map.get(&e.pair_half_edge).unwrap();
    //             g.add_edge(*e_index, *pair_e_index, ());
    //         }
    //     }

    //     for f in mesh.faces.values() {
    //         let f_index = face_map.get(&f.uuid).unwrap();
    //         let e_index = edge_map.get(&f.edge).unwrap();
    //         g.add_edge(*f_index, *e_index, ());
    //     }
    //     let dot = Dot::with_config(&g, &[Config::EdgeNoLabel]);
    //     println!("{:?}", dot);
    // }

    #[test]
    fn test_is_manifold() {
        let mut mesh = HalfEdgeMesh::new();
        let vertex = Vertex::new(Vec3::new(0., 0., 0.));
        let half_dege = HalfEdge::new(&vertex);
        mesh.insert_half_edge(half_dege);
        assert!(!mesh.is_mesh_manifold());
    }

    #[test]
    fn test_from_index_mesh() {
        let mesh = Box { size: 2. }.to_mesh();
        let mesh = HalfEdgeMesh::from(&mesh);
        // print_mesh_graph(&mesh);
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
        let mesh = Box { size: 2. }.to_mesh();
        let mesh = HalfEdgeMesh::from(&mesh);
        let mut count = 0;
        for (_, v1) in mesh.vertices.iter() {
            for (_, v2) in mesh.vertices.iter() {
                if v1.uuid == v2.uuid {
                    continue;
                }
                if let Some(_) = mesh.find_half_edge(v1, v2) {
                    count += 1;
                }
            }
        }
        assert_eq!(count, 36);
    }

    #[test]
    fn test_find_half_edges_from_start_edge_condation() {
        let mesh = HalfEdgeMesh::new();
        let v = Vertex::new(Vec3::new(0.0, 0.0, 0.0));
        let edges = mesh.find_half_edges_from_start(&v);
        assert!(edges.is_empty());
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
        let f1 = Face::new(&ec1, Vec3::new(0.0, 0.0, 0.0));

        // triangle 2
        let mut ec2 = HalfEdge::new(&v2);
        let mut e23 = HalfEdge::new(&v3);
        let mut e3c = HalfEdge::new(&center);
        let f2 = Face::new(&ec2, Vec3::new(0.0, 0.0, 0.0));

        // triangle 3
        let mut ec3 = HalfEdge::new(&v3);
        let mut e31 = HalfEdge::new(&v1);
        let mut e1c = HalfEdge::new(&center);
        let f3 = Face::new(&ec3, Vec3::new(0.0, 0.0, 0.0));

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
