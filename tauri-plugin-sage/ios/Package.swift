// swift-tools-version:5.3
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "tauri-plugin-sage",
    defaultLocalization: "en",
    platforms: [
        .iOS("15.0"),
    ],
    products: [
        // Products define the executables and libraries a package produces, and make them visible to other packages.
        .library(
            name: "tauri-plugin-sage",
            type: .static,
            targets: ["tauri-plugin-sage"]
        ),
    ],
    dependencies: [
        .package(name: "Tauri", path: "../.tauri/tauri-api")
    ],
    targets: [
        // Targets are the basic building blocks of a package. A target can define a module or a test suite.
        // Targets can depend on other targets in this package, and on products in packages this package depends on.
        .target(
            name: "tauri-plugin-sage",
            dependencies: [
                .byName(name: "Tauri"),
                .byName(name: "TangemSdk"),
            ],
            path: "Sources"
        ),
        .target(
            name: "TangemSdk",
            dependencies: [
                "TangemSdk_secp256k1"
            ],
            path: "TangemSdk",
            exclude: [
                "Crypto/secp256k1",
                "module.modulemap",
                "TangemSdk.h",
            ],
            resources: [
                .process("Common/Localization/Resources"),
                .copy("Haptics"),
                .copy("Crypto/BIP39/Wordlists/english.txt"),
                .copy("PrivacyInfo.xcprivacy"),
                .copy("Assets"),
            ]
        ),
        .target(
            name: "TangemSdk_secp256k1",
            path: "TangemSdk/Crypto/secp256k1"
        ),
        .testTarget(
            name: "TangemSdkTests",
            dependencies: [
                "TangemSdk",
            ],
            path: "TangemSdkTests",
            resources: [
                .copy("Jsons"),
            ]
        ),
    ]
)
