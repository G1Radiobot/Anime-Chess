use std::{f32::consts::PI};

use bevy::{
    ecs::{schedule::SystemSetConfig, system::EntityCommands}, math::*, prelude::*, sprite::*, transform::TransformSystem, ui::update, utils::HashMap, window::{CursorGrabMode, PresentMode, PrimaryWindow, WindowLevel, WindowMode, WindowTheme}
};

use crate::map::*;
use crate::camera::*;
use crate::shared::*;
use bevy::math::bounding::*;

#[derive(Event)]
pub struct MoveDirection(pub Vec3);

#[derive(Event)]
pub struct Rotate(pub f32);

#[derive(Event)]
pub struct Select;

#[derive(Event)]
pub struct UpdateSelectorLocation(pub SelectorLocation);

#[derive(Event)]
pub struct MouseToCursor;

#[derive(Event)]
pub struct MoveSelectorLocation(pub Vec3);

#[derive(Event)]
pub struct Cancel;

pub fn mouse_movement
(
    mut cursor_moved: EventReader<CursorMoved>,
    mut mouse_to_cursor: EventWriter<MouseToCursor>
)
{
    for _event in cursor_moved.read()
    {
        mouse_to_cursor.send(MouseToCursor);
        //println!("mouse_movement mousetocursor fired");
    }
}

pub fn mouse_pos_raycast(
    res: Res<Assets<Mesh>>,
    w_qry: Query<&Window, With<PrimaryWindow>>,
    camera_qry: Query<(&GlobalTransform, &Camera), With<PrimaryCamera>>,
    transform_qry: Query<(&Handle<Mesh>, &Transform), With<IsTile>>,
    mut update_selector_location: EventWriter<UpdateSelectorLocation>,
    mut cursor_moved: EventReader<MouseToCursor>
)
{
    //TODO Split Logic into correct modules - Eventually, the cursor should just appear on hover, and all this should do is trigger a select event
    for _event in cursor_moved.read()
    {
        let mut tile_center = Vec3::ZERO;

        let mouse_ray;
        let (c_trans, camera) = camera_qry.get_single().unwrap();
        let Some(m_pos) = w_qry.single().cursor_position() else {return};
        //let m_pos = event.position;
        let Some(m_ray) = camera.viewport_to_world(c_trans, m_pos) else {return};
        mouse_ray = RayCast3d::from_ray(m_ray, 40.0);
        let mut closest = 100.0;
        

        for (handle, h_trans) in &transform_qry
        {
            let v2 = res.get(handle).unwrap().compute_aabb().unwrap();
            let var = Aabb3d::new(h_trans.translation, v2.half_extents);
            
            if let Some(distance) = mouse_ray.aabb_intersection_at(&var)
            {
                if distance < closest
                {
                    closest = distance;
                    tile_center = h_trans.translation;
                }
            } else {}
        }
        let arrow_vec = m_ray.get_point(closest);
        update_selector_location.send(UpdateSelectorLocation(SelectorLocation{precise_location: arrow_vec, tile_location: tile_center}));
        //println!("update_selector_location fired");
    }    
}

pub fn fire_select
(
    mut select: EventWriter<Select>,
    mut cancel: EventWriter<Cancel>,
    keys: Res<ButtonInput<MouseButton>>
)
{
    if keys.just_pressed(MouseButton::Left)
    {
        select.send(Select);
    }
    if keys.just_pressed(MouseButton::Right)
    {
        cancel.send(Cancel);
    }
}

pub fn get_move_direction(
    keys: Res<ButtonInput<KeyCode>>,
    mut key_pressed: EventWriter<MoveDirection>,
    mut mouse_to_cursor: EventWriter<MouseToCursor>
)
{
    let mut send: Vec3 = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyA){
        send -= Vec3::X;
    }
    if keys.pressed(KeyCode::KeyW){
        send -= Vec3::Z;
    }
    if keys.pressed(KeyCode::KeyD){
        send += Vec3::X;
    }
    if keys.pressed(KeyCode::KeyS){
        send += Vec3::Z;
    };
    if send != Vec3::ZERO
    {
        key_pressed.send(MoveDirection(send));
        mouse_to_cursor.send(MouseToCursor);
        //println!("move_direction mousetocursor fired");
    };

    if keys.just_released(KeyCode::KeyA) || keys.just_released(KeyCode::KeyW) || keys.just_released(KeyCode::KeyD) || keys.just_released(KeyCode::KeyS)
    {
        mouse_to_cursor.send(MouseToCursor);
    }
}

pub fn get_rotation(
    keys: Res<ButtonInput<KeyCode>>,
    mut key_pressed: EventWriter<Rotate>,
    mut mouse_to_cursor: EventWriter<MouseToCursor>
)
{
    let mut send: f32 = 0.0;
    if keys.pressed(KeyCode::KeyQ)
    {
        send += 2.0*PI;
    } else if keys.pressed(KeyCode::KeyE)
    {
        send -= 2.0*PI;
    }
    if send != 0.0
    {
        key_pressed.send(Rotate(send));
        mouse_to_cursor.send(MouseToCursor);
        //println!("get_rotation mousetocursor fired");
    }
    if keys.just_released(KeyCode::KeyQ) || keys.just_released(KeyCode::KeyE)
    {
        mouse_to_cursor.send(MouseToCursor);
    }
}
