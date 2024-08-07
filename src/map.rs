use bevy::{
    color::palettes::{css::{BLACK, BLUE, GHOST_WHITE, GREEN, RED}, tailwind::{GREEN_400, GREEN_950}}, ecs::{schedule::SystemSetConfig, system::EntityCommands}, prelude::*, sprite::*, transform::TransformSystem, utils::HashMap, window::{CursorGrabMode, PresentMode, PrimaryWindow, WindowLevel, WindowMode, WindowTheme}
};

use std::f32::consts::PI;

use crate::{shared::*, Player, Select, UnitID, UpdateSelectorLocation};
use crate::unit::*;

pub struct Tile
{
    name: String,
    id: u32,
    mv_cost: f32,
    rand_info: Color,
}

#[derive(Component)]
pub struct MapSize(pub usize, pub usize);

#[derive(Component)]
pub struct TileMap(Vec<Vec<u32>>);

#[derive(Component)]
pub struct TileList(HashMap<u32, Tile>);

#[derive(Component, Deref, DerefMut)]
pub struct UnitMap(Vec<Vec<Option<Entity>>>);

#[derive(Component, Default, Clone, Copy)]
pub struct SelectedUnit
{
    selected_unit: Option<Entity>,
    selected_loc: Option<Location>,
    targeted_unit: Option<Entity>,
    targeted_loc: Option<Location>
}

#[derive(Event)]
pub struct UnitOnTile(pub Option<Entity>, pub Option<Location>);

#[derive(Component)]
pub struct IsTile();

#[derive(Bundle)]
pub struct MapBundle
{
    map_name: ObjName,
    map_size: MapSize,
    tile_map: TileMap,
    unit_map: UnitMap,
    tile_list: TileList,
    selected_unit: SelectedUnit
}

pub fn init_map(mut cmd: Commands)
{
    cmd.spawn(MapBundle
    {
        map_name: ObjName("test".into()),
        map_size: MapSize(24, 17),
        tile_map: TileMap(vec![vec![1; 24]; 17]),
        unit_map: UnitMap(vec![vec![None; 24]; 17]),
        tile_list: TileList(HashMap::new()),
        selected_unit: SelectedUnit
        {
            ..default()
        }
    });
    cmd.spawn(
    {
        SelectorLocation{precise_location: Vec3::ZERO, tile_location: Vec3::ZERO}
    });
}

