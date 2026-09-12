import XCTest
@testable import RustyLib

final class RustyLibTests: XCTestCase {

    // MARK: - Basic Smoke Tests

    func testRustHello() {
        let result = rustHello()
        XCTAssertTrue(result.contains("kubo-rs"), "rustHello should mention kubo-rs")
    }

    func testRustAdd() {
        XCTAssertEqual(rustAdd(a: 2, b: 3), 5)
        XCTAssertEqual(rustAdd(a: 0, b: 0), 0)
    }

    // MARK: - HybridNode Lifecycle

    func testHybridStartStop() throws {
        try hybridStartTry(online: false)
        let ipfsID = hybridIpfsPeerId()
        XCTAssertFalse(ipfsID.isEmpty, "IPFS peer ID should not be empty after start")

        let p2pID = hybridP2pPeerId()
        // P2P ID may be empty in offline mode; that's OK
        _ = p2pID

        try hybridStopTry()
    }

    func testHybridIpfsRoundtrip() throws {
        try hybridStartTry(online: false)
        defer { _ = hybridStop() }

        let payload = "Swift test roundtrip \(UUID().uuidString)"
        let cid = try ipfsAddTry(data: Data(payload.utf8))
        XCTAssertFalse(cid.isEmpty, "CID should not be empty")

        let fetched = try ipfsCatTry(cid: cid)
        let fetchedString = String(data: fetched, encoding: .utf8)
        XCTAssertEqual(fetchedString, payload, "Round-trip data should match")
    }

    func testHybridPinManagement() throws {
        try hybridStartTry(online: false)
        defer { _ = hybridStop() }

        let cid = try ipfsAddTry(data: Data("pin test".utf8))
        XCTAssertTrue(ipfsPinAdd(cid: cid, recursive: true), "Pin add should succeed")

        let pins = ipfsPinLs()
        let hasPin = pins.contains(cid) || pins.contains("/ipfs/\(cid)")
        XCTAssertTrue(hasPin, "Pinned CID should appear in pin list: \(pins)")

        XCTAssertTrue(ipfsPinRm(cid: cid, recursive: true), "Pin rm should succeed")
    }

    // MARK: - Block Operations

    func testBlockPutGetStat() throws {
        try hybridStartTry(online: false)
        defer { _ = hybridStop() }

        let data = Data("block test data".utf8)
        let cid = ipfsBlockPut(data: data)
        XCTAssertFalse(cid.isEmpty, "Block CID should not be empty")

        let fetched = ipfsBlockGet(cid: cid)
        XCTAssertEqual(fetched, data, "Block get should return original data")

        let size = ipfsBlockStat(cid: cid)
        XCTAssertEqual(size, UInt64(data.count), "Block stat should report correct size")
    }

    // MARK: - Nostr

    func testNostrKeygen() {
        let sk = nostrGenerateKey()
        XCTAssertFalse(sk.isEmpty, "Generated secret key should not be empty")

        let pk = nostrGetPublicKey(sk: sk)
        XCTAssertFalse(pk.isEmpty, "Derived public key should not be empty")
        XCTAssertNotEqual(sk, pk, "Secret and public keys should differ")
    }

    func testNostrEventSignVerify() {
        let sk = nostrGenerateKey()
        let eventJson = nostrEventSign(sk: sk, content: "Swift test event", kind: 1)
        XCTAssertFalse(eventJson.isEmpty, "Signed event JSON should not be empty")

        let valid = nostrEventVerify(eventJson: eventJson)
        XCTAssertTrue(valid, "Freshly signed event should verify")
    }

    // MARK: - Git

    func testGitInitAndHead() {
        let tmpDir = FileManager.default.temporaryDirectory
            .appendingPathComponent("kubo-rs-swift-test-\(UUID().uuidString)")
            .path

        defer { try? FileManager.default.removeItem(atPath: tmpDir) }

        XCTAssertTrue(gitInit(path: tmpDir, bare: false), "Git init should succeed")

        let head = gitHead(path: tmpDir)
        // Fresh repo may have empty head; just ensure it doesn't crash
        _ = head

        let branches = gitBranches(path: tmpDir)
        XCTAssertTrue(branches.isEmpty || branches.contains("master") || branches.contains("main"), "Branches should be empty or contain default branch")

        let remotes = gitRemotes(path: tmpDir)
        XCTAssertTrue(remotes.isEmpty, "Fresh repo should have no remotes")
    }

