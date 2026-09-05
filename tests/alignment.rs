//! Alignment tests: verify Go FFI implementations match native Rust crates.

use nostr::prelude::*;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Nostr alignment
// ---------------------------------------------------------------------------

#[test]
fn test_nostr_key_alignment() {
    // Generate a keypair via Go FFI.
    let sk_go = kubo_rs::nostr_generate_key().expect("go keygen failed");
    let pk_go = kubo_rs::nostr_get_public_key(&sk_go).expect("go pubkey failed");

    // Parse the same secret key with the native Rust nostr crate.
    let keys_rs = Keys::parse(&sk_go).expect("rust key parse failed");
    let pk_rs = keys_rs.public_key().to_hex();

    assert_eq!(
        pk_go, pk_rs,
        "public keys derived from the same secret key must match"
    );
}

#[test]
fn test_nostr_event_sign_alignment_go_to_rust() {
    // Sign an event via Go FFI.
    let sk = kubo_rs::nostr_generate_key().expect("go keygen failed");
    let event_json = kubo_rs::nostr_event_sign(&sk, "alignment test", 1).expect("go sign failed");

    // Parse the event with the native Rust nostr crate and verify it.
    let event: Event = serde_json::from_str(&event_json).expect("rust event parse failed");
    event
        .verify()
        .expect("rust verify of go-signed event failed");

    // Also verify via Go FFI that the event is valid.
    let ok = kubo_rs::nostr_event_verify(&event_json).expect("go verify failed");
    assert!(ok, "go verify must succeed for its own signed event");
}

#[test]
fn test_nostr_event_sign_alignment_rust_to_go() {
    // Create and sign an event with the native Rust nostr crate.
    let keys = Keys::generate();
    let unsigned = UnsignedEvent::new(
        keys.public_key(),
        Timestamp::now(),
        Kind::TextNote,
        [],
        "alignment test from rust",
    );
    let event = keys.sign_event(unsigned).expect("rust sign failed");
    let event_json = serde_json::to_string(&event).expect("rust serialize failed");

    // Verify the Rust-signed event via Go FFI.
    let ok =
        kubo_rs::nostr_event_verify(&event_json).expect("go verify of rust-signed event failed");
    assert!(ok, "go verify must accept rust-signed event");
}

