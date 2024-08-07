use std::f32::consts::PI;


use bevy::{
    ecs::{schedule::SystemSetConfig, system::EntityCommands},
    prelude::*,
    sprite::*,
    math::f32::Quat,
    transform::TransformSystem,
    utils::HashMap,
    window::{CursorGrabMode, PresentMode, PrimaryWindow, WindowLevel, WindowMode, WindowTheme},
    render::camera::*,
};

use crate::input::*;
use crate::shared::*;

pub fn move_camera(
    mut qry: Query<(&PrimaryCamera, &mut Transform)>, 
    mut translate_camera: EventReader<MoveDirection>,
    mut rotate_camera: EventReader<Rotate>,
    //mut cursor_moved: EventWriter<MouseToCursor>
)
{
    let (camera, mut transform) = qry.single_mut();
    for event in translate_camera.read()
    {
        
        let mut trans = transform.translation;
        let mut delta = transform.rotation;
        delta *= Quat::from_axis_angle(Vec3::X, PI/4.0);
        trans += (delta * (event.0/3.0));
        transform.translation = trans;
        //println!("{:?}", transform.translation);
        //println!("Camera Translated");
        
    }
    for event in rotate_camera.read()
    {
        
        let mut rot = transform.rotation;
        rot *= Quat::from_axis_angle(rot.inverse() * Vec3::Y, event.0/240.0);
        transform.rotation = rot;
        //println!("CameraRotated")
        //println!("{:?}", transform.rotation);
        //println!("Rotate Camera {:?}", event.0);
    }
    
    //cursor_moved.send(MouseToCursor);
}

pub fn default_camera(mut cmd: Commands)
{
    let mut temp = cmd.spawn((Camera3dBundle
    {
        transform: Transform::from_xyz(11.5, 9.0, 17.0).with_rotation(Quat::from_axis_angle(Vec3::Y, 0.0) * Quat::from_axis_angle(Vec3::X, -PI/4.0)),
        
        /*
        projection: Projection::Orthographic(OrthographicProjection
        {
            scaling_mode: ScalingMode::WindowSize(100.0),
            ..default()
        }),
        */
        
        ..default()
    },

    PrimaryCamera()));
}
