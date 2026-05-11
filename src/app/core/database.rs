// SPDX-License-Identifier: GPL-3.0-only

use anywho::anywho;
use keepass::{Database, DatabaseKey};
use secrecy::{ExposeSecret, SecretString};
use std::{path::PathBuf, sync::Arc, sync::Mutex};
use tracing::{info, warn};

use crate::{
    APP_ID,
    app::core::entry::{FreeTotpEntry, update_free_totp_entry_in_keepass},
};

/// Checks whether the application database already exists.
///
/// This function looks for the database file in the platform-specific
/// application data directory under [`APP_ID`].
///
/// # Returns
///
/// - `Ok(Some(path))` if the database file exists, where `path` is the full
///   path to the database file.
/// - `Ok(None)` if the database file does not exist.
/// - `Err(_)` if the system data directory cannot be determined.
///
/// # Errors
///
/// Returns an error if the platform-specific data directory is unavailable.
pub fn check_database() -> Result<Option<PathBuf>, anywho::Error> {
    let path = dirs::data_dir()
        .ok_or_else(|| anywho!("Could not determine data directory"))?
        .join(APP_ID)
        .join("database.kdbx");

    info!("DATABASE_PATH {:?}", &path);

    if path.exists() {
        Ok(Some(path))
    } else {
        Ok(None)
    }
}

pub async fn create_database(password: SecretString) -> Result<PathBuf, anywho::Error> {
    let path = dirs::data_dir()
        .ok_or_else(|| anywho!("Could not determine data directory"))?
        .join(APP_ID)
        .join("database.kdbx");

    smol::unblock(move || {
        // Create the directory if it does not exist
        let dir_path = path
            .parent()
            .ok_or_else(|| anywho!("Database path has no parent directory"))?;
        std::fs::create_dir_all(dir_path)?;

        let mut db = Database::new();
        db.meta.database_name = Some(String::from("FreeTotp Database"));

        let mut root = db.root_mut();
        let mut group = root.add_group();
        group.name = String::from("Default Group");

        db.save(
            &mut std::fs::File::create(&path)?,
            DatabaseKey::new().with_password(password.expose_secret()),
        )?;

        Ok(path)
    })
    .await
}

pub async fn unlock_database(
    path: PathBuf,
    password: SecretString,
) -> Result<FreeTotpDatabase, anywho::Error> {
    smol::unblock(move || {
        let mut file = std::fs::File::open(&path)?;
        let key = DatabaseKey::new().with_password(password.expose_secret());
        let _db = Database::open(&mut file, key).map_err(|e| match e {
            keepass::error::DatabaseOpenError::Key(_) => anywho!("Incorrect Password"),
            other => other.into(),
        })?;

        Ok(FreeTotpDatabase {
            path: Box::from(path),
            password: Box::from(password),
            lock: Arc::new(Mutex::new(())),
        })
    })
    .await
}

#[derive(Debug, Clone)]
pub struct FreeTotpDatabase {
    path: Box<PathBuf>,
    password: Box<SecretString>,
    lock: Arc<std::sync::Mutex<()>>, // We use this to prevent Race Condition / Data Loss
}

impl FreeTotpDatabase {
    pub async fn list_entries(&self) -> Result<Vec<FreeTotpEntry>, anywho::Error> {
        let lock = self.lock.clone();

        let path = self.path.clone();
        let password = self.password.clone();

        smol::unblock(move || {
            let _guard = lock
                .lock()
                .map_err(|e| anywho!("Database lock poisoned: {}", e))?;

            let mut file = std::fs::File::open(&*path)?;
            let key = DatabaseKey::new().with_password(password.expose_secret());
            let db = Database::open(&mut file, key)?;
            drop(file); // this should't be needed here because we only read, I just added it for consistency

            let entries = db
                .root()
                .group_by_name("Default Group")
                .map(|g| {
                    let mut v = g
                        .entries()
                        .map(|e| FreeTotpEntry::try_from(e.to_owned()))
                        .collect::<Result<Vec<_>, _>>()?;
                    v.sort_by_key(|a| a.name.to_lowercase());
                    Ok::<Vec<FreeTotpEntry>, anywho::Error>(v)
                })
                .transpose()?
                .unwrap_or_else(Vec::new);

            Ok(entries)
        })
        .await
    }

    pub async fn add_entry(&self, entry: FreeTotpEntry) -> Result<(), anywho::Error> {
        self.add_entries(vec![entry]).await
    }

