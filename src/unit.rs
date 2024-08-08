use std::{f32::consts::PI, time::Duration};

use bevy::{
    ecs::{schedule::SystemSetConfig, system::EntityCommands},
    prelude::*,
    sprite::*,
    transform::TransformSystem,
    utils::{hashbrown::HashMap},
    window::{CursorGrabMode, PresentMode, PrimaryWindow, WindowLevel, WindowMode, WindowTheme},
};

use bevy_sprite3d::*;

use crate::shared::*;
use crate::map::*;

pubify!(#[derive(Component)]
struct Health
{
    max: u32,
    current: u32,
    temp: u32,
});

impl Default for Health {
    fn default() -> Self 
    { Health
        {
            max: 20,
            current: 20,
            temp: 0,
        }
    }
}

/// A location on the map grid.
#[derive(Component, Clone, Copy)]
pub struct Location(pub usize, pub usize);


#[derive(Component, PartialEq, Debug, Clone)]
pub struct UnitID
{
    team: u32,
    id: u32
}

impl Default for UnitID
{
    fn default() -> Self
    { UnitID
        {
            team: 0,
            id: 0,
        }
    }
}

#[derive(Component)]
pub struct Team(pub u32);

#[derive(Component)]
pub struct Movement(pub f32);

#[derive(Component)]
pub struct Sprite(pub String);

#[derive(Component)]
pub struct IsUnit;

#[derive(Component)]
pub struct FaceCamera;

#[derive(Component)]
pub struct TestTimer
{
    timer: Timer
}

// Note: To speed loading this up, I can change "String" to &str, which will make it static. Remember this if I want the extra speed later.
// Todo: Alternatively, just make it generic for type saftey and maximum speed. Not really any reason it can't just be an enum.
// May require deserialization.
/// A library of animations.
#[derive(Component)]
pub struct AnimationLibrary 
{
    animations: HashMap<String, Vec<usize>>,
    current_animation: String,
    //animations: Vec<usize>,
    current_frame: usize,
    timer: Timer,
}

impl AnimationLibrary
{
    fn builder() -> AnimationLibraryBuilder
    { 
        AnimationLibraryBuilder::default()
    }

    pub fn set_animation(&mut self, animation: String)
    {
        self.current_animation = animation;
    }
}

#[derive(Default)]
pub struct AnimationLibraryBuilder
{
    animations: HashMap<String, Vec<usize>>,
    current_animation: String,
    current_frame: usize,
    timer: Timer
}

impl AnimationLibraryBuilder
{
    pub fn new() -> AnimationLibraryBuilder
    {
        let mut me = AnimationLibraryBuilder
        {
            animations: HashMap::new(),
            current_animation: "idle".into(),
            current_frame: 0,
            timer: Timer::new(Duration::from_millis(200), TimerMode::Repeating),
        };
        me.animations.insert("idle".into(), (0..1).collect());
        return me
    }
    ///Pass a hashmap of animations to this to fill the animation library automatically.
    pub fn set_animations(mut self, hash_map: HashMap<String, Vec<usize>>) -> AnimationLibraryBuilder
    {
        for (name, frames) in hash_map
        {
            self.animations.insert(name.clone(), frames.clone());
        };
        self
    }
    ///Set the current animation by name.
    pub fn set_animation(mut self, name: String) -> AnimationLibraryBuilder
    {
        self.current_animation = name;
        self
    }
    ///Set the frame of the current animation, relative to the animation itself, not the index of the complete sprite sheet.
    pub fn set_frame(mut self, frame: usize) -> AnimationLibraryBuilder
    {
        self.current_frame = frame;
        self
    }
    ///Set the timer in milliseconds.
    pub fn set_timer(mut self, millsec: u64) -> AnimationLibraryBuilder
    {
        self.timer = Timer::new(Duration::from_millis(millsec), TimerMode::Repeating);
        self
    }
    ///Set the timer in frames per second.
    pub fn set_frames_per(mut self, frames_per: u16) -> AnimationLibraryBuilder
    {
        let convert = 1000/frames_per;
        self.timer = Timer::new(Duration::from_millis(convert.into()), TimerMode::Repeating);
        self
    }
    pub fn build(self) -> AnimationLibrary
    {
        AnimationLibrary
        {
            animations: self.animations,
            current_animation: self.current_animation,
            current_frame: self.current_frame,
            timer: self.timer,
        }
    }
}

