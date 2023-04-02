use std::{collections::HashMap, fmt::Debug, slice::Iter};

use anyhow::Ok;
use glam::Vec3;

use crate::{
    bvh::{build_options::BuildBvhOption, BVH},
    half_edge::HalfEdgeMesh,
    traits::Bounded,
    AABB,
};

#[cfg(test)]
mod index_mesh_tests;

#[derive(Clone, Copy)]
pub struct IndexTriangle(pub usize, pub usize, pub usize);

impl Debug for IndexTriangle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({}, {}, {})", self.0, self.1, self.2))?;
        std::fmt::Result::Ok(())
    }
}

#[derive(Debug)]
pub struct IndexMesh {
    pub(crate) vertices: Vec<Vec3>,
    pub(crate) triangles: Vec<IndexTriangle>,
}

impl From<&HalfEdgeMesh> for IndexMesh {
    fn from(value: &HalfEdgeMesh) -> Self {
        let mut mesh = IndexMesh::new();
        let mut vertex_map = HashMap::new();
        for (index, (uuid, v)) in value.vertices.iter().enumerate() {
            mesh.vertices.push(v.geometry);
            vertex_map.insert(uuid, index);
        }
        for (_, f) in value.faces.iter() {
            let (v1, v2, v3) = value.find_vertex_in_face(f);
            mesh.triangles.push(IndexTriangle(
                *vertex_map.get(&v1.uuid).unwrap(),
                *vertex_map.get(&v2.uuid).unwrap(),
                *vertex_map.get(&v3.uuid).unwrap(),
            ));
        }
        mesh
    }
}

impl IndexMesh {
    pub fn new() -> Self {
        IndexMesh {
            vertices: vec![],
            triangles: vec![],
        }
    }

    pub fn vertices(&self) -> Iter<Vec3> {
        self.vertices.iter()
    }

    pub fn triangles(&self) -> Iter<IndexTriangle> {
        self.triangles.iter()
    }

    pub fn from_stl<F: std::io::Read + std::io::Seek>(f: &mut F) -> anyhow::Result<Self> {
        let mut res = IndexMesh::new();
        let mesh = stl_io::read_stl(f)?;
        mesh.vertices.into_iter().for_each(|vertex| {
            res.vertices
                .push(Vec3::new(vertex[0], vertex[1], vertex[2]));
        });
        mesh.faces.into_iter().for_each(|face| {
            res.triangles.push(IndexTriangle(
                face.vertices[0],
                face.vertices[1],
                face.vertices[2],
            ));
        });
        Ok(res)
    }

    pub fn to_stl<F: std::io::Write>(&self, f: &mut F) -> anyhow::Result<()> {
        let mut triangles = Vec::new();
        for triangle in self.triangles.iter() {
            let normal = (self.vertices[triangle.1] - self.vertices[triangle.0])
                .cross(self.vertices[triangle.2] - self.vertices[triangle.0])
                .normalize();
            triangles.push(stl_io::Triangle {
                normal: stl_io::Vertex::new(normal.into()),
                vertices: [
                    stl_io::Vertex::new(self.vertices[triangle.0].into()),
                    stl_io::Vertex::new(self.vertices[triangle.1].into()),
                    stl_io::Vertex::new(self.vertices[triangle.2].into()),
                ],
            });
        }
        stl_io::write_stl(f, triangles.iter())?;
        Ok(())
    }

    pub fn from_obj<F: std::io::Read>(f: &mut F) -> anyhow::Result<Self> {
        let data = obj::ObjData::load_buf(f)?;
        let mut triangles = vec![];
        for obj in data.objects {
            for g in obj.groups {
                for p in g.polys {
                    let mut iter = p.0.iter();
                    triangles.push(IndexTriangle(
                        iter.next().unwrap().0,
                        iter.next().unwrap().0,
                        iter.next().unwrap().0,
                    ));
                }
            }
        }

        Ok(IndexMesh {
            vertices: data
                .position
                .into_iter()
                .map(|p| Vec3::from_array(p))
                .collect::<Vec<_>>(),
            triangles,
        })
    }

    pub fn to_obj<F: std::io::Write>(&self, f: &mut F) -> anyhow::Result<()> {
        for v in self.vertices.iter() {
            writeln!(f, "v {} {} {}", v.x, v.y, v.z)?;
        }

        for t in self.triangles.iter() {
            writeln!(f, "f {} {} {}", t.0 + 1, t.1 + 1, t.2 + 1)?;
        }
        Ok(())
    }

    pub fn build_aabb_bvh<'a>(&'a self, option: BuildBvhOption) -> BVH<3, AABB<3>, &IndexTriangle>
    where
        (&'a IndexTriangle, [Vec3; 3]): Bounded<3, AABB<3>>,
    {
        let a = self
            .triangles()
            .map(|t| {
                (
                    t,
                    [self.vertices[t.0], self.vertices[t.1], self.vertices[t.2]],
                )
            })
            .collect::<Vec<_>>();
        let a = BVH::<3, AABB<3>, _>::build(option, a);
        a.transfrom::<&IndexTriangle>()
    }

    /// ## Example
    ///
    ///
    /// ```
    /// fn build_bevy_mesh() -> Mesh {
    ///     let mesh = Box{size: 1.}.to_mesh();
    ///     let RenderableMesh{positions, normals, indices} = mesh.to_renderable_mesh();
    ///     let mut bevy_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    ///     bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    ///     bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    ///     bevy_mesh.set_indices(Some(Indices::U32(indices)));
    ///     bevy_mesh
    /// }
    ///
    /// ```
    pub fn to_renderable_mesh(&self) -> RenderableMesh {
        // let map = HashMap::new();
        // map.insert(0.1, 0.2);
        let vertices = self.vertices().collect::<Vec<_>>();
        let mut indices_count = 0;
        let mut indices = vec![];
        let mut positions = vec![];
        let mut normals = vec![];
        for tri in self.triangles() {
            let v0 = vertices[tri.0];
            let v1 = vertices[tri.1];
            let v2 = vertices[tri.2];
            let n = (*v1 - *v0).cross(*v2 - *v0).normalize();
            positions.push(v0.to_array());
            positions.push(v1.to_array());
            positions.push(v2.to_array());
            normals.push(n.to_array());
            normals.push(n.to_array());
            normals.push(n.to_array());
            indices.push(indices_count);
            indices_count += 1;
            indices.push(indices_count);
            indices_count += 1;
            indices.push(indices_count);
            indices_count += 1;
        }
        RenderableMesh {
            positions,
            normals,
            indices,
        }
    }
}

pub struct RenderableMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

impl<'a> From<(&'a IndexTriangle, [Vec3; 3])> for &'a IndexTriangle {
    fn from(value: (&'a IndexTriangle, [Vec3; 3])) -> Self {
        value.0
    }
}