pub fn load_map(mut qry: Query<(&MapSize, &mut TileMap, &mut TileList)>)
{
    let (map_size, mut tile_map, mut tile_list) = qry.single_mut();
    tile_map.0 = 
    vec![
        vec![0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        vec![0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        vec![0,0,1,1,0,0,0,0,3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        vec![0,0,1,1,0,0,0,0,3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        vec![0,0,1,1,0,0,0,0,3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        vec![0,0,1,1,0,0,0,0,3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        vec![0,0,1,1,0,0,0,0,3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        vec![0,0,1,0,0,0,0,0,3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        vec![0,0,0,2,2,2,2,2,2,2,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        vec![0,0,0,2,2,2,2,2,2,2,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    ];
    tile_list.0.insert(0, Tile
    {
        name: "Plain".into(),
        id: 0,
        mv_cost: 1.0,
        rand_info: Color::Srgba(GREEN),
    });
    tile_list.0.insert(1, Tile
    {
        name: "Forest".into(),
        id: 1,
        mv_cost: 1.5,
        rand_info: Color::Srgba(GREEN_400),
    });
    tile_list.0.insert(2, Tile
    {
        name: "Road".into(),
        id: 2,
        mv_cost: 0.8,
        rand_info: Color::Srgba(GHOST_WHITE),
    });
    tile_list.0.insert(3, Tile
    {
        name: "Cliff".into(),
        id: 3,
        mv_cost: 99.0,
        rand_info: Color::Srgba(BLACK),
    });
}

pub fn render_grid(mut giz: Gizmos)
{
    
    for z in 0..18
    {
        giz.ray(Vec3::new(-0.5, 0.61, -0.5 + z as f32), Vec3::new(24.0, 0.0, 0.0), Color::Srgba(RED))
    }
    for x in 0..25
    {
        giz.ray(Vec3::new(-0.5 + x as f32,0.61,-0.5), Vec3::new(0.0,0.0,17.0), Color::Srgba(RED))
    }
}

pub fn populate_grid(
    mut cmd: Commands, 
    asset_server: Res<AssetServer>,
    mut meshs: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut qry: Query<(&TileMap, &TileList)>)
{
    let mut z = 0;
    
    let h = meshs.add(Cuboid::new(1.0, 0.2, 1.0));
    let (col, tile_list) = qry.single();
    for column in &col.0
    {
        let mut x = 0;
        for id in column
        {
            cmd.spawn((PbrBundle
            {
                mesh: h.clone(),
                material: materials.add(StandardMaterial
                    {
                        base_color: tile_list.0[id].rand_info,
                        ..default()
                    }),
                transform: Transform::from_xyz(x as f32, 0.5, z as f32),
                ..default()
            },
            IsTile(),
            ));
            x += 1;
        }
        z += 1;
    }
    cmd.spawn(PointLightBundle
        {
            point_light: PointLight 
            {
                shadows_enabled: true,
                intensity: 10.0,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..default()
        });
}

pub fn update_selector_location(
    mut qry: Query<&mut SelectorLocation>,
    mut update_selector_loc: EventReader<UpdateSelectorLocation>
)
{
    let mut selector_loc = qry.single_mut();
    for event in update_selector_loc.read()
    {
        *selector_loc = event.0;
        //println!("update_selector_location consumed");
    }
}

pub fn render_selector(
    mut gizmo: Gizmos,
    qry: Query<&SelectorLocation>
)
{
    let selector_loc = qry.single();
    let p_loc = selector_loc.precise_location;
    let t_loc = selector_loc.tile_location;
    gizmo.arrow((Vec3::Y * 2.0) + p_loc, p_loc, Color::Srgba(BLUE));
        
    //println!("Made it to here.");
    //gizmo.arrow((Vec3::Y * 2.0) + Vec3::from(h_trans.translation), Vec3::from(h_trans.translation), Color::RED).with_tip_length(0.0);
    let mut x = 0.0;
    while x < 3.0
    {
        x += 1.0;
        gizmo.rect((Vec3::Y * (0.2 + (x * 0.15))) + Vec3::from(t_loc), Quat::from_axis_angle(Vec3::X, PI/2.0), Vec2::ONE, Color::Srgba(BLUE));
    }
}

pub fn tile_select
(
    map_qry: Query<(&TileMap, &TileList, &MapSize, &UnitMap)>,
    sel_qry: Query<&SelectorLocation>,
    mut sel_loc: EventReader<Select>,
    mut unit_on_tile: EventWriter<UnitOnTile>
)
{
    let (tile_map, tile_list, map_size, unit_map) = map_qry.single();
    let selector_loc = sel_qry.single();

    for _event in sel_loc.read()
    {
        let x = (selector_loc.precise_location.x + 0.5) as usize;
        let z = (selector_loc.precise_location.z + 0.5) as usize;

        if x >= map_size.0 || z >= map_size.1
        {
            break;
        }

        println!("{} {} {}", x, z, tile_list.0[&tile_map.0[z][x]].name);

        if let Some(unit) = unit_map[z][x]
        {
            unit_on_tile.send(UnitOnTile(Some(unit), Some(Location(x, z))));
        } else 
        {
            unit_on_tile.send(UnitOnTile(None, None));
        }
    }
}

pub fn FIELD_unit_selected
(
    mut update_selected_unit: EventReader<UnitOnTile>,
    mut sel_qry: Query<&mut SelectedUnit>
)
{
    let mut selected_unit = sel_qry.single_mut();
    for event in update_selected_unit.read()
    {
        if let (Some(sel_unit), Some(sel_unit_loc)) = (event.0, event.1)
        {
            selected_unit.selected_unit = Some(sel_unit);
            selected_unit.selected_loc = Some(sel_unit_loc);
        } else
        {
            println!("Here's where I'd open my menu- IF I HAD ONE!")
        }
    }
}

pub fn movement
(
    //mut unit_qry: Query<>
)
{

}

pub fn entry_unit_selected
(
    mut unit_qry: Query<&mut AnimationLibrary, With<IsUnit>>,
    sel_unit_qry: Query<&SelectedUnit>,
    cur_state: Res<State<Player>>
)
{
    if let Some(sel_unit) = sel_unit_qry.single().selected_unit
    {
        if let Ok(mut unit_ani_lib) = unit_qry.get_mut(sel_unit)
        {
            match cur_state.get()
            {
                Player::Movement => unit_ani_lib.set_animation("selected".into()),
                Player::Field => unit_ani_lib.set_animation("idle".into()),
                _ => panic!("In map. Game in incomplete state.")
            }
        } else {panic!("Literally how. entity has no animation library?")}
    } else {panic!("Entity doesn't exsist. unit_selected_entry")}
}

pub fn debug_selected_unit
(
    sel_unit_qry: Query<&SelectedUnit, Changed<SelectedUnit>>,
    unit_qry: Query<&ObjName, With<IsUnit>>
)
{
    for unit_selected in &sel_unit_qry
    {
        if let Some(sel_unit) = unit_selected.selected_unit
        {
            if let Ok(unit_name) = unit_qry.get(sel_unit)
            {
                println!("Selected unit is {}", unit_name.0)
            } else {panic!("Somehow no name?!?!")}
        } else {println!("Selected Unit has been cleared.")}
    }
}