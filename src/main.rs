use std::f32::consts::TAU;


use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::mouse::{self, MouseButtonInput, MouseMotion, MouseWheel};
use bevy::input::ButtonState;
use bevy::log::tracing_subscriber::fmt::time;
use bevy::math::NormedVectorSpace;
use bevy::prelude::*;
use bevy::render::camera;
use bevy::render::mesh::{self, Indices, PrimitiveTopology, SphereKind, SphereMeshBuilder};
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::render::render_asset::RenderAssetUsages;

const PHI: f32 = 1.61803398875;

#[derive(Component)]
struct Character;

#[derive(Clone)]
struct Triangle{
    //index is pretty much arbitrary but unique, but it is useful for debugging
    index: usize,
    triangle: Triangle3d,
}

#[derive(Component)]
struct Rotateable {
    speed: f32,
}

#[derive(Component)]
struct SubdivisionInput;

#[derive(Component)]
struct SubdivisionIncrement;

#[derive(Component)]
struct SubdivisionDecrement;

#[derive(Component)]
struct Sphere;

#[derive(Component)]
struct Camera;

#[derive(Resource)]
struct Subdivisions {
    value: usize,
}

#[derive(Resource)]
struct MouseState {
    dragging: bool,
}

#[derive(Resource)]
struct CharacterState {
    //true position on unit sphere
    center: Vec3,
    //projected position onto nearest triangle
    visual_center: Vec3,
    //local forward vector
    forward: Vec3,
    //local up vector
    up: Vec3,
    //right direction
    right: Vec3,
    //id of the closest triangle
    current_triangle_id: usize,

}

//global state of sphere, so modification of the number of subdivisions can be done without losing the current state of the sphere
#[derive(Resource, Clone)]
struct SphereState {
    wireframe: bool,
    //if constant rotation is enabled
    rotating: bool,
    //current transform of the sphere
    transform: Transform,
    //list of triangles
    triangles: Vec<Triangle>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WireframePlugin)
        .insert_resource(Subdivisions { value: 0 })
        .insert_resource(MouseState {
            dragging: false
        })
        .insert_resource(SphereState {
            wireframe: false,
            rotating: false,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            triangles: Vec::new(),
        })
        .insert_resource(CharacterState { 
            center: Vec3::Z,
            visual_center: Vec3::Z,
            current_triangle_id: 0, 
            forward: Vec3::Y,
            right: Vec3::Y.cross(Vec3::Z),
            up: Vec3::Z,})
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_shape)
        .add_systems(Update, handle_ui_interactions)
        .add_systems(Update, handle_mouse_rotate)
        .add_systems(Update, handle_mouse_scroll)
        .add_systems(Update, track_sphere_state)
        .add_systems(Update, handle_character_movement)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    subdivisions: Res<Subdivisions>,
    mut ambient_light: ResMut<AmbientLight>,
    mut sphere_state: ResMut<SphereState>
) {
    // Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        Camera,
    ));
 
    //character (cube for now)
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.02, 0.02, 0.02)),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.0, 0.8, 0.2),
                ..Default::default()
            }),
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            ..Default::default()
        },
        Character,
    ));

    //light
    ambient_light.brightness = 1000.0;

    //spawn initial sphere
    //create_geodesic_sphere(&mut commands, &mut meshes, &mut materials, sphere_state.clone(), subdivisions.value);
    create_geodesic_sphere_tri(&mut commands, &mut meshes, &mut materials, sphere_state, asset_server.clone(), subdivisions.value);

    // UI setup
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(30.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            right: Val::Px(0.0),
            top: Val::Auto, // or default for top positioning
            bottom: Val::Auto, // or default for bottom positioning
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        background_color: BackgroundColor(Color::NONE),
        ..default()
    })
    .with_children(|parent| {
        //text
        parent.spawn((
            TextBundle {
                text: Text::from_section(
                    "Subdivisions",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 15.0,
                        color: Color::WHITE,
                    },
                ),
                ..default()
            },
        ));

        parent.spawn(
            NodeBundle {
                style: Style {
                    width: Val::Px(100.0),
                    height: Val::Px(30.0),
                    position_type: PositionType::Relative,
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    bottom: Val::Auto,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Row,
                    ..default()
                },
                background_color: BackgroundColor(Color::NONE),
                ..default()
            }    
        )
        .with_children(|parent| {
            //subdivision field
            parent.spawn((
                TextBundle {
                    text: Text::from_section( 
                        format!("{}", subdivisions.value),
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 15.0,
                            color: Color::WHITE,
                        }),
                    style: Style {
                       right: Val::Px(5.0),
                       ..Default::default()
                    },
                    ..default()
                },
                SubdivisionInput,
            )
            );

            parent.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(15.0),
                        height: Val::Px(15.0),
                        margin: UiRect::all(Val::Px(1.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
                    ..default()
                },
                SubdivisionIncrement,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "+",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 15.0,
                        color: Color::WHITE,
                    }
                ));
            });

            //decrement button
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(15.0),
                        height: Val::Px(15.0),
                        margin: UiRect::all(Val::Px(1.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
                    ..default()
                },
                SubdivisionDecrement,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "-",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 15.0,
                        color: Color::WHITE,
                    }
                ));
            });
        });
    });
}


