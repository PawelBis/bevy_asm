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
    pub id: u32,
    pub name: String,
    pub path: String,
    pub asset_type: AssetType,
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
                FOREIGN KEY(type) REFERENCES asset_type(id)
            )
            "#,
            (),
        )?;

        Ok(())
    }
}
