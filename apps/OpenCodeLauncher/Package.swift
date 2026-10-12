// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "OpenCodeLauncher",
    platforms: [.macOS(.v13)],
    targets: [
        .executableTarget(
            name: "OpenCodeLauncher",
            path: "Sources/OpenCodeLauncher"
        )
    ]
)
