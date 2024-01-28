mod asset;

use asset::{AssetType, DbAsset};
use bevy::{prelude::*, utils};
use rusqlite::Connection;
use std::{env, sync::Mutex};
use thiserror::Error;

#[derive(Debug, Error)]
enum Error {
    #[error("failed to read CARGO_MANIFEST_DIR env var")]
    MissingCargoManifestDir(#[from] std::env::VarError),
    #[error("failed to open SQLite connection")]
    FailedToOpenSqliteConnection(#[from] rusqlite::Error),
}

#[derive(Resource)]
struct DbConnection(Mutex<Connection>);

/// This plugin creates an asset database. This db stores asset metadata.
/// Db enables easy filtering and searching, and for medium scale it should be enough.
pub struct BevyAsmPlugin;

impl Plugin for BevyAsmPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_db.map(utils::error));
    }
}

fn setup_db(mut commands: Commands) -> Result<(), Error> {
    let manifest_path = env::var("CARGO_MANIFEST_DIR")?;
    let _db_path = format!("{}/{}", manifest_path, "assetdb");
    //let db = Connection::open(db_path)?;
    let db = Connection::open_in_memory()?;

    AssetType::create_table(&db)?;
    DbAsset::create_table(&db)?;

    commands.insert_resource(DbConnection(Mutex::new(db)));
    Ok(())
}
