use std::env;

use surrealdb::engine::any::{connect, Any};
use surrealdb::Surreal;

use crate::DbMode;

async fn setup_db(
    db_mode: DbMode,
    namespace: String,
    db_name: String,
) -> Result<Surreal<Any>, String> {
    let db_url = match db_mode {
        DbMode::InMemory => "mem://".to_string(),
        DbMode::FileSystem(filename) => {
            let manifest_path = env::var("CARGO_MANIFEST_DIR").unwrap();
            // Default asset directory path can be changed with `BEVY_ASSET_ROOT` env var
            let assets_dir = match env::var("BEVY_ASSET_ROOT") {
                Ok(path) => path,
                Err(_) => format!("{}/{}", manifest_path, "assets"),
            };

            format!("file://{assets_dir}/{filename}")
        }
        DbMode::Cloud => todo!(),
    };

    let db = connect(db_url).await.unwrap();
    db.use_ns(namespace).use_db(db_name).await.unwrap();

    Ok(db)
}
