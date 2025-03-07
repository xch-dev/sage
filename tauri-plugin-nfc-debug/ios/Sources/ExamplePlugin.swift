import SwiftRs
import Tauri
import UIKit
import WebKit
import CoreNFC
import TangemSdk

class PingArgs: Decodable {
    let value: String?
}

class ExamplePlugin: Plugin {
    @objc public func ping(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(PingArgs.self)
        
        let sdk = TangemSdk()
        
        sdk.scanCard() { result in
            switch result {
            case .success(let value):
                invoke.resolve(["value": value.cardPublicKey.hexString])
            case .failure(let error):
                invoke.resolve(["value": "There was an error"])
            }
        }
    }
}

@_cdecl("init_plugin_nfc_debug")
func initPlugin() -> Plugin {
    return ExamplePlugin()
}