fn handle_character_movement(
    mut character_state: ResMut<CharacterState>,
    mut character_query: Query<(&Character, &mut Transform)>,
    sphere_state: Res<SphereState>,
    time: Res<Time>,
) {

    
    
    let dt = time.delta_seconds();

    let turn_rate = 0.0;
    let speed = 0.1;
    for (_, mut transform) in &mut character_query {
        
        //apply sphere transform to character
        transform.translation = sphere_state.transform.rotation.mul_vec3(character_state.center);
        character_state.center = transform.translation;

        //recalc up
        character_state.up = character_state.center.normalize();

        //apply sphere transform to forward and right
        character_state.forward = sphere_state.transform.rotation.mul_vec3(character_state.forward);
        character_state.right = sphere_state.transform.rotation.mul_vec3(character_state.right);

        // Update position
        transform.translation =  (character_state.center + character_state.forward * speed * dt).normalize();
        character_state.center = transform.translation;

        // Update forward direction
        character_state.forward = (character_state.forward - transform.translation * speed * dt -   character_state.right * turn_rate * dt).normalize();

        // Update right direction
        character_state.right = (character_state.right + character_state.forward * turn_rate * dt).normalize();

        //correct orthogonality and normalize vectors
        character_state.forward = (character_state.forward - character_state.up.dot(character_state.forward) * character_state.up).normalize();
        character_state.right = (character_state.right - character_state.up.dot(character_state.right) * character_state.up).normalize();
        character_state.right = (character_state.right - character_state.forward.dot(character_state.right) * character_state.forward).normalize();
        character_state.up = character_state.center.normalize();
    }
}


fn handle_ui_interactions(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&SubdivisionIncrement>, Option<&SubdivisionDecrement>),
        Changed<Interaction>,
    >,
    mut subdivisions: ResMut<Subdivisions>,
    mut text_query: Query<&mut Text, With<SubdivisionInput>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sphere_query: Query<Entity, With<Sphere>>,
    mut sphere_state: ResMut<SphereState>,
    asset_server: Res<AssetServer>,
) {
    let old_subdivisions = subdivisions.value;
    for (interaction, mut background_color, increment, decrement) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Check if this is an increment or decrement button
                if increment.is_some() {
                    if subdivisions.value < 6 {
                        subdivisions.value += 1;
                    }
                } else if decrement.is_some() {
                    if subdivisions.value > 0 {
                        subdivisions.value -= 1;
                    }
                }

                // Update the displayed text
                if let Ok(mut text) = text_query.get_single_mut() {
                    text.sections[0].value = format!("{}", subdivisions.value);
                }

                // Remove the old sphere if subdivisions have changed
                if old_subdivisions != subdivisions.value {
                    for entity in sphere_query.iter() {
                        commands.entity(entity).despawn_recursive();
                    }
                }

                *background_color = BackgroundColor(Color::srgb(0.5, 0.5, 0.5));
            }
            _ => {
                *background_color = BackgroundColor(Color::srgb(0.5, 0.5, 0.5));
            }
        }
    }

    //if subdivisions have changed, create new sphere
    if subdivisions.value != old_subdivisions {
        //create_geodesic_sphere(&mut commands, &mut meshes, &mut materials, sphere_state, subdivisions.value);
        create_geodesic_sphere_tri(&mut commands, &mut meshes, &mut materials, sphere_state, asset_server.clone(), subdivisions.value);
    }
}