pubify!(#[derive(Bundle)]
struct UnitBundle
{
    is_unit: IsUnit,
    unit_name: ObjName,
    team: Team,
    movement: Movement,
    health: Health,
    loc: Location,
    sprite: Sprite,
    //model: PbrBundle,
});

#[derive(Event)]
pub struct UpdateUnitRenderLocation(Entity);

///Hello!
#[derive(Resource, Default, Clone)]
pub struct SpriteAsset
{
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>
}

#[derive(Resource, Default, Clone)]
pub struct SpriteAssetLibrary(HashMap<String, SpriteAsset>);

pub fn face_camera
(
    cam_query: Query<&Transform, (With<PrimaryCamera>, With<Camera>)>,
    mut query: Query<&mut Transform, (With<FaceCamera>, Without<Camera>)>,
) 
{
    let cam_transform= cam_query.single();
    for mut transform in query.iter_mut() 
    {
        transform.rotation = cam_transform.rotation;
        
        /*
        let mut delta = cam_transform.translation - transform.translation;
        delta.y = 0.0;
        delta += transform.translation;
        transform.look_at(delta, Vec3::Y);
        */
        
    }
}

///Load in unit sprite. Needs to run before init_unit_model.
pub fn init_unit_sprite
(
    mut cmd: Commands,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>
)
{
    let pic: Handle<Image> = asset_server.load("PlayerGuinevere.png");
    let layout: Handle<TextureAtlasLayout> = layouts.add
    (TextureAtlasLayout::from_grid
        (
            UVec2::new(32, 32), 
            4, 
            5,
            None, 
            None
        )
    );
    cmd.insert_resource(SpriteAsset{image: pic, layout});
}


pub fn init_unit_model
(
    mut cmd: Commands, 
    image_server: Res<SpriteAsset>,
    //mut meshs: ResMut<Assets<Mesh>>, 
    //mut materials: ResMut<Assets<StandardMaterial>>,
    mut update_unit_render_location: EventWriter<UpdateUnitRenderLocation>,
    mut sprite_params: Sprite3dParams,
)
{
    let sprite_name: String = "PlayerGuinevere.png".into();
    let atlas = TextureAtlas 
    {
        layout: image_server.layout.clone(),
        index: 0,
    };

    let mut ani_hash: HashMap<String, Vec<usize>> = HashMap::new();
    ani_hash.insert("idle".into(), vec![0,0,0,0,0,1,2,2,2,2,2,1]);
    ani_hash.insert("running_lr".into(), (4..8).collect());
    ani_hash.insert("running_down".into(), (8..12).collect());
    ani_hash.insert("running_up".into(), (12..16).collect());
    ani_hash.insert("selected".into(), vec![16,16,16,16,16,17,18,18,18,18,18,17]);

    let ani_lib: AnimationLibrary = AnimationLibraryBuilder::new()
        .set_animations(ani_hash)
        .set_frames_per(12)
        .set_animation("running_lr".into())
        .build();
    let me = cmd.spawn((UnitBundle
    {
        is_unit: IsUnit,
        unit_name: ObjName("Guinevere".into()),
        team: Team(0),
        movement: Movement(5.0),
        health: Health
        {
            ..default()
        },
        loc: Location(7, 9),
        sprite: Sprite(sprite_name),
        /*
        model: PbrBundle
        {
            mesh: meshs.add(Cuboid::new(0.9, 0.9, 0.9)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(pic),
                ..default()
            }),
            transform: Transform{translation: Vec3::new(0.0, 1.1, 0.0), rotation: Quat::from_axis_angle(Vec3::Y, PI), scale: Vec3::ONE},
            ..default()
        }
        */
    },
    Sprite3d
    {
        image: image_server.image.clone(),
        pixels_per_metre: 16.,

        ..default()
    }.bundle_with_atlas(&mut sprite_params, atlas),
    FaceCamera,

    AnimationLibrary
    {
        animations: ani_lib.animations,
        current_animation: ani_lib.current_animation,
        current_frame: ani_lib.current_frame,
        timer: ani_lib.timer,
    },

    /*
    TestTimer
    {
        timer: Timer::new(Duration::from_secs(2), TimerMode::Repeating)
    }
    */
    )).id();
    update_unit_render_location.send(UpdateUnitRenderLocation(me));
}

