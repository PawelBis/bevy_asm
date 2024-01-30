mod asset;

use asset::{AssetType, DbAsset};
use bevy::{
    prelude::*,
    utils::{self, HashMap},
};
use rusqlite::Connection;
use std::{env, sync::Mutex};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, Error)]
enum Error {
    #[error("failed to read CARGO_MANIFEST_DIR env var")]
    MissingCargoManifestDir(#[from] std::env::VarError),
    #[error("failed to open SQLite connection")]
    DbError(#[from] rusqlite::Error),
    #[error("parent directory missing")]
    ParentDirectoryMissing,
    #[error("filename missing")]
    FilenameMissing,
    #[error("failed 'to_str()'")]
    FiledToStrConversion,
}

#[derive(Resource, Debug)]
struct DbConnection(Mutex<Connection>);

/// This plugin creates an asset database. This db stores asset metadata.
/// Db enables easy filtering and searching, and for medium scale it should be enough.
pub struct BevyAsmPlugin; // {
                          //pub use_in_memory_db: bool,
                          //};

impl Plugin for BevyAsmPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_db.map(utils::error));
    }
}

fn setup_db(mut commands: Commands) -> Result<(), Error> {
    let manifest_path = env::var("CARGO_MANIFEST_DIR")?;
    let _db_path = format!("{}/{}", manifest_path, "assetdb");
    // Default asset directory path can be changed with `BEVY_ASSET_ROOT` env var
    let assets_dir = match env::var("BEVY_ASSET_ROOT") {
        Ok(path) => path,
        Err(_) => format!("{}/{}", manifest_path, "assets"),
    };

    let db = Connection::open(_db_path)?;
    //let db = Connection::open_in_memory()?;

    AssetType::create_table(&db)?;
    DbAsset::create_table(&db)?;

    // Create asset_type table storing the map
    db.execute(
        r#"
        CREATE TABLE IF NOT EXISTS directory (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            parent INTEGER
        )
        "#,
        (),
    )?;

    scan_fs_for_assets(&db, assets_dir)?;

    commands.insert_resource(DbConnection(Mutex::new(db)));
    Ok(())
}

/// Populate `asset` table with assets found in file system
fn scan_fs_for_assets(db: &Connection, assets_dir: String) -> Result<(), Error> {
    let mut dir_to_id = HashMap::<String, u32>::new();
    let mut current_id: u32 = 1;
    for entry in WalkDir::new(assets_dir) {
        let entry = entry.unwrap();
        let entry_path = entry.path();
        if entry_path.is_dir() {
            let parent_name = entry_path
                .parent()
                .ok_or(Error::ParentDirectoryMissing)?
                .file_name()
                .ok_or(Error::FilenameMissing)?
                .to_str()
                .ok_or(Error::FiledToStrConversion)?
                .to_owned();
            let parent_id = dir_to_id.get(&parent_name);

            let inserted_rows = db.execute(
                "INSERT INTO directory (name, parent) VALUES(?1, ?2);",
                (entry.file_name().to_str(), parent_id),
            )?;

            dir_to_id.insert(
                entry
                    .file_name()
                    .to_str()
                    .ok_or(Error::FiledToStrConversion)?
                    .to_owned(),
                current_id,
            );
            current_id += inserted_rows as u32;
        } else {
        }
    }

    Ok(())
}
