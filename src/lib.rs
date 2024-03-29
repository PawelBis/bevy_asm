mod asset;
mod db;

use asset::{AssetType, DbAsset};
use bevy::{prelude::*, utils::HashMap};
use rusqlite::Connection;
use std::{env, sync::Mutex};
use thiserror::Error;
use tokio::runtime::Handle;
use walkdir::WalkDir;

#[derive(Debug, Error)]
enum Error {
    #[error("failed to read CARGO_MANIFEST_DIR env var")]
    MissingCargoManifestDir(#[from] std::env::VarError),
    #[error("failed to open SQLite connection")]
    Db(#[from] rusqlite::Error),
    #[error("failed WalkDir iteration: {0}")]
    WalkDir(#[from] walkdir::Error),
    #[error("parent directory missing")]
    ParentDirectoryMissing,
    #[error("filename missing")]
    FilenameMissing,
    #[error("failed 'to_str()'")]
    FiledToStrConversion,
    #[error("assets without extensions are not supported at the moment")]
    AssetWithoutExtension,
    #[error("assets without parent - this shouldn't happen, maybe bug in `WalkDir`")]
    AssetWithoutParentDirectory,
}

#[derive(Resource, Debug)]
struct DbConnection(Mutex<Connection>);

#[derive(Default, Debug, Clone)]
pub enum DbMode {
    /// DB will be created only in the RAM.
    #[default]
    InMemory,
    /// DB will be created in file-system mode. Provided string will
    /// be used as a filename.
    FileSystem(String),
    /// Unsupported
    Cloud,
}

/// This plugin creates an asset database. This db stores asset metadata.
/// Db enables an easy filtering/searching.
///
/// Db can be created as an 'in-memory' database or it can persist data using
/// a file-based storage engine.
#[derive(Clone, Debug, Resource)]
pub struct BevyAsmPlugin {
    pub runtime: Handle,
    pub db_mode: DbMode,
}

impl Plugin for BevyAsmPlugin {
    fn build(&self, app: &mut App) {
        let db_connection = setup_db(self).unwrap();
        app.insert_resource(self.clone())
            .insert_resource(db_connection);
    }
}

fn setup_db(config: &BevyAsmPlugin) -> Result<DbConnection, Error> {
    let manifest_path = env::var("CARGO_MANIFEST_DIR")?;
    // Default asset directory path can be changed with `BEVY_ASSET_ROOT` env var
    let assets_dir = match env::var("BEVY_ASSET_ROOT") {
        Ok(path) => path,
        Err(_) => format!("{manifest_path}/assets"),
    };

    let db = match &config.db_mode {
        DbMode::InMemory => Connection::open_in_memory()?,
        DbMode::FileSystem(name) => {
            let db_path = format!("{}/{}", assets_dir, &name);
            Connection::open(db_path)?
        }
        DbMode::Cloud => unimplemented!(),
    };

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

    Ok(DbConnection(Mutex::new(db)))
}

/// Use `WalkDir` to iterate through assets directory, and populate `directory` and
/// `asset` tables accordingly.
fn scan_fs_for_assets(db: &Connection, assets_dir: String) -> Result<(), Error> {
    let mut dir_to_id = HashMap::<String, u32>::new();
    let mut current_id: u32 = 1;
    for entry in WalkDir::new(assets_dir) {
        let entry = entry?;
        let entry_path = entry.path();
        let entry_filename = entry.file_name().to_str();
        let parent_name = entry_path
            .parent()
            .ok_or(Error::ParentDirectoryMissing)?
            .file_name()
            .ok_or(Error::FilenameMissing)?
            .to_str()
            .ok_or(Error::FiledToStrConversion)?
            .to_owned();
        let parent_id = dir_to_id.get(&parent_name);

        if entry_path.is_dir() {
            let inserted_rows = db.execute(
                "INSERT INTO directory (name, parent) VALUES(?1, ?2);",
                (entry_filename, parent_id),
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
            let extension = entry_path.extension().ok_or(Error::AssetWithoutExtension)?;
            let asset_type = AssetType::from_extension(extension.to_string_lossy().to_string());
            db.execute(
                "INSERT INTO asset (name, path, type, parent_directory) VALUES (?1, ?2, ?3, ?4);",
                (
                    entry_filename,
                    entry_path.to_string_lossy().to_string(),
                    asset_type.to_u32(),
                    parent_id.ok_or(Error::AssetWithoutParentDirectory)?,
                ),
            )?;
        }
    }

    Ok(())
}
