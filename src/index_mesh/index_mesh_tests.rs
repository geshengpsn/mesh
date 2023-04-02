use std::{fs::File, io::Cursor};

use glam::Vec3;

use super::IndexMesh;

#[test]
fn test_from_stl() {
    let mut reader = ::std::io::Cursor::new(
        b"solid foobar
                   facet normal 0.1 0.2 0.3
                       outer loop
                           vertex 1 2 3
                           vertex 4 5 6e-15
                           vertex 7 8 9.87654321
                       endloop
                   endfacet
                   endsolid foobar"
            .to_vec(),
    );
    let res = IndexMesh::from_stl(&mut reader);
    assert!(res.is_ok());
    let mesh = res.unwrap();
    assert_eq!(mesh.vertices.len(), 3);
    assert_eq!(mesh.triangles.len(), 1);
    assert_eq!(mesh.vertices[mesh.triangles[0].0], Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(
        mesh.vertices[mesh.triangles[0].1],
        Vec3::new(4.0, 5.0, 6e-15)
    );
    assert_eq!(
        mesh.vertices[mesh.triangles[0].2],
        Vec3::new(7.0, 8.0, 9.87654321)
    );
}

#[test]
fn test_to_stl() {
    let mut writer = ::std::io::Cursor::new(Vec::new());
    let mut mesh = IndexMesh::new();
    mesh.vertices.push(Vec3::new(1.0, 2.0, 3.0));
    mesh.vertices.push(Vec3::new(4.0, 5.0, 6e-15));
    mesh.vertices.push(Vec3::new(7.0, 8.0, 9.87654321));
    mesh.triangles.push(super::IndexTriangle(0, 1, 2));
    let res = mesh.to_stl(&mut writer);
    assert!(res.is_ok());
}

#[test]
fn test_build_bvh() {
    let mesh = IndexMesh::from_stl(&mut File::open("assets/bunny.stl").unwrap()).unwrap();
    let start = std::time::Instant::now();
    let _bvh = mesh.build_aabb_bvh(Default::default());
    let end = std::time::Instant::now();
    println!("build bvh: {:?}", end - start);
    // let res = bvh.intersect(g, &mesh);
    // println!("{:?}", bvh);
}

#[test]
fn test_from_obj() {
    let mut f = File::open("assets/bunny.obj").unwrap();
    let res = IndexMesh::from_obj(&mut f);
    assert!(res.is_ok());
    println!("{:?}", res);
}

#[test]
fn test_to_obj() {
    let mut f = File::open("assets/bunny.obj").unwrap();
    let mut buf = vec![];
    let mesh = IndexMesh::from_obj(&mut f).unwrap();
    let tri_len = mesh.triangles.len();
    let ver_len = mesh.vertices.len();

    let res = mesh.to_obj(&mut buf);
    assert!(res.is_ok());

    let mut cursor = Cursor::new(buf);
    let mesh = IndexMesh::from_obj(&mut cursor).unwrap();
    assert_eq!(tri_len, mesh.triangles.len());
    assert_eq!(ver_len, mesh.vertices.len());
}