fn handle_mouse_rotate(
    mut mouse_state: ResMut<MouseState>,
    mut mousebtn_evr: EventReader<MouseButtonInput>,
    mut mousemov_evr: EventReader<MouseMotion>,
    mut sphere_query: Query<(&mut Transform, &Sphere)>,
) { 

    //handle rotation state
    for event in mousebtn_evr.read() {
        match event.button {
            MouseButton::Left => {
                match event.state {
                    ButtonState::Pressed => {
                        mouse_state.dragging = true;
                    }
                    ButtonState::Released => {
                        mouse_state.dragging = false;
                    }
                }
            }
            _ => {}
        }
    }

    //handle rotation
    for event in mousemov_evr.read() {
        let MouseMotion { delta } = event;
        
        if mouse_state.dragging {
            for (mut transform, _) in &mut sphere_query {
                transform.rotate_x(delta.y * 0.01);
                transform.rotate_y(delta.x * 0.01);
            }
        }
    }
}


 
 fn handle_mouse_scroll(
    mut mousescroll_evr: EventReader<MouseWheel>,
    mut camera_query: Query<(&mut Transform, &Camera)>,
 ) {
    for event in mousescroll_evr.read() {
        let MouseWheel { unit: _, y, x: _, window: _ } = event;
        for (mut transform, _) in &mut camera_query {
            transform.translation.z -= y * 0.1;
        }
    }
 }

 //mut gets triangles from sphere

 //tracks state of the sphere
 fn track_sphere_state( 
    mut sphere_state: ResMut<SphereState>,
    mut sphere_transform_query: Query<(&Transform, &Sphere)>,
) {

    //track transform of sphere
    for (transform, _) in &mut sphere_transform_query {
        sphere_state.transform = *transform;
        for triangle in &mut sphere_state.triangles {
            triangle.triangle = Triangle3d::new(
                transform.rotation.mul_vec3(triangle.triangle.vertices[0]),
                transform.rotation.mul_vec3(triangle.triangle.vertices[1]),
                transform.rotation.mul_vec3(triangle.triangle.vertices[2]),
            );
        }
        
    }
}

//track character state
fn track_character_state(
    mut character_state: ResMut<CharacterState>,
    mut character_query: Query<(&Character, &Transform)>,
) {
    for (_, transform) in &mut character_query {
        character_state.center = transform.translation;
    }
}

//make sure the character rotates with the sphere
fn rotate_character(
    mut sphere_state: ResMut<SphereState>,
    mut character_query: Query<(&Character, &mut Transform)>,
) {
    //apply the sphere transform to the character
    for (character, mut transform) in &mut character_query {
        *transform = sphere_state.transform;
    }
}

fn create_geodesic_sphere(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>, sphere_state: SphereState,  subdivisions: usize){

    let kind: SphereKind = mesh::SphereKind::Ico {
        subdivisions: subdivisions,
    };
    let radius = 0.5;
    let mesh = SphereMeshBuilder::new(radius, kind).build();

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 1.0, 1.0),
                ..Default::default()
            }), 
            transform: sphere_state.transform, 
            ..Default::default()
        }, 
        Wireframe,
        Rotateable {speed: 0.00},
        Sphere,
    ));
}

