use std::any::TypeId;

use super::Error;
use bevy::prelude::Image;
use rusqlite::Connection;
use strum::{Display, IntoEnumIterator};
use strum_macros::EnumIter;

/// Describes type of the asset. Those types are deducted from supported extensions.
#[derive(Display, EnumIter)]
pub enum AssetType {
    /// Type is not known and unsupported
    Untyped,
    /// Image type
    Image,
}

const IMAGE_EXTENSIONS: &[&str] = &["png"];

impl AssetType {
    /// Create a table for a u32 -> AssetType mapping
    pub fn create_table(connection: &Connection) -> Result<(), Error> {
        // Create asset_type table storing the map
        connection.execute(
            r#"
            CREATE TABLE IF NOT EXISTS asset_type (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            )
            "#,
            (),
        )?;

        for asset_type in AssetType::iter() {
            connection.execute(
                "INSERT INTO 'asset_type' (name) VALUES (?1)",
                [&asset_type.to_string()],
            )?;
        }

        Ok(())
    }

    pub fn from_extension(extension: String) -> Self {
        if IMAGE_EXTENSIONS.iter().any(|e| *e == extension) {
            AssetType::Image
        } else {
            AssetType::Untyped
        }
    }
}

impl From<TypeId> for AssetType {
    fn from(value: TypeId) -> Self {
        if value == TypeId::of::<Image>() {
            AssetType::Image
        } else {
            AssetType::Untyped
        }
    }
}

pub struct DbAsset {
    /// Id in the db
    pub id: u32,
    /// Name of the asset - can
    pub name: String,
    /// Path to the asset or it's descriptor
    pub path: Option<String>,
    /// Type of the asset, extension based
    pub asset_type: AssetType,
    /// In what directory is the asset located
    pub parent_directory: u32,
}

impl DbAsset {
    /// Create trable that stores assets metadata
    pub fn create_table(connection: &Connection) -> Result<(), Error> {
        connection.execute(
            r#"
            CREATE TABLE IF NOT EXISTS asset (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT,
                type INTEGER,
                parent_directory INTEGER,
                FOREIGN KEY(type) REFERENCES asset_type(id),
                FOREIGN KEY(parent_directory) REFERENCES directory(id)
            );
            "#,
            (),
        )?;

        Ok(())
    }
}
