// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "GameKitBridge",
    platforms: [
        .macOS(.v12)
    ],
    products: [
        .library(
            name: "GameKitBridge",
            type: .static,
            targets: ["GameKitBridge"])
    ],
    targets: [
        .target(
            name: "GameKitBridge",
            path: "Sources/GameKitBridge",
            publicHeadersPath: "include")
    ]
)