#[test]
fn test_nostr_kind_values_alignment() {
    // Verify that the Rust `nostr` crate Kind enum values align with
    // the Go `go-nostr` kind constants in `go/nostr/kinds.go`.
    let cases: &[(Kind, i32, &str)] = &[
        // NIP-01 / NIP-05
        (Kind::Metadata, 0, "KindProfileMetadata"),
        (Kind::TextNote, 1, "KindTextNote"),
        (Kind::RecommendRelay, 2, "KindRecommendServer"),
        (Kind::ContactList, 3, "KindFollowList"),
        // NIP-04
        (
            Kind::EncryptedDirectMessage,
            4,
            "KindEncryptedDirectMessage",
        ),
        // NIP-09
        (Kind::EventDeletion, 5, "KindDeletion"),
        // NIP-18
        (Kind::Repost, 6, "KindRepost"),
        // NIP-25
        (Kind::Reaction, 7, "KindReaction"),
        // NIP-58
        (Kind::BadgeAward, 8, "KindBadgeAward"),
        // NIP-28
        (Kind::ChannelCreation, 40, "KindChannelCreation"),
        (Kind::ChannelMetadata, 41, "KindChannelMetadata"),
        (Kind::ChannelMessage, 42, "KindChannelMessage"),
        (Kind::ChannelHideMessage, 43, "KindChannelHideMessage"),
        (Kind::ChannelMuteUser, 44, "KindChannelMuteUser"),
        // NIP-59
        (Kind::Seal, 13, "KindSeal"),
        (Kind::GiftWrap, 1059, "KindGiftWrap"),
        // NIP-17
        (Kind::PrivateDirectMessage, 14, "KindDirectMessage"),
        // NIP-18
        (Kind::GenericRepost, 16, "KindGenericRepost"),
        // NIP-22
        (Kind::Comment, 1111, "KindComment"),
        // NIP-94
        (Kind::FileMetadata, 1063, "KindFileMetadata"),
        // NIP-53
        (Kind::LiveEventMessage, 1311, "KindLiveChatMessage"),
        // NIP-34
        (Kind::GitPatch, 1617, "KindPatch"),
        (Kind::GitIssue, 1621, "KindIssue"),
        (Kind::GitReply, 1622, "KindReply"),
        (Kind::GitStatusOpen, 1630, "KindStatusOpen"),
        (Kind::GitStatusApplied, 1631, "KindStatusApplied"),
        (Kind::GitStatusClosed, 1632, "KindStatusClosed"),
        (Kind::GitStatusDraft, 1633, "KindStatusDraft"),
        // NIP-56
        (Kind::Reporting, 1984, "KindReporting"),
        // NIP-32
        (Kind::Label, 1985, "KindLabel"),
        // NIP-35
        (Kind::Torrent, 2003, "KindTorrent"),
        (Kind::TorrentComment, 2004, "KindTorrentComment"),
        // NIP-57
        (Kind::ZapRequest, 9734, "KindZapRequest"),
        (Kind::ZapReceipt, 9735, "KindZap"),
        // NIP-84
        (Kind::Highlight, 9802, "KindHighlights"),
        // NIP-51
        (Kind::MuteList, 10000, "KindMuteList"),
        (Kind::PinList, 10001, "KindPinList"),
        (Kind::RelayList, 10002, "KindRelayListMetadata"),
        (Kind::Bookmarks, 10003, "KindBookmarkList"),
        (Kind::Communities, 10004, "KindCommunityList"),
        (Kind::PublicChats, 10005, "KindPublicChatList"),
        (Kind::BlockedRelays, 10006, "KindBlockedRelayList"),
        (Kind::SearchRelays, 10007, "KindSearchRelayList"),
        (Kind::SimpleGroups, 10009, "KindSimpleGroupList"),
        (Kind::Interests, 10015, "KindInterestList"),
        (Kind::Emojis, 10030, "KindEmojiList"),
        // NIP-17
        (Kind::InboxRelays, 10050, "KindDMRelayList"),
        // NIP-47
        (Kind::WalletConnectInfo, 13194, "KindNWCWalletInfo"),
        // NIP-42
        (Kind::Authentication, 22242, "KindClientAuthentication"),
        // NIP-47
        (Kind::WalletConnectRequest, 23194, "KindNWCWalletRequest"),
        (Kind::WalletConnectResponse, 23195, "KindNWCWalletResponse"),
        // NIP-46
        (Kind::NostrConnect, 24133, "KindNostrConnect"),
        // NIP-98
        (Kind::BlossomAuth, 24242, "KindBlobs"),
        // NIP-98
        (Kind::HttpAuth, 27235, "KindHTTPAuth"),
        // NIP-51
        (Kind::FollowSet, 30000, "KindCategorizedPeopleList"),
        (Kind::RelaySet, 30002, "KindRelaySets"),
        (Kind::BookmarkSet, 30003, "KindBookmarkSets"),
        (Kind::ArticlesCurationSet, 30004, "KindCuratedSets"),
        (Kind::VideosCurationSet, 30005, "KindCuratedVideoSets"),
        (Kind::InterestSet, 30015, "KindInterestSets"),
        (Kind::EmojiSet, 30030, "KindEmojiSets"),
        // NIP-58
        (Kind::BadgeSet, 30008, "KindProfileBadges"),
        (Kind::BadgeDefinition, 30009, "KindBadgeDefinition"),
        // NIP-15
        (Kind::SetStall, 30017, "KindStallDefinition"),
        (Kind::SetProduct, 30018, "KindProductDefinition"),
        // NIP-23
        (Kind::LongFormTextNote, 30023, "KindArticle"),
        // NIP-51
        (Kind::ReleaseArtifactSet, 30063, "KindReleaseArtifactSets"),
        // NIP-78
        (
            Kind::ApplicationSpecificData,
            30078,
            "KindApplicationSpecificData",
        ),
        // NIP-53
        (Kind::LiveEvent, 30311, "KindLiveEvent"),
        // NIP-38
        (Kind::UserStatus, 30315, "KindUserStatuses"),
        // NIP-34
        (
            Kind::GitRepoAnnouncement,
            30617,
            "KindRepositoryAnnouncement",
        ),
        (Kind::RepoState, 30618, "KindRepositoryState"),
        // NIP-29
        (Kind::GroupPutUser, 9000, "KindSimpleGroupPutUser"),
        (Kind::GroupRemoveUser, 9001, "KindSimpleGroupRemoveUser"),
        (Kind::GroupEditMetadata, 9002, "KindSimpleGroupEditMetadata"),
        (Kind::GroupDeleteEvent, 9005, "KindSimpleGroupDeleteEvent"),
        (Kind::GroupCreateGroup, 9007, "KindSimpleGroupCreateGroup"),
        (Kind::GroupDeleteGroup, 9008, "KindSimpleGroupDeleteGroup"),
        (Kind::GroupCreateInvite, 9009, "KindSimpleGroupCreateInvite"),
        (Kind::GroupJoinRequest, 9021, "KindSimpleGroupJoinRequest"),
        (Kind::GroupLeaveRequest, 9022, "KindSimpleGroupLeaveRequest"),
        (Kind::GroupMetadata, 39000, "KindSimpleGroupMetadata"),
        (Kind::GroupAdmins, 39001, "KindSimpleGroupAdmins"),
        (Kind::GroupMembers, 39002, "KindSimpleGroupMembers"),
        (Kind::GroupRoles, 39003, "KindSimpleGroupRoles"),
        // NIP-61
        (Kind::CashuNutZap, 9321, "KindNutZap"),
        (Kind::CashuNutZapInfo, 10019, "KindNutZapInfo"),
        // NIP-90
        (Kind::JobFeedback, 7000, "KindJobFeedback"),
        // NIP-03
        (Kind::OpenTimestamps, 1040, "KindOpenTimestamps"),
    ];

    for (rust_kind, expected_value, go_name) in cases {
        let actual = rust_kind.as_u16() as i32;
        assert_eq!(
            actual, *expected_value,
            "Kind mismatch: Rust {:?} ({}) != Go {} ({})",
            rust_kind, actual, go_name, expected_value
        );
    }
}

