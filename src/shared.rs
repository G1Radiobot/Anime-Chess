use bevy::{
    ecs::{schedule::SystemSetConfig, system::EntityCommands},
    prelude::*,
    sprite::*,
    transform::TransformSystem,
    utils::HashMap,
    window::{CursorGrabMode, PresentMode, PrimaryWindow, WindowLevel, WindowMode, WindowTheme},
};

macro_rules! pubify {
    {
        $(#[derive($($derive:ident),*)])*
        struct $name:ident {
            $($field:ident: $t:ty,)*
        }
    } => {
        $(#[derive($($derive),*)])*
        pub struct $name {
            $(pub $field: $t),*
        }
    };
    {
        $(#[derive($($derive:ident),*)])*
        struct $name:ident($($t:ty),*)
    } => {
        $(#[derive($($derive),*)])*
        pub struct $name(pub $($t),*);
    }
}

pub(crate) use pubify;

#[derive(Component, Clone, Copy)]
pub struct SelectorLocation
{
    pub precise_location: Vec3,
    pub tile_location: Vec3
}

pubify!(#[derive(Component)]
struct ObjName(String));

#[derive(Component)]
pub struct PrimaryCamera();