    // MARK: - Git Advanced

    func testGitLogAndTags() {
        let tmpDir = FileManager.default.temporaryDirectory
            .appendingPathComponent("kubo-rs-swift-test-\(UUID().uuidString)")
            .path
        defer { try? FileManager.default.removeItem(atPath: tmpDir) }

        XCTAssertTrue(gitInit(path: tmpDir, bare: false), "Git init should succeed")

        let logBefore = gitLog(path: tmpDir, maxCount: 10)
        XCTAssertFalse(logBefore.isEmpty, "gitLog should return valid JSON even for empty repo")

        let tagsBefore = gitTags(path: tmpDir)
        XCTAssertFalse(tagsBefore.isEmpty, "gitTags should return valid JSON even for empty repo")
    }

    func testGitCloneAndFetch() {
        let tmpDir = FileManager.default.temporaryDirectory
            .appendingPathComponent("kubo-rs-swift-test-\(UUID().uuidString)")
            .path
        defer { try? FileManager.default.removeItem(atPath: tmpDir) }

        let ok = gitClone(url: "https://github.com/RandyMcMillan/kubo-rs.git", path: tmpDir, bare: false)
        if ok {
            XCTAssertTrue(FileManager.default.fileExists(atPath: (tmpDir as NSString).appendingPathComponent(".git")), ".git should exist after clone")

            let log = gitLog(path: tmpDir, maxCount: 5)
            XCTAssertFalse(log.isEmpty, "gitLog should return commits after clone")

            let tags = gitTags(path: tmpDir)
            XCTAssertFalse(tags.isEmpty, "gitTags should return valid JSON after clone")

            let fetchOk = gitFetchAll(path: tmpDir)
            XCTAssertTrue(fetchOk, "gitFetchAll should succeed for cloned repo")
        } else {
            XCTSkip("Clone skipped (network may be unavailable)")
        }
    }

    func testGitBlameOnReadme() {
        let tmpDir = FileManager.default.temporaryDirectory
            .appendingPathComponent("kubo-rs-swift-test-\(UUID().uuidString)")
            .path
        defer { try? FileManager.default.removeItem(atPath: tmpDir) }

        let ok = gitClone(url: "https://github.com/RandyMcMillan/kubo-rs.git", path: tmpDir, bare: false)
        if ok {
            let blame = gitBlame(path: tmpDir, filePath: "README.md")
            XCTAssertFalse(blame.isEmpty, "gitBlame should return blame for README.md")
        } else {
            XCTSkip("Clone skipped (network may be unavailable)")
        }
    }

    // MARK: - Module Interaction (full workflow)

    func testFullGitWorkflow() {
        let tmpDir = FileManager.default.temporaryDirectory
            .appendingPathComponent("kubo-rs-swift-test-\(UUID().uuidString)")
            .path
        defer { try? FileManager.default.removeItem(atPath: tmpDir) }

        // 1. Init
        XCTAssertTrue(gitInit(path: tmpDir, bare: false), "Init should succeed")

        // 2. Verify empty state
        let head = gitHead(path: tmpDir)
        _ = head
        let branches = gitBranches(path: tmpDir)
        XCTAssertTrue(branches.isEmpty || branches.contains("main") || branches.contains("master"), "Default branch expected")

        // 3. Verify log/tags on empty repo don't crash
        let log = gitLog(path: tmpDir, maxCount: 10)
        XCTAssertFalse(log.isEmpty, "Log should return JSON")
        let tags = gitTags(path: tmpDir)
        XCTAssertFalse(tags.isEmpty, "Tags should return JSON")

        // 4. Fetch on repo with no remotes should fail gracefully
        let fetchOk = gitFetchAll(path: tmpDir)
        XCTAssertFalse(fetchOk, "Fetch should fail when no remotes exist")
    }

    // MARK: - P2P Host (best-effort; may be empty in simulator)

    func testP2pHostSmoke() {
        let peerID = p2pStart()
        // P2P host may fail in CI/simulator; just ensure it doesn't crash
        _ = peerID
        _ = p2pListeningAddrs()
        _ = p2pProtocols()
        _ = p2pGossipTopic()
    }
}
