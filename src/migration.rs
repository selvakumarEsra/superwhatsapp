//! The one-time move from an older install (REQ-SUPERWA-005).
//!
//! SuperWhatsapp was called ZapFast, FastsApp and FastWhatsApp before. This
//! module is the only place that knows those names: the folders, keychain
//! services, login item, log and programs an older install left behind. It
//! runs once, when SuperWhatsapp's own folder does not exist yet; after that
//! nothing reads, writes or creates anything under an older name
//! (REQ-SUPERWA-006, REQ-SUPERWA-007).

use std::path::{Path, PathBuf};

use directories::ProjectDirs;

use crate::paths::{AppDirs, adopt_directory, move_file};

/// The folder names older installs used, newest first. Only read, to move
/// their data over; nothing is ever created under them.
const PREVIOUS_NAMES: [&str; 3] = ["zapfast", "fastsapp", "fastwhatsapp"];
/// The platform qualifier and organization of those folders
/// (`me.paolino.zapfast` on macOS, `paolino\zapfast` on Windows).
const PREVIOUS_QUALIFIER: &str = "me";
const PREVIOUS_ORGANIZATION: &str = "paolino";
/// The log file of an install from before the rename.
const PREVIOUS_LOG: &str = "zapfast.log";
/// Keychain service of archive keys saved before the rename. Only read, to
/// copy a key over; never written.
pub(crate) const PREVIOUS_ARCHIVE_SERVICE: &str = "rocks.zapfast.ZapFast";
/// Keychain service of Znga API keys saved before the rename.
pub(crate) const PREVIOUS_ZNGA_SERVICE: &str = "rocks.zapfast.ZapFast.znga";
/// The Linux login item id of an install from before the rename.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub(crate) const PREVIOUS_AUTOSTART_ID: &str = "zapfast";
/// The macOS launch agent of an install from before the rename.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub(crate) const PREVIOUS_LAUNCH_AGENT: &str = "me.paolino.zapfast.plist";
/// Program and bundle names older releases installed, which the updater
/// replaces in place.
pub const PREVIOUS_PROGRAMS: [&str; 2] = ["zapfast", "fastsapp"];
pub const PREVIOUS_EXECUTABLE: &str = "zapfast";
pub const PREVIOUS_BUNDLES: [&str; 2] = ["ZapFast.app", "FastsApp.app"];

impl AppDirs {
    /// The folders of older installs, newest first.
    fn previous() -> impl Iterator<Item = Self> {
        PREVIOUS_NAMES
            .into_iter()
            .filter_map(|name| Self::of(ProjectDirs::from(PREVIOUS_QUALIFIER, PREVIOUS_ORGANIZATION, name)))
    }

    /// Where a copy of an older install that is still running answers, so
    /// a new start can hand over to it instead of moving its folders away
    /// while it runs (REQ-SUPERWA-005).
    pub fn previous_runtimes() -> Vec<PathBuf> {
        Self::previous().map(|dirs| dirs.runtime).collect()
    }

    /// Adopts earlier names, newest first, without replacing existing data.
    /// Each encrypted archive's key moves with its folder (REQ-SUPERWA-005).
    /// Call only after acquiring the instance guard, and never for demo runs.
    pub fn adopt_previous_names(&self) -> std::io::Result<()> {
        // Once SuperWhatsapp has its own folder the move is done: the older
        // folders are never read again, even when they reappear because an
        // older copy ran (REQ-SUPERWA-005.A3, A4).
        if self.state.try_exists()? {
            if Self::previous().any(|old| old.state.is_dir() && old.state != self.state) {
                log::warn!("an older install's folders are left alone: SuperWhatsapp already has its own");
            }
            return Ok(());
        }
        self.adopt_now()
    }

