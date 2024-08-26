use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::render::render_asset::RenderAssetUsages;


#[derive(Component)]
struct Rotateable {
    speed: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WireframePlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_shape)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    // Light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(-0.5, 0.5, -2.5).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });    

    // Set the number of subdivisions
    let subdivisions = 1;

    // Create the octahedron mesh and subdivide it
    let mesh = generate_geodesic_sphere(subdivisions);

    // Spawn the mesh with a material and the wireframe component
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 1.0, 1.0),
                ..Default::default()
            }),
            transform: Transform::from_scale(Vec3::splat(0.5)), // Scale down
            ..Default::default()
        }, 
        Wireframe,
        Rotateable {speed: 0.3}, // Add the Wireframe component
));
}

// Function to create an octahedron and subdivide it
fn generate_geodesic_sphere(subdivisions: usize) -> Mesh {
    let mut vertices = vec![
        // Top vertex
        [0.0, 1.0, 0.0],
        // Bottom vertex
        [0.0, -1.0, 0.0],
        // Around the equator
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        [-1.0, 0.0, 0.0],
        [0.0, 0.0, -1.0],
    ];

    let mut indices = vec![
        0, 2, 3,
        0, 3, 4,
        0, 4, 5,
        0, 5, 2,
        1, 3, 2,
        1, 4, 3,
        1, 5, 4,
        1, 2, 5,
    ];

    for _ in 0..subdivisions {
        subdivide(&mut vertices, &mut indices);
    }

    // Normalize vertices to make it a sphere
    vertices.iter_mut().for_each(|v| {
        let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        v[0] /= len;
        v[1] /= len;
        v[2] /= len;
    });

    let normals = vertices.clone(); // Since the vertices are on the unit sphere, the normals are the same

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

// Function to subdivide each triangle into 4 smaller triangles
fn subdivide(vertices: &mut Vec<[f32; 3]>, indices: &mut Vec<u32>) {
    let mut new_indices = vec![];

    for chunk in indices.chunks(3) {
        let v0 = chunk[0] as usize;
        let v1 = chunk[1] as usize;
        let v2 = chunk[2] as usize;

        let midpoint = |i1: usize, i2: usize| -> [f32; 3] {
            let v1 = vertices[i1];
            let v2 = vertices[i2];
            [
                (v1[0] + v2[0]) * 0.5,
                (v1[1] + v2[1]) * 0.5,
                (v1[2] + v2[2]) * 0.5,
            ]
        };

        let m01 = midpoint(v0, v1);
        let m12 = midpoint(v1, v2);
        let m20 = midpoint(v2, v0);

        let i_m01 = vertices.len() as u32;
        let i_m12 = (vertices.len() + 1) as u32;
        let i_m20 = (vertices.len() + 2) as u32;

        vertices.push(m01);
        vertices.push(m12);
        vertices.push(m20);

        new_indices.extend_from_slice(&[
            chunk[0], i_m01, i_m20,
            i_m01, chunk[1], i_m12,
            i_m12, chunk[2], i_m20,
            i_m01, i_m12, i_m20,
        ]);
    }

    *indices = new_indices;
}

fn rotate_shape(mut shapes: Query<(&mut Transform, &Rotateable)>, timer: Res<Time>) {
    for (mut transform, shape) in &mut shapes {
        transform.rotate_y(-shape.speed * TAU * timer.delta_seconds());
    }
}