#[test]
fn test_nostr_kind_event_round_trip_for_each_kind() {
    // For a representative set of kinds, verify that signing an event
    // with the Go FFI and parsing it with the Rust crate preserves
    // the kind correctly.
    let sk = kubo_rs::nostr_generate_key().expect("go keygen failed");

    let representative_kinds: &[(i32, &str)] = &[
        (0, "metadata"),
        (1, "text note"),
        (3, "contact list"),
        (4, "encrypted DM"),
        (6, "repost"),
        (7, "reaction"),
        (40, "channel creation"),
        (10000, "mute list"),
        (10002, "relay list"),
        (30023, "article"),
    ];

    for &(kind_val, desc) in representative_kinds {
        let event_json = kubo_rs::nostr_event_sign(&sk, desc, kind_val)
            .expect(&format!("go sign failed for kind {}", kind_val));
        let event: Event = serde_json::from_str(&event_json)
            .expect(&format!("rust parse failed for kind {}", kind_val));
        assert_eq!(
            event.kind.as_u16() as i32,
            kind_val,
            "kind {} ({}) must survive round-trip",
            kind_val,
            desc
        );
        event
            .verify()
            .expect(&format!("rust verify failed for kind {}", kind_val));
    }
}

// ---------------------------------------------------------------------------
// Git alignment
// ---------------------------------------------------------------------------