pub fn animate_sprites
(
    time: Res<Time>,
    mut query: Query<(&mut AnimationLibrary, &mut TextureAtlas)>,
) 
{
    for (mut animation, mut atlas) in query.iter_mut() 
    {
        animation.timer.tick(time.delta());

        if animation.timer.just_finished() 
        {
            let ani_length: usize;
            if let Some(cur_ani) = animation.animations.get(&animation.current_animation)
            {
                atlas.index = cur_ani[animation.current_frame];
                ani_length = cur_ani.len();
            } else {panic!("Missing {} animation!", animation.current_animation)};
            animation.current_frame += 1;
            animation.current_frame %= ani_length;
        }
    }
}
/// updates units rendered location to match internal location
/// 
/// todo Probably needs to be replaced with a dedicated rendering/animation module
pub fn update_render_location
(
    mut qry: Query<(&mut Transform, &Location), (With<ObjName>, Changed<Location>)>,
)
{
    for (mut transform, loc) in qry.iter_mut()
    {
        transform.translation = Vec3::new(loc.0 as f32, 1.1, loc.1 as f32)
    }
}

//Just moves all units with TestTimer components one to the right each frame
pub fn test_move_one_right
(
    mut unit_qry: Query<(&mut Location, &mut TestTimer, Entity), With<IsUnit>>,
    time: Res<Time>,
    mut update_unit_render_location: EventWriter<UpdateUnitRenderLocation>
)
{
    for (mut unit_loc, mut test_timer, unit) in unit_qry.iter_mut()
    {
        test_timer.timer.tick(time.delta());

        if test_timer.timer.finished()
        {
            unit_loc.0 += 1;
            update_unit_render_location.send(UpdateUnitRenderLocation(unit));
        }
    }
}

//Watches for changes in internal unit location and synchs the unitmap automatically
//todo Solve interdependcy issues- put into shared, put all components in shared, create "intergration module"?
//todo Move UpdateUnitRenderLocation calls in here so they happen automatically, though it should be replaced
//todo eventually with a dedicated rendering/animation module
pub fn synch_unit_map
(
    unit_qry: Query<(&Location, Entity), (Changed<Location>, With<IsUnit>)>,
    mut unit_map_qry: Query<&mut UnitMap>
)
{
    for (&unit_loc, unit) in &unit_qry
    {
        if let Ok(mut map) = unit_map_qry.get_single_mut()
        {
            map[unit_loc.1][unit_loc.0] = Some(unit);
        } else {panic!("AY! Where'd my unit map go, eh?")}


    }
}

#[cfg(test)]
mod test 
{
    use super::*;

    #[test]
    pub fn test_equal()
    {
        let bob = UnitID
        {
            team: 0,
            id: 1,
        };
        let steve = UnitID
        {
            team: 0,
            id: 1,
        };
        let joe = UnitID
        {
            team: 1,
            id: 1,
        };
        assert_eq!(bob, steve);
        assert_ne!(bob, joe);
    }
    
}
