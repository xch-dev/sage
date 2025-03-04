import SwiftRs
import Tauri
import UIKit
import WebKit
import CoreNFC

class PingArgs: Decodable {
  let value: String?
}

class ExamplePlugin: Plugin, NFCTagReaderSessionDelegate {
  // Required delegate methods
  func tagReaderSessionDidBecomeActive(_ session: NFCTagReaderSession) {
    // Not used in our diagnostic, just required by the protocol
  }
  
  func tagReaderSession(_ session: NFCTagReaderSession, didInvalidateWithError error: Error) {
    // Not used in our diagnostic, just required by the protocol
  }
  
  func tagReaderSession(_ session: NFCTagReaderSession, didDetect tags: [NFCTag]) {
    // Not used in our diagnostic, just required by the protocol
  }

  @objc public func ping(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(PingArgs.self)
    
    // Check Info.plist entry
    let entry = Bundle.main.infoDictionary?["NFCReaderUsageDescription"] as? String
    let hasUsageDescription = entry != nil && entry!.count > 0
    
    // Check entitlements
    let entitlements = Bundle.main.infoDictionary?["com.apple.developer.nfc.readersession.formats"] as? [String]
    
    // NFC availability
    let ndefAvailable = NFCNDEFReaderSession.readingAvailable
    
    // Try to create a session (don't start it)
    let tagSession = NFCTagReaderSession(
      pollingOption: [.iso14443, .iso15693],
      delegate: self,
      queue: DispatchQueue.main
    )
    
    let canCreateTagSession = tagSession != nil
    
    // Create diagnostic data
    let diagnosticData: [String: Any] = [
      "ndefAvailable": ndefAvailable,
      "hasUsageDescription": hasUsageDescription,
      "entitlements": entitlements ?? [],
      "canCreateTagSession": canCreateTagSession,
      "bundleId": Bundle.main.bundleIdentifier ?? "unknown"
    ]
    
    // Convert to JSON string
    let jsonData = try JSONSerialization.data(withJSONObject: diagnosticData, options: .prettyPrinted)
    let jsonString = String(data: jsonData, encoding: .utf8) ?? "Failed to create JSON"
    
    // Return in the same format as before
    invoke.resolve(["value": jsonString])
  }
}

@_cdecl("init_plugin_nfc_debug")
func initPlugin() -> Plugin {
  return ExamplePlugin()
}
