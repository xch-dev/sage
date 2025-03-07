// swift-tools-version:5.5
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "tauri-plugin-nfc-debug",
    platforms: [
        .macOS(.v10_13),
        .iOS(.v15),
    ],
    products: [
        // Products define the executables and libraries a package produces, and make them visible to other packages.
        .library(
            name: "tauri-plugin-nfc-debug",
            type: .static,
            targets: ["tauri-plugin-nfc-debug"]),
    ],
    dependencies: [
        .package(name: "Tauri", path: "../.tauri/tauri-api"),
        .package(url: "https://github.com/tangem/tangem-sdk-ios.git", .exact("3.9.0"))
    ],
    targets: [
        // Targets are the basic building blocks of a package. A target can define a module or a test suite.
        // Targets can depend on other targets in this package, and on products in packages this package depends on.
        .target(
            name: "tauri-plugin-nfc-debug",
            dependencies: [
                .byName(name: "Tauri"),
                .product(name: "TangemSdk", package: "tangem-sdk-ios")
            ],
            path: "Sources")
    ]
)
