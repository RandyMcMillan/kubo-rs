// swift-tools-version:5.10
import PackageDescription

let package = Package(
    name: "RustyLib",
    platforms: [
        .iOS(.v15),
        .macCatalyst(.v15)
    ],
    products: [
        .library(
            name: "RustyLib",
            targets: ["RustyLib"])
    ],
    dependencies: [
    ],
    targets: [
        .target(
            name: "rustylibFFI",
            path: "Sources/rustylibFFI",
            publicHeadersPath: "."
        ),
        .target(
            name: "RustyLib",
            dependencies: [
                .byName(name: "RustyCore"),
                .byName(name: "rustylibFFI"),
            ],
            path: "Sources/RustyLib",
            linkerSettings: [
                .linkedLibrary("resolv"),
                .linkedFramework("Security"),
                .linkedFramework("CoreFoundation"),
            ]
        ),
        .binaryTarget(
            name: "RustyCore",
            path: "artifacts/RustyCore.xcframework"
        ),
        .testTarget(
            name: "RustyLibTests",
            dependencies: ["RustyLib"]
        ),
    ]
)
