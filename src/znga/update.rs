//! Self-update for Znga builds: releases come from this fork, signed with
//! the fork's own key, so an update never swaps Znga for the upstream app.
//! Replaces `updates::CONFIG` when the `znga` feature is on.

use fastframe_update::{MacConfig, Prereleases, UpdateConfig};

/// The fork's GitHub repository, which publishes Znga releases.
pub const REPOSITORY: &str = "selvakumarEsra/superwhatsapp";

/// Upstream's names and history, with the fork's repository and key.
/// Fork versions are pre-releases (`0.21.1-beta.1`), so they follow every
/// published fork release above them.
pub const CONFIG: UpdateConfig = UpdateConfig {
    legacy_names: &["zapfast", "fastsapp"],
    macos: MacConfig {
        // The bundle ID moves to cc.superwhatsapp over two releases: this one
        // keeps the old ID and accepts the new one, so it can update in place
        // to the release that switches (REQ-SUPERWA-008).
        bundle_ids: &["me.paolino.fastsapp", "cc.superwhatsapp"],
        executable_names: &["superwhatsapp", "zapfast"],
        legacy_bundle_names: &["ZapFast.app", "FastsApp.app"],
    },
    publisher_key: Some(include_str!("update-public-key.hex")),
    additional_publisher_keys: &[],
    prereleases: Prereleases::WhenRunningPrerelease,
    ..UpdateConfig::new(
        REPOSITORY,
        "SuperWhatsapp",
        "superwhatsapp",
        env!("CARGO_PKG_VERSION"),
    )
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn updates_come_from_the_fork_with_its_own_key() {
        CONFIG.validate().unwrap();
        assert_eq!(CONFIG.repository, REPOSITORY);
        assert_eq!(
            CONFIG.publisher_key.map(str::trim),
            Some(include_str!("update-public-key.hex").trim())
        );
        assert_ne!(
            CONFIG.publisher_key,
            Some(include_str!("../../assets/update-public-key.hex")),
            "never upstream's key"
        );
    }

    #[test]
    fn updates_accept_the_next_bundle_id() {
        assert!(CONFIG.macos.bundle_ids.contains(&"me.paolino.fastsapp"));
        assert!(CONFIG.macos.bundle_ids.contains(&"cc.superwhatsapp"));
    }
}