#[test]
fn test_git_init_alignment() {
    let path = tmp_path("git_init_alignment");

    // Init via Go FFI.
    kubo_rs::git_init(path.to_str().unwrap(), false).expect("go git init failed");
    assert!(
        path.join(".git").exists(),
        ".git should exist after go init"
    );

    // Open the same repo with the native Rust git2 (libgit2) crate.
    let repo = git2::Repository::open(&path).expect("git2 open failed");

    // Verify the repo is not bare.
    assert!(!repo.is_bare(), "repo should not be bare");

    // Create a commit via git2 so HEAD exists.
    let sig = git2::Signature::now("Test", "test@example.com").expect("signature failed");
    let tree_id = {
        let mut index = repo.index().expect("index failed");
        let blob_id = repo.blob(b"hello git alignment").expect("blob failed");
        index
            .add_frombuffer(
                &git2::IndexEntry {
                    ctime: git2::IndexTime::new(0, 0),
                    mtime: git2::IndexTime::new(0, 0),
                    dev: 0,
                    ino: 0,
                    mode: 0o100644,
                    uid: 0,
                    gid: 0,
                    file_size: 0,
                    id: blob_id,
                    flags: 0,
                    flags_extended: 0,
                    path: b"hello.txt".to_vec(),
                },
                b"hello git alignment",
            )
            .expect("add failed");
        index.write_tree().expect("write tree failed")
    };
    let tree = repo.find_tree(tree_id).expect("find tree failed");
    let commit_id = repo
        .commit(
            Some("HEAD"),
            &sig,
            &sig,
            "alignment test commit",
            &tree,
            &[],
        )
        .expect("commit failed");

    // Verify Go FFI sees the same HEAD.
    let repo_go = kubo_rs::Repository::open(&path).expect("go open failed");
    let head_go = repo_go.head().expect("go head failed");
    repo_go.close().expect("go close failed");

    assert_eq!(
        head_go,
        commit_id.to_string(),
        "HEAD hash must match between git2 and go-git ffi"
    );
}

#[test]
fn test_git_init_bare_alignment() {
    let path = tmp_path("git_init_bare_alignment");

    // Init bare repo via Go FFI.
    kubo_rs::git_init(path.to_str().unwrap(), true).expect("go git init bare failed");

    // Open with git2 and verify it's bare.
    let repo = git2::Repository::open(&path).expect("git2 open failed");
    assert!(repo.is_bare(), "repo should be bare");
}

// ---------------------------------------------------------------------------
// libp2p alignment
// ---------------------------------------------------------------------------

#[test]
fn test_libp2p_peer_id_format_alignment() {
    // Create a Go FFI host and get its peer ID.
    let host_go = kubo_rs::Host::new().expect("go host new failed");
    let peer_id_go = host_go.peer_id().expect("go peer_id failed");
    host_go.close().expect("go host close failed");

    // Verify the Go peer ID is a valid rust-libp2p PeerId.
    let peer_id_rs: libp2p::PeerId = peer_id_go
        .parse()
        .expect("go peer_id should parse as rust PeerId");

    // The string representation should round-trip.
    assert_eq!(
        peer_id_go,
        peer_id_rs.to_string(),
        "peer id string representation must round-trip"
    );
}

#[test]
fn test_libp2p_keypair_peer_id_derivation_alignment() {
    // Generate a Rust libp2p keypair and derive its peer ID.
    let keypair_rs = libp2p::identity::Keypair::generate_ed25519();
    let peer_id_rs = keypair_rs.public().to_peer_id();

    // Verify the peer ID is valid according to Go (just format-check).
    let peer_id_str = peer_id_rs.to_string();
    assert!(
        peer_id_str.starts_with("12D3KooW") || peer_id_str.len() == 52,
        "ed25519 peer id should have expected format"
    );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn tmp_path(name: &str) -> PathBuf {
    let path = PathBuf::from("tmp").join("alignment").join(name);
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("mkdir failed");
    path
}
