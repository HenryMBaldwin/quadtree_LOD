use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn(Camera3dBundle {
    transform: Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });


    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 3000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 4.0, 4.0),
        ..Default::default()
    });

    // Create the icosahedron mesh
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());

    // Vertices of a unit icosahedron
    let vertices = vec![
        [-1.0, 0.0, 1.618], [1.0, 0.0, 1.618], [-1.0, 0.0, -1.618], [1.0, 0.0, -1.618],
        [0.0, 1.618, 1.0], [0.0, 1.618, -1.0], [0.0, -1.618, 1.0], [0.0, -1.618, -1.0],
        [1.618, 1.0, 0.0], [1.618, -1.0, 0.0], [-1.618, 1.0, 0.0], [-1.618, -1.0, 0.0],
    ];

    // Indices for drawing triangles
    let indices = vec![
        0, 6, 1, 0, 1, 4, 0, 4, 10, 0, 10, 11, 0, 11, 6, 
        1, 6, 9, 6, 11, 7, 6, 7, 9, 1, 9, 8, 1, 8, 4, 
        4, 8, 5, 4, 5, 10, 10, 5, 3, 10, 3, 11, 11, 3, 7, 
        7, 3, 2, 7, 2, 9, 9, 2, 8, 8, 2, 5, 5, 2, 3,
    ];

    let normals: Vec<[f32; 3]> = vertices.iter().map(|v| {
        let len: f32 = ((v[0] * v[0] + v[1] * v[1] + v[2] * v[2]) as f32).sqrt();
        [v[0] / len, v[1] / len, v[2] / len]
    }).collect();

    // Add data to the mesh
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));

    // Spawn the mesh with a material
    commands.spawn(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.3, 0.5, 0.3),
            ..Default::default()
        }),
        ..Default::default()
    });
}