fn create_geodesic_sphere_tri(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    mut sphere_state: ResMut<SphereState>,
    asset_server: AssetServer,
    subdivisions: usize,
){

    //define unit sphere vertices for icosahedron
    let mut vertices: Vec<Vec3> = vec![
        Vec3::new(-1.0,  PHI, 0.0).normalize(),
        Vec3::new( 1.0,  PHI, 0.0).normalize(),
        Vec3::new(-1.0, -PHI, 0.0).normalize(),
        Vec3::new( 1.0, -PHI, 0.0).normalize(),

        Vec3::new(0.0, -1.0,  PHI).normalize(),
        Vec3::new(0.0,  1.0,  PHI).normalize(),
        Vec3::new(0.0, -1.0, -PHI).normalize(),
        Vec3::new(0.0,  1.0, -PHI).normalize(),

        Vec3::new( PHI, 0.0, -1.0).normalize(),
        Vec3::new( PHI, 0.0,  1.0).normalize(),
        Vec3::new(-PHI, 0.0, -1.0).normalize(),
        Vec3::new(-PHI, 0.0,  1.0).normalize(),
    ];

    let mut index = 1;
    let mut triangles: Vec<Triangle> = vec![
        Triangle {index: {index.clone()}, triangle: Triangle3d::new(vertices[0], vertices[11], vertices[5])},
        Triangle {index: {index += 1; index.clone()}, triangle: Triangle3d::new(vertices[0],  vertices[5], vertices[1])},
        Triangle {index: {index += 1; index.clone()}, triangle: Triangle3d::new(vertices[0],  vertices[1], vertices[7])},
        Triangle {index: {index += 1; index.clone()}, triangle: Triangle3d::new(vertices[0],  vertices[7], vertices[10])},
        Triangle {index: {index += 1; index.clone()}, triangle: Triangle3d::new(vertices[0],  vertices[10], vertices[11])},

        Triangle {index: {index += 1; index.clone()}, triangle: Triangle3d::new(vertices[1],  vertices[5], vertices[9])},
        Triangle {index: {index += 1; index.clone()}, triangle: Triangle3d::new(vertices[5],  vertices[11], vertices[4])},
        Triangle {index: {index += 1; index.clone()}, triangle: Triangle3d::new(vertices[11],  vertices[10], vertices[2])},
        Triangle {index: {index += 1; index.clone()}, triangle: Triangle3d::new(vertices[10],  vertices[7], vertices[6])},
        Triangle {index: {index += 1; index.clone()}, triangle: Triangle3d::new(vertices[7],  vertices[1], vertices[8])},

        Triangle {index: {index += 1; index.clone()}, triangle: Triangle3d::new(vertices[3],  vertices[9], vertices[4])},
        Triangle {index: {index += 1; index.clone()}, triangle: Triangle3d::new(vertices[3],  vertices[4], vertices[2])},
        Triangle {index: {index += 1; index.clone()}, triangle: Triangle3d::new(vertices[3],  vertices[2], vertices[6])},
        Triangle {index: {index += 1; index.clone()}, triangle: Triangle3d::new(vertices[3],  vertices[6], vertices[8])},
        Triangle {index: {index += 1; index.clone()}, triangle: Triangle3d::new(vertices[3],  vertices[8], vertices[9])},

        Triangle {index: {index += 1; index.clone()}, triangle: Triangle3d::new(vertices[4],  vertices[9], vertices[5])},
        Triangle {index: {index += 1; index.clone()}, triangle: Triangle3d::new(vertices[2],  vertices[4], vertices[11])},
        Triangle {index: {index += 1; index.clone()}, triangle: Triangle3d::new(vertices[6],  vertices[2], vertices[10])},
        Triangle {index: {index += 1; index.clone()}, triangle: Triangle3d::new(vertices[8],  vertices[6], vertices[7])},
        Triangle {index: {index += 1; index.clone()}, triangle: Triangle3d::new(vertices[9],  vertices[8], vertices[1])},
    ];

    //subdivide correct number of times
    for i in 0..subdivisions {
        let (new_vertices, new_triangles) = subdivide(triangles);
        vertices = new_vertices;
        triangles = new_triangles;
    }
    let individual = false;
    //create each triangle mesh individually
    if individual {
        for triangle in triangles.clone() {
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![triangle.triangle.vertices[0], triangle.triangle.vertices[1], triangle.triangle.vertices[2]]);
            mesh.insert_indices(Indices::U32(vec![0, 1, 2]));
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(mesh),
                    material: materials.add(StandardMaterial {
                        base_color: Color::srgb(1.0, 1.0, 1.0),
                        ..Default::default()
                    }), 
                    transform: sphere_state.transform, 
                    ..Default::default()
                }, 
                Wireframe,
                Sphere,
            ));
        }
    }
    else {
        //create one mesh with all triangles
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        let mut positions: Vec<Vec3> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        for triangle in triangles.clone() {
            positions.push(triangle.triangle.vertices[0]);
            positions.push(triangle.triangle.vertices[1]);
            positions.push(triangle.triangle.vertices[2]);
            indices.push(indices.len() as u32);
            indices.push(indices.len() as u32);
            indices.push(indices.len() as u32);
        }
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_indices(Indices::U32(indices));
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 1.0, 1.0),
                    ..Default::default()
                }), 
                transform: sphere_state.transform, 
                ..Default::default()
            }, 
            Wireframe,
            Sphere,
        ));
    }

    //add transform to triangles
    for triangle in &mut triangles {
        triangle.triangle = Triangle3d::new(
            sphere_state.transform.rotation.mul_vec3(triangle.triangle.vertices[0]),
            sphere_state.transform.rotation.mul_vec3(triangle.triangle.vertices[1]),
            sphere_state.transform.rotation.mul_vec3(triangle.triangle.vertices[2]),
        );
    }
    //add triangles to sphere state
    sphere_state.triangles = triangles;
}

