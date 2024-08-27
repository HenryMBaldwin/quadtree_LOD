use std::f32::consts::TAU;


use bevy::input::mouse::{self, MouseButtonInput, MouseMotion, MouseWheel};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::render::camera;
use bevy::render::mesh::{self, Indices, PrimitiveTopology, SphereKind, SphereMeshBuilder};
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::render::render_asset::RenderAssetUsages;


#[derive(Component)]
struct Character;

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
    speed: f32,
}
//global state of sphere, so modification of the number of subdivisions can be done without losing the current state of the sphere
#[derive(Resource, Clone)]
struct SphereState {
    wireframe: bool,
    //if constant rotation is enabled
    rotating: bool,
    //curreent transform of the sphere
    transform: Transform,
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
        })
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_shape)
        .add_systems(Update, handle_ui_interactions)
        .add_systems(Update, handle_mouse_rotate)
        .add_systems(Update, handle_mouse_scroll)
        .add_systems(Update, track_sphere_state)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    subdivisions: Res<Subdivisions>,
    mut ambient_light: ResMut<AmbientLight>,
    sphere_state: Res<SphereState>
) {
    // Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        Camera,
    ));
 
    //character (cube for now)
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.05, 0.05, 0.05)),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.0, 0.8, 0.2),
                ..Default::default()
            }),
            transform: Transform::from_xyz(0.0, 0.0, 0.5),
            ..Default::default()
        },
        Character,
    ));

    //light
    ambient_light.brightness = 1000.0;

    //spawn initial sphere
    create_geodesic_sphere(&mut commands, &mut meshes, &mut materials, sphere_state.clone(), subdivisions.value);

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
    sphere_state: Res<SphereState>
) {
    for (interaction, mut background_color, increment, decrement) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Check if this is an increment or decrement button
                if increment.is_some() {
                    if subdivisions.value < 15 {
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

                // Remove the old sphere
                for entity in sphere_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }

                // Recreate the geodesic sphere with the new subdivisions
                create_geodesic_sphere(&mut commands, &mut meshes, &mut materials, sphere_state.clone(), subdivisions.value);

                *background_color = BackgroundColor(Color::srgb(0.5, 0.5, 0.5));
            }
            _ => {
                *background_color = BackgroundColor(Color::srgb(0.5, 0.5, 0.5));
            }
        }
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
 //tracks state of the sphere
 fn track_sphere_state( 
    mut sphere_state: ResMut<SphereState>,
    sphere_query: Query<(&Transform, &Sphere)>,
) {

    for (transform, _) in &mut sphere_query.iter() {
        sphere_state.transform = *transform;
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

fn rotate_shape(mut shapes: Query<(&mut Transform, &Rotateable)>, timer: Res<Time>) {
    for (mut transform, shape) in &mut shapes {
        transform.rotate_y(shape.speed * TAU * timer.delta_seconds());
    }
}

