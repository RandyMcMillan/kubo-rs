//! Nostr URL parsing for git repository references.
//!
//! Inspired by the `nostr://` URL scheme used in gnostr/ngit.
//!
//! Format: `nostr://<npub|nip05-address>/<identifier>`

use crate::error::Error;

/// A parsed `nostr://` repository URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NostrUrl {
    /// The raw URL string.
    pub raw: String,
    /// The authority part (npub hex or nip05 identifier).
    pub authority: String,
    /// The repository identifier.
    pub identifier: String,
}

impl NostrUrl {
    /// Parse a `nostr://` URL string.
    ///
    /// # Errors
    ///
    /// Returns an error if the string is not a valid nostr URL.
    pub fn parse(url: &str) -> Result<Self, Error> {
        let url = url.trim();
        let prefix = "nostr://";
        if !url.starts_with(prefix) {
            return Err(Error::InvalidNostrUrl);
        }

        let rest = &url[prefix.len()..];
        let mut parts = rest.splitn(2, '/');
        let authority = parts
            .next()
            .ok_or(Error::InvalidNostrUrl)?
            .trim()
            .to_string();
        let identifier = parts
            .next()
            .ok_or(Error::InvalidNostrUrl)?
            .trim()
            .to_string();

        if authority.is_empty() || identifier.is_empty() {
            return Err(Error::InvalidNostrUrl);
        }

        Ok(NostrUrl {
            raw: url.to_string(),
            authority,
            identifier,
        })
    }

    /// Return true if the authority looks like a hex public key (64 chars).
    #[must_use]
    pub fn authority_is_pubkey(&self) -> bool {
        self.authority.len() == 64 && self.authority.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Reconstruct the canonical URL string.
    #[must_use]
    pub fn to_url(&self) -> String {
        format!("nostr://{}/{}", self.authority, self.identifier)
    }
}

impl std::fmt::Display for NostrUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_url())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nostr_url() {
        let url = NostrUrl::parse(
            "nostr://abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890/my-repo",
        )
        .unwrap();
        assert_eq!(
            url.authority,
            "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
        );
        assert_eq!(url.identifier, "my-repo");
        assert!(url.authority_is_pubkey());
        assert_eq!(
            url.to_url(),
            "nostr://abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890/my-repo"
        );
    }

    #[test]
    fn test_parse_nip05_url() {
        let url = NostrUrl::parse("nostr://dan@gitworkshop.dev/ngit").unwrap();
        assert_eq!(url.authority, "dan@gitworkshop.dev");
        assert_eq!(url.identifier, "ngit");
        assert!(!url.authority_is_pubkey());
    }

    #[test]
    fn test_parse_invalid_missing_prefix() {
        assert!(matches!(
            NostrUrl::parse("https://example.com/repo"),
            Err(Error::InvalidNostrUrl)
        ));
    }

    #[test]
    fn test_parse_invalid_missing_identifier() {
        assert!(matches!(
            NostrUrl::parse(
                "nostr://abcdef1234567890abcdef1234567890abcdef1234567890abcdef12345678"
            ),
            Err(Error::InvalidNostrUrl)
        ));
    }

    #[test]
    fn test_parse_invalid_empty_authority() {
        assert!(matches!(
            NostrUrl::parse("nostr:///repo"),
            Err(Error::InvalidNostrUrl)
        ));
    }

    #[test]
    fn test_display_roundtrip() {
        let original = "nostr://npub1xyz/my-project";
        let url = NostrUrl::parse(original).unwrap();
        assert_eq!(url.to_string(), original);
    }
}