fn subdivide(triangles: Vec<Triangle>) -> (Vec<Vec3>, Vec<Triangle>) {
    let mut new_vertices: Vec<Vec3> = Vec::new();
    let mut new_triangles: Vec<Triangle> = Vec::new();
    
        for triangle in triangles {

            //get vertices of triangle
            let a = triangle.triangle.vertices[0];
            let b = triangle.triangle.vertices[1];
            let c = triangle.triangle.vertices[2];

            //get new vertices and normalize
            let ab = a.midpoint(b).normalize();
            let bc = b.midpoint(c).normalize();
            let ca =  c.midpoint(a).normalize();

            new_vertices.push(a);
            new_vertices.push(b);
            new_vertices.push(c);
            new_vertices.push(ab);
            new_vertices.push(bc);
            new_vertices.push(ca);
            
            let mut index = 1;
            new_triangles.push(Triangle {index: {index += 1; index.clone()}, triangle: Triangle3d::new(a, ab, ca)});
            new_triangles.push(Triangle {index: {index += 1; index.clone()}, triangle: Triangle3d::new(b, bc, ab)});
            new_triangles.push(Triangle {index: {index += 1; index.clone()}, triangle: Triangle3d::new(c, ca, bc)});
            new_triangles.push(Triangle {index: {index += 1; index.clone()}, triangle: Triangle3d::new(ab, bc, ca)});
        }

    (new_vertices, new_triangles)
}

//dynamic texture generation
fn generate_triangle_index_texture(triangles: Vec<Triangle>) -> Vec<u8> {
    let mut texture: Vec<u8> = Vec::new();
    for triangle in triangles {
        texture.push(triangle.index as u8);
    }
    texture
}


fn rotate_shape(mut shapes: Query<(&mut Transform, &Rotateable)>, timer: Res<Time>) {
    for (mut transform, shape) in &mut shapes {
        transform.rotate_y(shape.speed * TAU * timer.delta_seconds());
    }
}

