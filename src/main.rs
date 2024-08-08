//use serde::*;
use bevy::{
    ecs::{schedule::SystemSetConfig, system::EntityCommands}, input::InputPlugin, prelude::*, sprite::*, transform::TransformSystem, utils::HashMap, window::{CursorGrabMode, PresentMode, PrimaryWindow, WindowLevel, WindowMode, WindowTheme},
    asset::LoadState
};
//use bevy_flycam::prelude::*;
//use bevy_editor_pls::controls::EditorControls;

mod map;
mod shared;
mod camera;
mod input;
mod unit;


use map::*;
use shared::*;
use camera::*;
use input::*;
use unit::*;

//use bevy_editor_pls::EditorPlugin;
//use bevy_editor_pls::controls;
use bevy_sprite3d::*;
//use bevy_editor_pls_default_windows::hierarchy::picking::EditorRayCastSource;

#[derive(SystemSet, States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState
{
    #[default] BattleMap
}

#[derive(SystemSet, SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(GameState = GameState::BattleMap)]
enum Phase
{
    #[default] Player,
    AI,
}

#[derive(SystemSet, SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(Phase = Phase::Player)]
enum Player
{
    Cutscene,
    Effect,
    #[default] Field,
    Movement,
    ActionMenu,
    Action
}

#[derive(SystemSet, States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum LoadingState
{
    #[default] LoadingSpriteTextures,
    MainLoop,
}

#[derive(SystemSet, States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum PlayerPhaseSubState
{
    #[default] Overview,
    UnitSelected,
}

fn main() -> Result<(), String> {
    App::new()
        .add_plugins
        ((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin 
                        {
                            primary_window: Some
                            (Window 
                                {
                                    title: "I am a window!".into(),
                                    present_mode: PresentMode::AutoVsync,
                                    // Tells wasm to resize the window according to the available canvas
                                    // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                                    prevent_default_event_handling: false,

                                    // resolution: (500., 300.).into(),
                                    mode: WindowMode::Windowed,
                                    ..default()
                                }
                            ),
                            ..default()
                        }
                    ),
            //PlayerPlugin,
            //EditorPlugin::default(),
        ))
        .add_plugins(Sprite3dPlugin)
        .init_state::<LoadingState>()
        .init_state::<GameState>()
            .add_sub_state::<Phase>()
                .add_sub_state::<Player>()

        //System Config Block
        .configure_sets
        (Update, 
            (
            //Player
                Player::Cutscene
                    .run_if(in_state(Player::Cutscene)),
                Player::Effect
                    .run_if(in_state(Player::Effect)),
                Player::Field
                    .run_if(in_state(Player::Field)),
                Player::Movement
                    .run_if(in_state(Player::Movement)),
                Player::ActionMenu
                    .run_if(in_state(Player::ActionMenu)),
                Player::Action
                    .run_if(in_state(Player::Action)),
            //GameState
                GameState::BattleMap
                    .run_if(in_state(GameState::BattleMap))
            )
        )

        //Perform initial loading of sprite textures because Sprite3d needs the textures to be preloaded
        .add_systems
        (OnEnter(GameState::BattleMap), 
            (
            //unit
                init_unit_sprite
                    .run_if(in_state(LoadingState::LoadingSpriteTextures)),
            )
        )

        //Check if the sprite textures are finished loading so we can start the rest of init
        .add_systems
        (Update, 
            (
            //main
                done_load_sprite,
            )
                .run_if(in_state(LoadingState::LoadingSpriteTextures))
        )

        //Init functions that start after textures are finished loading
        //Todo: We probably don't need to wait on most of these other than init_unit_model.
        .add_systems
        (OnEnter(LoadingState::MainLoop), 
            (
            //camera
                default_camera,
            //map
                init_map,
                load_map
                    .after(init_map),
                populate_grid
                    .after(load_map),
            //unit
                init_unit_model,
            //main
                setup_ambient_light
            )
        )

        //This systems block should be for systems that always run while a battle map is loaded.
        .add_systems
        (Update, 
            (
            //main
                player_phase_state_handler,
            //unit
                test_move_one_right,
                update_render_location, 
                synch_unit_map,
                face_camera,
                animate_sprites,
            //input
                mouse_movement
                    .before(mouse_pos_raycast),
                mouse_pos_raycast,
                get_move_direction
                    .before(move_camera)
                    .before(mouse_pos_raycast), 
                get_rotation
                    .before(move_camera)
                    .before(mouse_pos_raycast), 
                fire_select
                    .after(mouse_pos_raycast),
            //map
                render_grid, 
                update_selector_location
                    .before(tile_select),
                tile_select,
                render_selector
                    .after(update_selector_location),
                //Field System Set
                (
                    FIELD_unit_selected,
                )
                    .in_set(Player::Field),
                (
                    movement
                )
                    .in_set(Player::Movement),
                debug_selected_unit,
            //camera
                move_camera
            )
                .run_if(in_state(LoadingState::MainLoop))

        )

        .add_systems
        (OnEnter(Player::Movement), 
            (
                entry_unit_selected,
            )
        )
        
        .add_systems
        (OnEnter(Player::Field),
            (
                entry_unit_selected
                    .run_if(in_state(LoadingState::MainLoop))
            )
        )

        .add_event::<MoveDirection>()
        .add_event::<Rotate>()
        .add_event::<Select>()
        .add_event::<Cancel>()
        .add_event::<UpdateSelectorLocation>()
        .add_event::<UpdateUnitRenderLocation>()
        .add_event::<MouseToCursor>()
        .add_event::<UnitOnTile>()
        //.insert_resource(editor_controls())
        .run();

    Ok(())
}

