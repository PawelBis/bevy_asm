use std::rc::Rc;

use bevy::{asset::LoadedFolder, prelude::*};

/// Asset manager plugin
pub struct BevyAsmPlugin;

#[derive(Resource, Deref, Clone)]
struct LoadedFolderHandle(Handle<LoadedFolder>);

impl Plugin for BevyAsmPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_asset_folder).add_systems(
            Update,
            show_asset_count.run_if(resource_exists::<LoadedFolderHandle>()),
        );
    }
}

fn load_asset_folder(mut commands: Commands, asset_server: Res<AssetServer>) {
    let folder_handle = asset_server.load_folder("");
    commands.insert_resource(LoadedFolderHandle(folder_handle));
}

fn show_asset_count(
    loaded_folder_handle: Res<LoadedFolderHandle>,
    folder_assets: Res<Assets<LoadedFolder>>,
) {
    let lfh = &*loaded_folder_handle;
    if let Some(folder) = folder_assets.get(&lfh.0) {
        println!("Cnt: {}", folder.handles.len());
    };
}
