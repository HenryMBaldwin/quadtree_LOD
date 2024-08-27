use std::f32::consts::TAU;


use bevy::prelude::*;
use bevy::render::mesh::{self, Indices, PrimitiveTopology, SphereKind, SphereMeshBuilder};
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::render::render_asset::RenderAssetUsages;

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

#[derive(Resource)]
struct Subdivisions {
    value: usize,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WireframePlugin)
        .insert_resource(Subdivisions { value: 0 })
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_shape)
        .add_systems(Update, handle_ui_interactions)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    subdivisions: Res<Subdivisions>,
    mut ambient_light: ResMut<AmbientLight>,
) {
    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
 

    ambient_light.brightness = 1000.0;
    //spawn initial sphere
    create_geodesic_sphere(&mut commands, &mut meshes, &mut materials, subdivisions.value);

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
                        margin: UiRect::all(Val::Px(5.0)),
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
                        margin: UiRect::all(Val::Px(5.0)),
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
) {
    for (interaction, mut background_color, increment, decrement) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Check if this is an increment or decrement button
                if increment.is_some() {
                    if subdivisions.value < 10 {
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
                create_geodesic_sphere(&mut commands, &mut meshes, &mut materials, subdivisions.value);

                *background_color = BackgroundColor(Color::srgb(0.5, 0.5, 0.5));
            }
            _ => {
                *background_color = BackgroundColor(Color::srgb(0.5, 0.5, 0.5));
            }
        }
    }
}

fn create_geodesic_sphere(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>, subdivisions: usize){

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
            ..Default::default()
        }, 
        Wireframe,
        Rotateable {speed: 0.3},
        Sphere,
    ));
}

fn rotate_shape(mut shapes: Query<(&mut Transform, &Rotateable)>, timer: Res<Time>) {
    for (mut transform, shape) in &mut shapes {
        transform.rotate_y(-shape.speed * TAU * timer.delta_seconds());
    }
}