    /// The move itself, without the once-only check.
    fn adopt_now(&self) -> std::io::Result<()> {
        let mut move_key = |old_identity: &str, archive: &Path| {
            crate::archive::move_archive_key(old_identity, archive)
                .map_err(|error| std::io::Error::other(format!("{error:#}")))
        };
        for old in Self::previous() {
            self.adopt_moving_keys(&old, &mut move_key)?;
        }
        if let Some(to) = eframe::storage_dir("superwhatsapp") {
            for name in PREVIOUS_NAMES {
                if let Some(from) = eframe::storage_dir(name) {
                    adopt_directory(&from, &to)?;
                }
            }
        }
        move_file(&self.state.join(PREVIOUS_LOG), &self.log_file())
    }

    /// Moves an older install's folders here like [`Self::adopt`], and each
    /// encrypted archive's keyring key with them. When a key cannot be
    /// moved, its folder goes back and nothing after it moves.
    fn adopt_moving_keys(
        &self,
        old: &Self,
        move_key: &mut dyn FnMut(&str, &Path) -> std::io::Result<()>,
    ) -> std::io::Result<()> {
        // macOS keeps configuration and state in one folder; whichever pair
        // moves it first carries its archives.
        for (from, to) in [
            (&old.config, &self.config),
            (&old.state, &self.state),
            (&old.cache, &self.cache),
        ] {
            adopt_directory_with_keys(from, to, move_key)?;
        }
        Ok(())
    }

    /// [`Self::adopt_moving_keys`] for archives with no key to move.
    #[cfg(test)]
    fn adopt(&self, old: &Self) -> std::io::Result<()> {
        self.adopt_moving_keys(old, &mut |_, _| Ok(()))
    }
}

/// The encrypted-archive candidates in a state folder, relative to it: the
/// single-account archive at its root and each account's.
fn archives_in(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    if dir.join("archive.db").is_file() {
        found.push(PathBuf::from("archive.db"));
    }
    if let Ok(accounts) = std::fs::read_dir(dir.join("accounts")) {
        for account in accounts.flatten() {
            let relative = Path::new("accounts")
                .join(account.file_name())
                .join("archive.db");
            if dir.join(&relative).is_file() {
                found.push(relative);
            }
        }
    }
    found.sort();
    found
}

