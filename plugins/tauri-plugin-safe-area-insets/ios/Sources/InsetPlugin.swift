import SwiftRs
import Tauri
import UIKit
import WebKit

class InsetsResponse: Encodable {
    let top: Double
    let bottom: Double
    let left: Double
    let right: Double

    init() {
        // Return all zeros for insets
        self.top = 0.0
        self.bottom = 0.0
        self.left = 0.0
        self.right = 0.0
    }
}

class InsetPlugin: Plugin {
    @objc public func getInsets(_ invoke: Invoke) throws {
        let response = InsetsResponse()
        invoke.resolve(response)
    }
}

@_cdecl("init_plugin_safe_area_insets")
func initPlugin() -> Plugin {
    return InsetPlugin()
}