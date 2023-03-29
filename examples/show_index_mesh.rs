use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{Indices, MeshVertexBufferLayout},
        render_resource::{
            AsBindGroup, PolygonMode, PrimitiveTopology, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
    },
};
use mesh::IndexMesh;

use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};

fn main() {
    App::new()
        .insert_resource(Msaa::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(MaterialPlugin::<LineMaterial>::default())
        .add_plugin(LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin::default())
        .add_startup_system(setup)
        .run();
}

fn from_stl() -> IndexMesh {
    let mut f = std::fs::File::open("assets/bunny.stl").unwrap();
    IndexMesh::from_stl(&mut f).unwrap()
}

#[derive(Default, AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "050ce6ac-080a-4d8c-b6b5-b5bab7560d8f"]
struct LineMaterial {
    #[uniform(0)]
    color: Color,
}

impl Material for LineMaterial {
    fn fragment_shader() -> ShaderRef {
        "line_material.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // This is the important part to tell bevy to render this material as a line between vertices
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        Ok(())
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut line_materials: ResMut<Assets<LineMaterial>>,
) {
    let mesh = from_stl();
    commands.spawn(PbrBundle {
        mesh: meshes.add(build_mesh_from_index_mesh(&mesh)),
        material: materials.add(StandardMaterial {
            base_color: Color::RED,
            ..default()
        }),
        ..default()
    });

    let bvh = mesh.build_bvh(Default::default());
    let mut vertices: Vec<Vec3> = vec![];
    for (n, _) in bvh.iter_rand(0) {
        if n.depth == 10 {
            let aabb = n.aabb;
            let a = Vec3::new(aabb.min[0], aabb.min[1], aabb.min[2]);
            let b = Vec3::new(aabb.max[0], aabb.min[1], aabb.min[2]);
            let c = Vec3::new(aabb.max[0], aabb.max[1], aabb.min[2]);
            let d = Vec3::new(aabb.min[0], aabb.max[1], aabb.min[2]);
            let e = Vec3::new(aabb.min[0], aabb.min[1], aabb.max[2]);
            let f = Vec3::new(aabb.max[0], aabb.min[1], aabb.max[2]);
            let g = Vec3::new(aabb.max[0], aabb.max[1], aabb.max[2]);
            let h = Vec3::new(aabb.min[0], aabb.max[1], aabb.max[2]);

            vertices.extend(
                [
                    a, b, b, c, c, d, d, a, e, f, f, g, g, h, h, e, a, e, b, f, c, g, d, h,
                ]
                .iter(),
            );
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::LineList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(mesh),
        material: line_materials.add(LineMaterial {
            color: Color::GREEN,
        }),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(0.0, 20.0, 0.0),
        ..Default::default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.0,
            range: 100.,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 20.0, 0.0),
        ..default()
    });

    // ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(50.0).into()),
        material: materials.add(Color::SILVER.into()),
        ..default()
    });

    commands
        .spawn(Camera3dBundle::default())
        .insert(OrbitCameraBundle::new(
            OrbitCameraController::default(),
            Vec3::new(0.0, 20., 20.0),
            Vec3::new(0., 0., 0.),
            Vec3::Y,
        ));
}

fn build_mesh_from_index_mesh(mesh: &IndexMesh) -> Mesh {
    let mut res = Mesh::new(PrimitiveTopology::TriangleList);
    let vertices = mesh.vertices().collect::<Vec<_>>();
    let mut indices_count = 0;
    let mut indices = vec![];
    let mut positions = vec![];
    let mut normals = vec![];
    for tri in mesh.triangles() {
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
    res.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    res.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    res.set_indices(Some(Indices::U32(indices)));
    res
}