fn setup_ambient_light(mut ambient_light: ResMut<AmbientLight>) {
    ambient_light.brightness = 600.0;
 }

/*
fn editor_controls() -> EditorControls {
    let mut editor_controls = EditorControls::default_bindings();
    editor_controls.unbind(controls::Action::PlayPauseEditor);

    editor_controls.insert(
        controls::Action::PlayPauseEditor,
        controls::Binding {
            input: controls::UserInput::Single(controls::Button::Keyboard(KeyCode::Escape)),
            conditions: vec![controls::BindingCondition::ListeningForText(false)],
        },
    );

    editor_controls
}
*/


///Check if images in "ImageAsset" are finished loading, and if so, switch the state to the main loop.
fn done_load_sprite
(
    asset_server: Res<AssetServer>,
    assets: Res<SpriteAsset>,
    mut next_state: ResMut<NextState<LoadingState>>
)
{
    if asset_server.get_load_state(assets.image.id()) == Some(LoadState::Loaded)
    {
        next_state.set(LoadingState::MainLoop)
    }
}

fn player_phase_state_handler
(
    current_state: Res<State<Player>>,
    mut next_state: ResMut<NextState<Player>>,
    mut update_sel_unit: EventReader<UnitOnTile>,
    mut cancel: EventReader<Cancel>
)
{
    for event in update_sel_unit.read()
    {
        if event.0.is_some()
        {
            println!("OH GOD MY STATE IS CHANGING AAAAAAA");
            match current_state.get()
            {
                Player::Field => next_state.set(Player::Movement),
                //Player::Movement => println!("Already in movement"),
                _ => println!("Already in farthest possible state or somehow fucked up and ended up in uncreated state")
            }
        } else if *current_state.get() == Player::Field
        {
            println!("Need to change to menu state here")
        }
    }
    //todo Maybe save last state and have cancel just revert to previous state to simplify this a bit. May need to get reworked entirely too as cancel shouldn't always change the state.
    for _event in cancel.read()
    {
        println!("MY STATE IS CHANGING BAAAACK AAAA");
        match current_state.get()
        {
            Player::Movement => next_state.set(Player::Field),
            _ => println!("Already in most basic state or fucked up and in uncreated state")
        }
    }
}