    pub async fn add_entries(&self, entries: Vec<FreeTotpEntry>) -> Result<(), anywho::Error> {
        let lock = self.lock.clone();

        let path = self.path.clone();
        let password = self.password.clone();

        smol::unblock(move || {
            let _guard = lock
                .lock()
                .map_err(|e| anywho!("Database lock poisoned: {}", e))?;

            let mut file = std::fs::File::open(&*path)?;
            let key = DatabaseKey::new().with_password(password.expose_secret());
            let mut db = Database::open(&mut file, key)?;
            drop(file);

            let mut root = db.root_mut();
            let mut target_group = root
                .group_by_name_mut("Default Group")
                .ok_or_else(|| anywho!("Default Group not found"))?;

            for entry in entries {
                let mut keepass_entry = target_group.add_entry();
                update_free_totp_entry_in_keepass(entry, &mut keepass_entry);
            }

            db.save(
                &mut std::fs::File::create(&*path)?,
                DatabaseKey::new().with_password(password.expose_secret()),
            )?;

            Ok(())
        })
        .await
    }

    pub async fn update_entry(&self, entry: FreeTotpEntry) -> Result<(), anywho::Error> {
        let lock = self.lock.clone();

        let path = self.path.clone();
        let password = self.password.clone();

        smol::unblock(move || {
            let _guard = lock
                .lock()
                .map_err(|e| anywho!("Database lock poisoned: {}", e))?;

            let mut file = std::fs::File::open(&*path)?;
            let key = DatabaseKey::new().with_password(password.expose_secret());
            let mut db = Database::open(&mut file, key)?;
            drop(file);

            let entry_id = entry
                .id
                .ok_or_else(|| anywho!("Cannot update entry without UUID"))?;

            let mut root = db.root_mut();
            let mut target_group = root
                .group_by_name_mut("Default Group")
                .ok_or_else(|| anywho!("Default Group not found"))?;

            // Find and update the entry
            let entry_id = target_group
                .entry_ids()
                .find(|e| e.uuid() == entry_id)
                .ok_or_else(|| anywho!("Entry with UUID {} not found", entry_id))?;
            let mut entry_found = target_group
                .entry_mut(entry_id)
                .ok_or_else(|| anywho!("Entry with UUID {} not found", entry_id))?;

            update_free_totp_entry_in_keepass(entry, &mut entry_found);

            db.save(
                &mut std::fs::File::create(&*path)?,
                DatabaseKey::new().with_password(password.expose_secret()),
            )?;

            Ok(())
        })
        .await
    }

    pub async fn delete_entry(&self, entry_id: uuid::Uuid) -> Result<(), anywho::Error> {
        let lock = self.lock.clone();

        let path = self.path.clone();
        let password = self.password.clone();

        smol::unblock(move || {
            let _guard = lock
                .lock()
                .map_err(|e| anywho!("Database lock poisoned: {}", e))?;

            let mut file = std::fs::File::open(&*path)?;
            let key = DatabaseKey::new().with_password(password.expose_secret());
            let mut db = Database::open(&mut file, key)?;
            drop(file);

            let mut root = db.root_mut();
            let mut target_group = root
                .group_by_name_mut("Default Group")
                .ok_or_else(|| anywho!("Default Group not found"))?;

            let entry_id = target_group
                .entry_ids()
                .find(|e| e.uuid() == entry_id)
                .ok_or_else(|| anywho!("Entry with UUID {} not found", entry_id))?;
            let entry_found = target_group
                .entry_mut(entry_id)
                .ok_or_else(|| anywho!("Entry with UUID {} not found", entry_id))?;

            entry_found.remove();

            db.save(
                &mut std::fs::File::create(&*path)?,
                DatabaseKey::new().with_password(password.expose_secret()),
            )?;

            Ok(())
        })
        .await
    }

    // Import the content given in standard totp
    pub async fn import_content(&self, file_path: PathBuf) -> Result<(), anywho::Error> {
        // Read the import file
        let content = std::fs::read_to_string(&file_path)
            .map_err(|e| anywho!("Failed to read import file: {}", e))?;

        let mut entries_to_import = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            match totp_rs::TOTP::from_url_unchecked(line) {
                Ok(totp) => {
                    let name = if totp.account_name.trim().is_empty() {
                        "Default".to_string()
                    } else {
                        totp.account_name.clone()
                    };

                    entries_to_import.push(FreeTotpEntry {
                        id: None,
                        name,
                        totp,
                    });
                }
                Err(e) => {
                    warn!("Warning: Failed to parse TOTP URL '{}': {}", line, e);
                }
            }
        }

        if !entries_to_import.is_empty() {
            self.add_entries(entries_to_import).await?;
        }

        Ok(())
    }

    // Export content to standard
    pub async fn export_content(&self, file_path: PathBuf) -> Result<(), anywho::Error> {
        let entries = self.list_entries().await?;

        if entries.is_empty() {
            return Err(anywho!("No entries found to export"));
        }

        let mut export_content = String::new();

        for entry in entries {
            let url = entry.totp.get_url();
            export_content.push_str(&url);
            export_content.push('\n');
        }

        std::fs::write(&file_path, export_content)
            .map_err(|e| anywho!("Failed to write export file: {}", e))?;

        Ok(())
    }
}