/// [`adopt_directory`] for a folder that may hold encrypted archives: each
/// archive's key identity is taken before the move, and its key is copied to
/// the folder's new identity after it. If a key cannot be copied the folder
/// is moved back, so the older install still opens, and the error stops the
/// start.
fn adopt_directory_with_keys(
    from: &Path,
    to: &Path,
    move_key: &mut dyn FnMut(&str, &Path) -> std::io::Result<()>,
) -> std::io::Result<()> {
    if !from.is_dir() {
        return Ok(());
    }
    if to.try_exists()? {
        log::warn!("an older install's folder is left alone: SuperWhatsapp already has its own");
        return Ok(());
    }
    let identities = archives_in(from)
        .into_iter()
        .map(|relative| {
            crate::archive::archive_key_identity(&from.join(&relative))
                .map(|identity| (relative, identity))
                .map_err(|error| std::io::Error::other(format!("{error:#}")))
        })
        .collect::<std::io::Result<Vec<_>>>()?;
    adopt_directory(from, to)?;
    for (relative, identity) in &identities {
        if let Err(error) = move_key(identity, &to.join(relative)) {
            if let Err(back) = std::fs::rename(to, from) {
                log::error!("could not move an older install's folder back: {back}");
            }
            return Err(std::io::Error::other(format!(
                "could not move an archive key to the SuperWhatsapp folder: {error}"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::AccountId;

    fn root(name: &str) -> PathBuf {
        let root = std::env::temp_dir()
            .join(format!("superwhatsapp-migration-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn rename_preserves_session_archive_settings_and_cached_files() {
        for name in ["fastsapp", "fastwhatsapp"] {
            let root = root(name);
            let old = AppDirs::under(&root.join(name));
            let new = AppDirs::under(&root.join("superwhatsapp"));
            old.ensure().unwrap();
            for path in [
                old.settings_file(),
                old.session_db(),
                old.state.join("session.db-wal"),
                old.archive_db(),
                old.state.join("archive.db-wal"),
                old.saved_sticker_dir().join("pack/sticker.webp"),
                old.media_cache_dir().join("photo.jpg"),
            ] {
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, b"preserved").unwrap();
            }
            new.adopt(&old).unwrap();
            new.adopt(&old).unwrap(); // A second launch is a no-op.
            for path in [
                new.settings_file(),
                new.session_db(),
                new.state.join("session.db-wal"),
                new.archive_db(),
                new.state.join("archive.db-wal"),
                new.saved_sticker_dir().join("pack/sticker.webp"),
                new.media_cache_dir().join("photo.jpg"),
            ] {
                assert_eq!(std::fs::read(path).unwrap(), b"preserved");
            }
            assert!(!old.config.exists());
            assert!(!old.state.exists());
            assert!(!old.cache.exists());
            std::fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn newest_data_wins_without_merging_archives() {
        let root = root("precedence");
        let new = AppDirs::under(&root.join("superwhatsapp"));
        let recent = AppDirs::under(&root.join("fastsapp"));
        let oldest = AppDirs::under(&root.join("fastwhatsapp"));
        recent.ensure().unwrap();
        oldest.ensure().unwrap();
        std::fs::create_dir_all(&new.config).unwrap();
        std::fs::write(new.settings_file(), b"new settings").unwrap();
        std::fs::write(recent.settings_file(), b"old settings").unwrap();
        std::fs::write(recent.archive_db(), b"recent archive").unwrap();
        std::fs::write(oldest.archive_db(), b"oldest archive").unwrap();
        new.adopt(&recent).unwrap();
        new.adopt(&oldest).unwrap();
        assert_eq!(std::fs::read(new.settings_file()).unwrap(), b"new settings");
        assert_eq!(
            std::fs::read(recent.settings_file()).unwrap(),
            b"old settings"
        );
        assert_eq!(std::fs::read(new.archive_db()).unwrap(), b"recent archive");
        assert_eq!(
            std::fs::read(oldest.archive_db()).unwrap(),
            b"oldest archive"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn shared_config_and_state_directory_moves_once() {
        let root = root("shared");
        let old = AppDirs {
            config: root.join("old/data"),
            state: root.join("old/data"),
            cache: root.join("old/cache"),
            runtime: root.join("old/run"),
        };
        let new = AppDirs {
            config: root.join("new/data"),
            state: root.join("new/data"),
            cache: root.join("new/cache"),
            runtime: root.join("new/run"),
        };
        old.ensure().unwrap();
        std::fs::write(old.session_db(), b"session").unwrap();
        new.adopt(&old).unwrap();
        assert_eq!(std::fs::read(new.session_db()).unwrap(), b"session");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn failed_migration_leaves_source_available_for_retry() {
        let root = root("failure");
        let old = AppDirs::under(&root.join("old"));
        let new = AppDirs::under(&root.join("blocked/new"));
        old.ensure().unwrap();
        std::fs::write(old.session_db(), b"session").unwrap();
        std::fs::write(root.join("blocked"), b"not a directory").unwrap();
        assert!(new.adopt(&old).is_err());
        assert_eq!(std::fs::read(old.session_db()).unwrap(), b"session");
        std::fs::remove_file(root.join("blocked")).unwrap();
        new.adopt(&old).unwrap();
        assert_eq!(std::fs::read(new.session_db()).unwrap(), b"session");
        std::fs::remove_dir_all(root).unwrap();
    }

    fn write(path: &Path, bytes: &[u8]) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }

    #[test]
    fn an_older_install_moves_with_its_archive_keys() {
        let root = root("keys");
        let old = AppDirs::under(&root.join("me.paolino.zapfast"));
        let new = AppDirs::under(&root.join("cc.superwhatsapp"));
        let archive = old.account(&AccountId::first()).archive_db();
        write(&archive, b"encrypted pages");
        write(&old.archive_db(), b"single account");
        write(&old.media_cache_dir().join("photo.jpg"), b"photo");
        let expected = [
            crate::archive::archive_key_identity(&old.archive_db()).unwrap(),
            crate::archive::archive_key_identity(&archive).unwrap(),
        ];
        let mut moved = Vec::new();
        new.adopt_moving_keys(&old, &mut |identity, path| {
            assert!(path.is_file(), "the key moves after its archive");
            moved.push((identity.to_owned(), path.to_path_buf()));
            Ok(())
        })
        .unwrap();
        assert_eq!(
            moved,
            vec![
                (
                    expected[1].clone(),
                    new.account(&AccountId::first()).archive_db()
                ),
                (expected[0].clone(), new.state.join("archive.db")),
            ]
        );
        assert_eq!(
            std::fs::read(new.account(&AccountId::first()).archive_db()).unwrap(),
            b"encrypted pages"
        );
        assert_eq!(
            std::fs::read(new.media_cache_dir().join("photo.jpg")).unwrap(),
            b"photo"
        );
        assert!(!old.state.exists() && !old.cache.exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_missing_key_moves_nothing() {
        let root = root("missing-key");
        let old = AppDirs::under(&root.join("me.paolino.zapfast"));
        let new = AppDirs::under(&root.join("cc.superwhatsapp"));
        let archive = old.account(&AccountId::first()).archive_db();
        write(&archive, b"encrypted pages");
        write(&old.media_cache_dir().join("photo.jpg"), b"photo");
        let error = new
            .adopt_moving_keys(&old, &mut |_, _| Err(std::io::Error::other("no key")))
            .unwrap_err();
        assert!(error.to_string().contains("no key"));
        // The folder went back, and the start stopped before the cache moved.
        assert_eq!(std::fs::read(&archive).unwrap(), b"encrypted pages");
        assert!(!new.state.exists());
        assert!(old.media_cache_dir().join("photo.jpg").exists());
        assert!(!new.cache.exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn the_new_folders_win_over_old_ones() {
        let root = root("new-wins");
        let old = AppDirs::under(&root.join("me.paolino.zapfast"));
        let new = AppDirs::under(&root.join("cc.superwhatsapp"));
        write(
            &old.account(&AccountId::first()).archive_db(),
            b"old archive",
        );
        write(
            &new.account(&AccountId::first()).archive_db(),
            b"new archive",
        );
        new.adopt_moving_keys(&old, &mut |_, _| panic!("no key moves"))
            .unwrap();
        assert_eq!(
            std::fs::read(new.account(&AccountId::first()).archive_db()).unwrap(),
            b"new archive"
        );
        assert_eq!(
            std::fs::read(old.account(&AccountId::first()).archive_db()).unwrap(),
            b"old archive"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_new_install_uses_only_superwhatsapp_folders() {
        let dirs = AppDirs::discover();
        for path in [&dirs.config, &dirs.state, &dirs.cache, &dirs.runtime] {
            let path = path.to_string_lossy().to_lowercase();
            assert!(path.contains("superwhatsapp"), "{path}");
            for old in ["zapfast", "paolino", "fastsapp", "fastwhatsapp"] {
                assert!(!path.contains(old), "{path}");
            }
        }
        assert!(dirs.log_file().ends_with("superwhatsapp.log"));
        #[cfg(target_os = "macos")]
        assert!(
            dirs.state
                .ends_with("Library/Application Support/cc.superwhatsapp")
        );
        for old in AppDirs::previous_runtimes() {
            assert!(!old.to_string_lossy().contains("superwhatsapp"));
        }
    }

    #[test]
    fn the_move_runs_only_while_superwhatsapp_has_no_folder() {
        let root = root("once");
        let new = AppDirs::under(&root.join("cc.superwhatsapp"));
        std::fs::create_dir_all(&new.state).unwrap();
        // With its own folder present, an older one is never looked at,
        // whatever its state.
        new.adopt_previous_names().unwrap();
        assert!(!new.log_file().exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn only_this_module_names_the_older_install() {
        for name in PREVIOUS_NAMES {
            assert!(!AppDirs::discover().state.to_string_lossy().contains(name));
        }
        assert!(PREVIOUS_ARCHIVE_SERVICE.starts_with("rocks.zapfast"));
        assert!(PREVIOUS_ZNGA_SERVICE.ends_with(".znga"));
    }
}
