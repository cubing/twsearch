// swift-tools-version:5.3
import PackageDescription

let package = Package(
    name: "TwSearch",
    products: [
        .library(name: "TwSearch", targets: ["TwSearchLibSwift"]),
    ],
    targets: [
        .target(
            name: "TwSearchLibSwift",
            dependencies: ["TwSearchLib"],
            path: "./src/rs-swift/SwiftGenerated"
	),
        .binaryTarget(
            name: "TwSearchLib",
            url: "https://github.com/xbjfk/twsearch/releases/download/0.2.0/TwSearch.xcframework.zip",
            checksum: "3e4b804bfff4b91b2a7922f0cf7a78cc2baebc8968cfbade71563ec9802e9dfd"
        )
    ]
)
