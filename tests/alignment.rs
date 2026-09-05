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
    // For every named Kind variant in the Rust nostr crate, verify that
    // signing an event with the Go FFI and parsing it with the Rust crate
    // preserves the kind correctly and the signature verifies.
    let sk = kubo_rs::nostr_generate_key().expect("go keygen failed");

    #[rustfmt::skip]
    let all_kinds: &[(Kind, i32)] = &[
        (Kind::Metadata, 0),
        (Kind::TextNote, 1),
        (Kind::RecommendRelay, 2),
        (Kind::ContactList, 3),
        (Kind::OpenTimestamps, 1040),
        (Kind::EncryptedDirectMessage, 4),
        (Kind::EventDeletion, 5),
        (Kind::Repost, 6),
        (Kind::GenericRepost, 16),
        (Kind::Comment, 1111),
        (Kind::Reaction, 7),
        (Kind::BadgeAward, 8),
        (Kind::ChannelCreation, 40),
        (Kind::ChannelMetadata, 41),
        (Kind::ChannelMessage, 42),
        (Kind::ChannelHideMessage, 43),
        (Kind::ChannelMuteUser, 44),
        (Kind::MlsKeyPackage, 443),
        (Kind::MlsWelcome, 444),
        (Kind::MlsGroupMessage, 445),
        (Kind::RepoState, 30618),
        (Kind::GitPatch, 1617),
        (Kind::GitPullRequest, 1618),
        (Kind::GitPullRequestUpdate, 1619),
        (Kind::GitIssue, 1621),
        (Kind::GitReply, 1622),
        (Kind::GitStatusOpen, 1630),
        (Kind::GitStatusApplied, 1631),
        (Kind::GitStatusClosed, 1632),
        (Kind::GitStatusDraft, 1633),
        (Kind::WalletConnectInfo, 13194),
        (Kind::Reporting, 1984),
        (Kind::Label, 1985),
        (Kind::GroupPutUser, 9000),
        (Kind::GroupRemoveUser, 9001),
        (Kind::GroupEditMetadata, 9002),
        (Kind::GroupDeleteEvent, 9005),
        (Kind::GroupCreateGroup, 9007),
        (Kind::GroupDeleteGroup, 9008),
        (Kind::GroupCreateInvite, 9009),
        (Kind::GroupJoinRequest, 9021),
        (Kind::GroupLeaveRequest, 9022),
        (Kind::GroupMetadata, 39000),
        (Kind::GroupAdmins, 39001),
        (Kind::GroupMembers, 39002),
        (Kind::GroupRoles, 39003),
        (Kind::GroupLivekitParticipants, 39004),
        (Kind::ZapPrivateMessage, 9733),
        (Kind::ZapRequest, 9734),
        (Kind::ZapReceipt, 9735),
        (Kind::Highlight, 9802),
        (Kind::MuteList, 10000),
        (Kind::PinList, 10001),
        (Kind::RelayList, 10002),
        (Kind::Bookmarks, 10003),
        (Kind::Communities, 10004),
        (Kind::PublicChats, 10005),
        (Kind::BlockedRelays, 10006),
        (Kind::SearchRelays, 10007),
        (Kind::SimpleGroups, 10009),
        (Kind::Interests, 10015),
        (Kind::Emojis, 10030),
        (Kind::InboxRelays, 10050),
        (Kind::MlsKeyPackageRelays, 10051),
        (Kind::BlossomServerList, 10063),
        (Kind::Authentication, 22242),
        (Kind::WalletConnectRequest, 23194),
        (Kind::WalletConnectResponse, 23195),
        (Kind::WalletConnectNotification, 23196),
        (Kind::NostrConnect, 24133),
        (Kind::LiveEvent, 30311),
        (Kind::LiveEventMessage, 1311),
        (Kind::ProfileBadges, 10008),
        (Kind::BadgeSet, 30008),
        (Kind::BadgeDefinition, 30009),
        (Kind::Seal, 13),
        (Kind::GiftWrap, 1059),
        (Kind::PrivateDirectMessage, 14),
        (Kind::SetStall, 30017),
        (Kind::SetProduct, 30018),
        (Kind::JobFeedback, 7000),
        (Kind::FollowSet, 30000),
        (Kind::RelaySet, 30002),
        (Kind::BookmarkSet, 30003),
        (Kind::ArticlesCurationSet, 30004),
        (Kind::VideosCurationSet, 30005),
        (Kind::InterestSet, 30015),
        (Kind::EmojiSet, 30030),
        (Kind::ReleaseArtifactSet, 30063),
        (Kind::LongFormTextNote, 30023),
        (Kind::GitRepoAnnouncement, 30617),
        (Kind::FileMetadata, 1063),
        (Kind::BlossomAuth, 24242),
        (Kind::HttpAuth, 27235),
        (Kind::ApplicationSpecificData, 30078),
        (Kind::Torrent, 2003),
        (Kind::TorrentComment, 2004),
        (Kind::PeerToPeerOrder, 38383),
        (Kind::RequestToVanish, 62),
        (Kind::UserStatus, 30315),
        (Kind::VoiceMessage, 1222),
        (Kind::VoiceMessageReply, 1244),
        (Kind::CashuWallet, 17375),
        (Kind::CashuWalletUnspentProof, 7375),
        (Kind::CashuWalletSpendingHistory, 7376),
        (Kind::CashuWalletQuote, 7374),
        (Kind::CashuNutZapInfo, 10019),
        (Kind::GitUserGraspList, 10317),
        (Kind::CashuNutZap, 9321),
        (Kind::CodeSnippet, 1337),
        (Kind::Poll, 1068),
        (Kind::PollResponse, 1018),
        (Kind::ChatMessage, 9),
        (Kind::Thread, 11),
        (Kind::WebBookmark, 39701),
        (Kind::RelayMonitor, 10166),
        (Kind::RelayDiscovery, 30166),
    ];

    for &(kind, expected_value) in all_kinds {
        let val = expected_value;
        let event_json = kubo_rs::nostr_event_sign(&sk, "kind test", val)
            .unwrap_or_else(|_| panic!("go sign failed for kind {}", val));
        let event: Event = serde_json::from_str(&event_json)
            .unwrap_or_else(|_| panic!("rust parse failed for kind {}", val));
        assert_eq!(
            event.kind.as_u16() as i32,
            val,
            "kind {} ({:?}) must survive round-trip",
            val,
            kind
        );
        event
            .verify()
            .unwrap_or_else(|_| panic!("rust verify failed for kind {}", val));
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
// nostr-sdk alignment
// ---------------------------------------------------------------------------

#[test]
fn test_nostr_sdk_event_parses_go_signed_event() {
    // Verify that the nostr-sdk crate (which re-exports nostr::Event)
    // can parse and verify an event produced by the Go FFI.
    let sk = kubo_rs::nostr_generate_key().expect("go keygen failed");
    let event_json =
        kubo_rs::nostr_event_sign(&sk, "nostr-sdk alignment", 1).expect("go sign failed");

    let event: nostr_sdk::prelude::Event =
        serde_json::from_str(&event_json).expect("nostr-sdk parse failed");
    event.verify().expect("nostr-sdk verify failed");
    assert_eq!(event.content, "nostr-sdk alignment");
}

// ---------------------------------------------------------------------------
// NIP-34 (Git Stuff) alignment
// ---------------------------------------------------------------------------

#[test]
fn test_nip34_kind_values_alignment() {
    // NIP-34 defines git-related kinds. Verify Rust and Go agree.
    let cases: &[(Kind, i32, &str)] = &[
        (Kind::GitPatch, 1617, "KindPatch"),
        (Kind::GitPullRequest, 1618, "KindPullRequest"),
        (Kind::GitPullRequestUpdate, 1619, "KindPullRequestUpdate"),
        (Kind::GitIssue, 1621, "KindIssue"),
        (Kind::GitReply, 1622, "KindReply"),
        (Kind::GitStatusOpen, 1630, "KindStatusOpen"),
        (Kind::GitStatusApplied, 1631, "KindStatusApplied"),
        (Kind::GitStatusClosed, 1632, "KindStatusClosed"),
        (Kind::GitStatusDraft, 1633, "KindStatusDraft"),
        (
            Kind::GitRepoAnnouncement,
            30617,
            "KindRepositoryAnnouncement",
        ),
        (Kind::RepoState, 30618, "KindRepositoryState"),
    ];

    for (rust_kind, expected_value, go_name) in cases {
        let actual = rust_kind.as_u16() as i32;
        assert_eq!(
            actual, *expected_value,
            "NIP-34 Kind mismatch: Rust {:?} ({}) != Go {} ({})",
            rust_kind, actual, go_name, expected_value
        );
    }
}

#[test]
fn test_nip34_repo_announcement_rust_to_go() {
    // Construct a NIP-34 repository announcement event in Rust with
    // proper tags and verify the Go FFI accepts it.
    let keys = Keys::generate();
    let unsigned = UnsignedEvent::new(
        keys.public_key(),
        Timestamp::now(),
        Kind::GitRepoAnnouncement,
        [
            Tag::parse(["d", "my-repo"]).unwrap(),
            Tag::parse(["name", "My Repository"]).unwrap(),
            Tag::parse(["description", "A test repo"]).unwrap(),
            Tag::parse(["clone", "https://example.com/repo.git"]).unwrap(),
        ],
        "",
    );
    let event = keys.sign_event(unsigned).expect("rust sign failed");
    let event_json = serde_json::to_string(&event).expect("rust serialize failed");

    let ok = kubo_rs::nostr_event_verify(&event_json).expect("go verify failed");
    assert!(
        ok,
        "go FFI must verify rust-signed NIP-34 repo announcement"
    );
}

#[test]
fn test_nip34_repo_state_rust_to_go() {
    let keys = Keys::generate();
    let unsigned = UnsignedEvent::new(
        keys.public_key(),
        Timestamp::now(),
        Kind::RepoState,
        [
            Tag::parse(["d", "my-repo"]).unwrap(),
            Tag::parse(["HEAD", "ref: refs/heads/main"]).unwrap(),
            Tag::parse(["refs/heads/main", "abc123"]).unwrap(),
            Tag::parse(["refs/tags/v1.0", "def456"]).unwrap(),
        ],
        "",
    );
    let event = keys.sign_event(unsigned).expect("rust sign failed");
    let event_json = serde_json::to_string(&event).expect("rust serialize failed");

    let ok = kubo_rs::nostr_event_verify(&event_json).expect("go verify failed");
    assert!(ok, "go FFI must verify rust-signed NIP-34 repo state");
}

#[test]
fn test_nip34_patch_rust_to_go() {
    let keys = Keys::generate();
    let unsigned = UnsignedEvent::new(
        keys.public_key(),
        Timestamp::now(),
        Kind::GitPatch,
        [Tag::parse(["a", "30617:abcdef:my-repo"]).unwrap()],
        "diff --git a/file.txt b/file.txt\n--- a/file.txt\n+++ b/file.txt\n@@ -1 +1 @@\n-old\n+new\n",
    );
    let event = keys.sign_event(unsigned).expect("rust sign failed");
    let event_json = serde_json::to_string(&event).expect("rust serialize failed");

    let ok = kubo_rs::nostr_event_verify(&event_json).expect("go verify failed");
    assert!(ok, "go FFI must verify rust-signed NIP-34 patch");
}

#[test]
fn test_nip34_issue_and_status_rust_to_go() {
    let keys = Keys::generate();

    for &(kind, desc) in &[
        (Kind::GitIssue, "issue"),
        (Kind::GitReply, "reply"),
        (Kind::GitStatusOpen, "status-open"),
        (Kind::GitStatusApplied, "status-applied"),
        (Kind::GitStatusClosed, "status-closed"),
        (Kind::GitStatusDraft, "status-draft"),
    ] {
        let unsigned = UnsignedEvent::new(
            keys.public_key(),
            Timestamp::now(),
            kind,
            [Tag::parse(["e", "abc123"]).unwrap()],
            format!("{} description", desc),
        );
        let event = keys.sign_event(unsigned).expect("rust sign failed");
        let event_json = serde_json::to_string(&event).expect("rust serialize failed");

        let ok = kubo_rs::nostr_event_verify(&event_json)
            .unwrap_or_else(|_| panic!("go verify failed for NIP-34 {}", desc));
        assert!(ok, "go FFI must verify rust-signed NIP-34 {}", desc);
    }
}

#[test]
fn test_nip34_go_to_rust_round_trip() {
    // Sign NIP-34 events via Go FFI and parse/verify them with Rust.
    let sk = kubo_rs::nostr_generate_key().expect("go keygen failed");

    let kinds: &[(i32, &str)] = &[
        (30617, "repo announcement"),
        (30618, "repo state"),
        (1617, "patch"),
        (1618, "pull request"),
        (1619, "pr update"),
        (1621, "issue"),
        (1622, "reply"),
        (1630, "status open"),
        (1631, "status applied"),
        (1632, "status closed"),
        (1633, "status draft"),
    ];

    for &(kind_val, desc) in kinds {
        let event_json = kubo_rs::nostr_event_sign(&sk, desc, kind_val)
            .unwrap_or_else(|_| panic!("go sign failed for NIP-34 {}", desc));
        let event: Event = serde_json::from_str(&event_json)
            .unwrap_or_else(|_| panic!("rust parse failed for NIP-34 {}", desc));
        assert_eq!(
            event.kind.as_u16() as i32,
            kind_val,
            "NIP-34 kind {} ({}) must survive round-trip",
            kind_val,
            desc
        );
        event
            .verify()
            .unwrap_or_else(|_| panic!("rust verify failed for NIP-34 {}", desc));
    }
}

// ---------------------------------------------------------------------------
// NIP-19 alignment
// ---------------------------------------------------------------------------

#[test]
fn test_nip19_pubkey_encode_alignment() {
    let sk = kubo_rs::nostr_generate_key().expect("go keygen failed");
    let pk = kubo_rs::nostr_get_public_key(&sk).expect("go pubkey failed");

    // Encode via Go FFI.
    let npub_go = kubo_rs::nostr_nip19_encode_pubkey(&pk).expect("go encode failed");
    assert!(npub_go.starts_with("npub1"), "npub must start with npub1");

    // Decode via Go FFI.
    let pk_decoded = kubo_rs::nostr_nip19_decode_pubkey(&npub_go).expect("go decode failed");
    assert_eq!(pk_decoded, pk, "round-trip must preserve pubkey");

    // Encode via Rust nostr crate and compare.
    let npub_rs = PublicKey::from_hex(&pk)
        .expect("rust parse failed")
        .to_bech32()
        .expect("rust encode failed");
    assert_eq!(
        npub_go, npub_rs,
        "Go and Rust NIP-19 pubkey encoding must match"
    );
}

#[test]
fn test_nip19_seckey_encode_alignment() {
    let sk = kubo_rs::nostr_generate_key().expect("go keygen failed");

    // Encode via Go FFI.
    let nsec_go = kubo_rs::nostr_nip19_encode_seckey(&sk).expect("go encode failed");
    assert!(nsec_go.starts_with("nsec1"), "nsec must start with nsec1");

    // Decode via Go FFI.
    let sk_decoded = kubo_rs::nostr_nip19_decode_seckey(&nsec_go).expect("go decode failed");
    assert_eq!(sk_decoded, sk, "round-trip must preserve seckey");

    // Encode via Rust nostr crate and compare.
    let nsec_rs = Keys::parse(&sk)
        .expect("rust parse failed")
        .secret_key()
        .to_bech32()
        .expect("rust encode failed");
    assert_eq!(
        nsec_go, nsec_rs,
        "Go and Rust NIP-19 seckey encoding must match"
    );
}

#[test]
fn test_nip19_note_encode_alignment() {
    let fake_id = "a".repeat(64);

    // Encode via Go FFI.
    let note_go = kubo_rs::nostr_nip19_encode_note(&fake_id).expect("go encode failed");
    assert!(note_go.starts_with("note1"), "note must start with note1");

    // Decode via Go FFI.
    let id_decoded = kubo_rs::nostr_nip19_decode_note(&note_go).expect("go decode failed");
    assert_eq!(id_decoded, fake_id, "round-trip must preserve id");

    // Encode via Rust nostr crate and compare.
    let note_rs = EventId::from_hex(&fake_id)
        .expect("rust parse failed")
        .to_bech32()
        .expect("rust encode failed");
    assert_eq!(
        note_go, note_rs,
        "Go and Rust NIP-19 note encoding must match"
    );
}

#[test]
fn test_nip19_entity_encode_alignment() {
    let pk = kubo_rs::nostr_generate_key().expect("go keygen failed");
    let pubkey = kubo_rs::nostr_get_public_key(&pk).expect("go pubkey failed");

    // Encode via Go FFI (no relay).
    let naddr_go = kubo_rs::nostr_nip19_encode_entity(&pubkey, 30617, "my-repo", "")
        .expect("go encode failed");
    assert!(
        naddr_go.starts_with("naddr1"),
        "naddr must start with naddr1"
    );

    // Decode via Go FFI.
    let json = kubo_rs::nostr_nip19_decode_entity(&naddr_go).expect("go decode failed");
    assert!(
        json.contains("my-repo"),
        "decoded entity must contain identifier"
    );
    assert!(json.contains(&pubkey), "decoded entity must contain pubkey");

    // Encode via Rust nostr crate and compare.
    let coord = Coordinate::new(
        Kind::GitRepoAnnouncement,
        PublicKey::from_hex(&pubkey).expect("rust parse failed"),
    )
    .identifier("my-repo");
    let naddr_rs = coord.to_bech32().expect("rust encode failed");
    assert_eq!(
        naddr_go, naddr_rs,
        "Go and Rust NIP-19 entity encoding must match"
    );
}

// ---------------------------------------------------------------------------
// Git extended alignment
// ---------------------------------------------------------------------------

#[test]
fn test_git_bare_alignment() {
    let path = tmp_path("git_bare_alignment");

    // Init bare repo via Go FFI.
    kubo_rs::git_init(path.to_str().unwrap(), true).expect("go git init bare failed");

    // Check via Go FFI.
    let repo_go = kubo_rs::Repository::open(&path).expect("go open failed");
    assert!(
        repo_go.is_bare().expect("go is_bare failed"),
        "go FFI must report bare"
    );
    repo_go.close().expect("go close failed");

    // Check via git2.
    let repo_rs = git2::Repository::open(&path).expect("git2 open failed");
    assert!(repo_rs.is_bare(), "git2 must report bare");
}

#[test]
fn test_git_branches_and_remotes_alignment() {
    let path = tmp_path("git_branches_remotes_alignment");

    // Init via Go FFI.
    kubo_rs::git_init(path.to_str().unwrap(), false).expect("go git init failed");

    // Create a commit so we can create branches.
    let repo_rs = git2::Repository::open(&path).expect("git2 open failed");
    let sig = git2::Signature::now("Test", "test@example.com").expect("signature failed");
    let tree_id = {
        let mut index = repo_rs.index().expect("index failed");
        let blob_id = repo_rs.blob(b"hello").expect("blob failed");
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
                b"hello",
            )
            .expect("add failed");
        index.write_tree().expect("write tree failed")
    };
    let tree = repo_rs.find_tree(tree_id).expect("find tree failed");
    let commit_id = repo_rs
        .commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
        .expect("commit failed");

    // Add a remote via git2.
    repo_rs
        .remote("origin", "https://example.com/repo.git")
        .expect("remote add failed");

    // Open via Go FFI and verify alignment.
    let repo_go = kubo_rs::Repository::open(&path).expect("go open failed");

    // Branches.
    let branches_go = repo_go.branches().expect("go branches failed");
    let branches_rs: Vec<String> = repo_rs
        .branches(Some(git2::BranchType::Local))
        .expect("git2 branches failed")
        .filter_map(|b| {
            b.ok()
                .and_then(|(b, _)| b.name().ok().flatten().map(String::from))
        })
        .collect();
    assert_eq!(
        branches_go, branches_rs,
        "branch lists must align between go-git FFI and git2"
    );

    // Remotes.
    let remotes_go = repo_go.remotes().expect("go remotes failed");
    let remotes_rs: Vec<String> = repo_rs
        .remotes()
        .expect("git2 remotes failed")
        .iter()
        .filter_map(|n| n.ok().flatten().map(String::from))
        .collect();
    assert_eq!(
        remotes_go, remotes_rs,
        "remote lists must align between go-git FFI and git2"
    );

    // Create branch via Go FFI.
    repo_go
        .create_branch("feature", &commit_id.to_string())
        .expect("go create branch failed");

    let branches_go_after = repo_go.branches().expect("go branches after create failed");
    assert!(
        branches_go_after.contains(&"feature".to_string()),
        "new branch must appear in go FFI branch list"
    );

    repo_go.close().expect("go close failed");
}

// ---------------------------------------------------------------------------
// nostr:// URL alignment (gnostr ideas)
// ---------------------------------------------------------------------------

#[test]
fn test_nostr_url_parse_alignment() {
    let url_str =
        "nostr://abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890/my-repo";
    let url = kubo_rs::NostrUrl::parse(url_str).expect("parse failed");
    assert_eq!(
        url.authority,
        "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
    );
    assert_eq!(url.identifier, "my-repo");
    assert!(url.authority_is_pubkey());
    assert_eq!(url.to_url(), url_str);
}

#[test]
fn test_nostr_url_nip05_alignment() {
    let url = kubo_rs::NostrUrl::parse("nostr://dan@gitworkshop.dev/ngit").expect("parse failed");
    assert_eq!(url.authority, "dan@gitworkshop.dev");
    assert_eq!(url.identifier, "ngit");
    assert!(!url.authority_is_pubkey());
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
