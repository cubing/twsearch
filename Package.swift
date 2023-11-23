// swift-tools-version:5.3
import PackageDescription

let package = Package(
    name: "TWSearch",
    products: [
        .library(name: "TWSearch", targets: ["TWSearchLibSwift"]),
    ],
    targets: [
        .target(
            name: "TWSearchLibSwift",
            dependencies: ["TWSearchLib"],
            path: "./src/rs-swift/SwiftGenerated"
	),
        .binaryTarget(
            name: "TWSearchLib",
            url: "https://github.com/xbjfk/twsearch/releases/download/0.2.0/TWSearch.xcframework.zip",
            checksum: "3e4b804bfff4b91b2a7922f0cf7a78cc2baebc8968cfbade71563ec9802e9dfd"
        )
    ]
)
