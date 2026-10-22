//! Background GitHub update checker for desktop builds of QuadFurn.
//! Queries GitHub Releases API asynchronously at startup and notifies if a newer semver tag exists.

#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc::{channel, Receiver};

/// Parses a version string like "v0.2.6" or "0.2.5" into (major, minor, patch).
pub fn parse_version_tuple(s: &str) -> Option<(u32, u32, u32)> {
    let clean = s.trim().trim_start_matches(['v', 'V']);
    let main_part = clean.split('-').next()?; // ignore pre-release suffix
    let mut parts = main_part.split('.');
    let major: u32 = parts.next()?.parse().ok()?;
    let minor: u32 = parts.next()?.parse().ok()?;
    let patch: u32 = parts.next()?.parse().ok()?;
    Some((major, minor, patch))
}

/// Returns true if `remote` is strictly newer than `current`.
pub fn is_newer_version(current: &str, remote: &str) -> bool {
    match (parse_version_tuple(current), parse_version_tuple(remote)) {
        (Some(cur), Some(rem)) => rem > cur,
        _ => false,
    }
}

/// Starts a background thread checking GitHub Releases API for a newer version.
/// Silently ignores any failures (network offline, timeouts, rate limits).
#[cfg(not(target_arch = "wasm32"))]
pub fn start_background_check() -> Option<Receiver<String>> {
    let (tx, rx) = channel();
    std::thread::Builder::new()
        .name("quadfurn-update-check".to_string())
        .spawn(move || {
            if let Some(newer_tag) = check_github_latest_release() {
                let _ = tx.send(newer_tag);
            }
        })
        .ok()?;
    Some(rx)
}

#[cfg(not(target_arch = "wasm32"))]
fn check_github_latest_release() -> Option<String> {
    #[derive(serde::Deserialize)]
    struct Release {
        tag_name: String,
    }

    let resp = ureq::get("https://api.github.com/repos/rbm13s/QuadFurn/releases/latest")
        .set("User-Agent", "QuadFurn-CAD-App")
        .set("Accept", "application/vnd.github.v3+json")
        .timeout(std::time::Duration::from_secs(5))
        .call()
        .ok()?;

    let body = resp.into_string().ok()?;
    let rel: Release = serde_json::from_str(&body).ok()?;
    let current_version = env!("CARGO_PKG_VERSION");
    if is_newer_version(current_version, &rel.tag_name) {
        Some(rel.tag_name)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing_and_comparison() {
        assert_eq!(parse_version_tuple("0.2.5"), Some((0, 2, 5)));
        assert_eq!(parse_version_tuple("v0.2.6"), Some((0, 2, 6)));
        assert_eq!(parse_version_tuple("V1.0.0-beta"), Some((1, 0, 0)));
        assert_eq!(parse_version_tuple("invalid"), None);

        assert!(is_newer_version("0.2.5", "v0.2.6"));
        assert!(is_newer_version("0.2.5", "0.3.0"));
        assert!(is_newer_version("0.2.5", "v1.0.0"));
        assert!(!is_newer_version("0.2.5", "v0.2.5"));
        assert!(!is_newer_version("0.2.5", "v0.2.4"));
        assert!(!is_newer_version("0.2.5", "0.1.9"));
    }
}
