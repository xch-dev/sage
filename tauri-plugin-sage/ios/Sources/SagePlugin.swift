import CoreNFC
import ObjectiveC
import SwiftRs
import Tauri
import UIKit
import WebKit

private let maxSnapshotWidth: CGFloat = 720
private let maxSnapshotPixelArea: CGFloat = 1_000_000
private var sageWebviewRegistrationKey: UInt8 = 0

private func isSageAppWebviewLabel(_ label: String) -> Bool {
  for prefix in ["app-", "system-app-"] {
    if label.hasPrefix(prefix) && label.count > prefix.count {
      return true
    }
  }

  return false
}

private struct SetWebviewBoundsArgs: Decodable {
  let label: String
  let x: Double
  let y: Double
  let width: Double
  let height: Double
}

private struct SnapshotWebviewArgs: Decodable {
  let label: String
  let width: Double
}

private final class WeakWebview {
  weak var value: WKWebView?
  let identity: ObjectIdentifier

  init(_ value: WKWebView) {
    self.value = value
    self.identity = ObjectIdentifier(value)
  }
}

private final class SageWebviewRegistration {
  let label: String
  let identity: ObjectIdentifier

  init(label: String, webview: WKWebView) {
    self.label = label
    self.identity = ObjectIdentifier(webview)
  }

  deinit {
    let label = self.label
    let identity = self.identity
    let unregister = {
      SageWebviewRegistry.shared.unregister(label: label, identity: identity)
    }

    if Thread.isMainThread {
      unregister()
    } else {
      DispatchQueue.main.async(execute: unregister)
    }
  }
}

/// Tauri owns the WKWebViews. This registry only gives the native Sage shell a
/// weak, label-bound handle for layout and trusted tab previews.
private final class SageWebviewRegistry {
  static let shared = SageWebviewRegistry()

  private var webviews: [String: WeakWebview] = [:]

  private init() {}

  func register(_ webview: WKWebView, label: String) {
    precondition(Thread.isMainThread)

    guard isSageAppWebviewLabel(label) else {
      return
    }

    let registration = SageWebviewRegistration(label: label, webview: webview)
    objc_setAssociatedObject(
      webview,
      &sageWebviewRegistrationKey,
      registration,
      .OBJC_ASSOCIATION_RETAIN_NONATOMIC
    )

    pruneReleasedWebviews()
    webviews[label] = WeakWebview(webview)
  }

  func unregister(label: String, identity: ObjectIdentifier) {
    precondition(Thread.isMainThread)

    guard let entry = webviews[label], entry.identity == identity else {
      return
    }

    webviews.removeValue(forKey: label)
  }

  func webview(label: String) -> WKWebView? {
    precondition(Thread.isMainThread)

    guard isSageAppWebviewLabel(label) else {
      return nil
    }

    guard let entry = webviews[label] else {
      return nil
    }

    guard let webview = entry.value else {
      webviews.removeValue(forKey: label)
      return nil
    }

    return webview
  }

  private func pruneReleasedWebviews() {
    webviews = webviews.filter { $0.value.value != nil }
  }
}

class SagePlugin: Plugin, NFCNDEFReaderSessionDelegate {
  var session: Session?

  @objc public func isNdefAvailable(_ invoke: Invoke) throws {
    invoke.resolve(["available": NFCNDEFReaderSession.readingAvailable])
  }

  @objc public func getNdefPayloads(_ invoke: Invoke) throws {
    if !NFCNDEFReaderSession.readingAvailable {
      invoke.reject("NFC NDEF reading unavailable")
      return
    }

    self.startScanSession(invoke)
  }

  @objc public func setWebviewBounds(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(SetWebviewBoundsArgs.self)

    DispatchQueue.main.async {
      guard isSageAppWebviewLabel(args.label) else {
        invoke.reject("Invalid Sage app webview label")
        return
      }

      guard
        args.x.isFinite,
        args.y.isFinite,
        args.width.isFinite,
        args.height.isFinite,
        args.width > 0,
        args.height > 0
      else {
        invoke.reject("Webview bounds must be finite and positive")
        return
      }

      guard let webview = SageWebviewRegistry.shared.webview(label: args.label) else {
        invoke.reject("Unknown or closed webview: \(args.label)")
        return
      }

      guard let superview = webview.superview else {
        invoke.reject("Webview is not attached: \(args.label)")
        return
      }

      let hostBounds = superview.bounds.standardized
      let requestedFrame = CGRect(
        x: args.x,
        y: args.y,
        width: args.width,
        height: args.height
      ).standardized
      let boundedFrame = requestedFrame.intersection(hostBounds)

      guard
        !boundedFrame.isNull,
        boundedFrame.width >= 1,
        boundedFrame.height >= 1,
        boundedFrame.origin.x.isFinite,
        boundedFrame.origin.y.isFinite,
        boundedFrame.width.isFinite,
        boundedFrame.height.isFinite
      else {
        invoke.reject("Webview bounds are outside the host view")
        return
      }

      webview.translatesAutoresizingMaskIntoConstraints = true
      webview.autoresizingMask = []
      webview.clipsToBounds = true
      webview.frame = boundedFrame.integral.intersection(hostBounds)

      invoke.resolve()
    }
  }

  @objc public func snapshotWebview(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(SnapshotWebviewArgs.self)

    DispatchQueue.main.async {
      guard isSageAppWebviewLabel(args.label) else {
        invoke.reject("Invalid Sage app webview label")
        return
      }

      guard args.width.isFinite, args.width > 0 else {
        invoke.reject("Snapshot width must be finite and positive")
        return
      }

      guard let webview = SageWebviewRegistry.shared.webview(label: args.label) else {
        invoke.reject("Unknown or closed webview: \(args.label)")
        return
      }

      let bounds = webview.bounds.standardized

      guard
        webview.window != nil,
        bounds.width >= 1,
        bounds.height >= 1,
        bounds.width.isFinite,
        bounds.height.isFinite
      else {
        invoke.reject("Webview is not visible or has invalid bounds: \(args.label)")
        return
      }

      let aspectRatio = bounds.height / bounds.width
      let areaLimitedWidth = sqrt(maxSnapshotPixelArea / aspectRatio)

      guard areaLimitedWidth >= 1, areaLimitedWidth.isFinite else {
        invoke.reject("Webview aspect ratio is too large to snapshot safely")
        return
      }

      let snapshotWidth = min(CGFloat(args.width), maxSnapshotWidth, areaLimitedWidth)

      let configuration = WKSnapshotConfiguration()
      configuration.rect = bounds
      configuration.snapshotWidth = NSNumber(value: snapshotWidth)
      configuration.afterScreenUpdates = false

      webview.takeSnapshot(with: configuration) { image, error in
        if let error = error {
          invoke.reject("Failed to snapshot \(args.label): \(error)")
          return
        }

        guard let image = image, let data = image.pngData() else {
          invoke.reject("Failed to encode snapshot for \(args.label)")
          return
        }

        invoke.resolve([
          "dataUrl": "data:image/png;base64,\(data.base64EncodedString())"
        ])
      }
    }
  }

  private func startScanSession(_ invoke: Invoke) {
    let nfcSession = NFCNDEFReaderSession(
      delegate: self,
      queue: DispatchQueue.main,
      invalidateAfterFirstRead: true
    )

    nfcSession.alertMessage = "Scan an NFC tag"
    nfcSession.begin()

    self.session = Session(nfcSession, invoke)
  }

  func readerSession(_ session: NFCNDEFReaderSession, didDetectNDEFs messages: [NFCNDEFMessage]) {
    let message = messages.first!
    self.session?.invoke.resolve(["payloads": ndefMessagePayloads(message)])
  }

  func readerSession(_ session: NFCNDEFReaderSession, didDetect tags: [NFCNDEFTag]) {
    let tag = tags.first!

    session.connect(
      to: tag,
      completionHandler: { [self] (error) in
        if let error = error {
          self.closeSession(session, error: "cannot connect to tag: \(error)")
        } else {
          self.processTag(session: session, tag: tag)
        }
      }
    )
  }

  func readerSession(_ session: NFCNDEFReaderSession, didInvalidateWithError error: Error) {
    if (error as NSError).code == NFCReaderError.Code.readerSessionInvalidationErrorFirstNDEFTagRead.rawValue {
      Logger.debug("readerSessionInvalidationErrorFirstNDEFTagRead")
    } else {
      Logger.error("NDEF reader session error \(error)")
      self.session?.invoke.reject("session invalidated with error: \(error)")
    }
  }

  private func closeSession(_ session: NFCReaderSession) {
    session.invalidate()
    self.session = nil
  }

  private func closeSession(_ session: NFCReaderSession, error: String) {
    session.invalidate(errorMessage: error)
    self.session = nil
  }

  private func processTag<T: NFCNDEFTag>(session: NFCReaderSession, tag: T) {
    tag.queryNDEFStatus(completionHandler: {
      [self] (status, capacity, error) in
      if let error = error {
        self.closeSession(session, error: "cannot connect to tag: \(error)")
      } else {
        self.readNDEFTag(
          session: session,
          status: status,
          tag: tag
        )
      }
    })
  }

  private func readNDEFTag<T: NFCNDEFTag>(
    session: NFCReaderSession,
    status: NFCNDEFStatus,
    tag: T
  ) {
    switch status {
    case .notSupported:
      self.resolveInvoke(nil)
      self.closeSession(session)
      return
    default:
      break
    }

    tag.readNDEF(completionHandler: {
      [self] (message, error) in
      if let error = error {
        let code = (error as NSError).code
        if code != 403 {
          self.closeSession(session, error: "Failed to read: \(error)")
          return
        }
      }

      session.alertMessage = "NFC tag successfully scanned"

      self.resolveInvoke(message)
      self.closeSession(session)
    })
  }

  private func resolveInvoke(_ message: NFCNDEFMessage?) {
    var data: JsonObject = [:]

    if let message = message {
      data["payloads"] = ndefMessagePayloads(message)
    } else {
      data["payloads"] = []
    }

    self.session?.invoke.resolve(data)
  }

  private func ndefMessagePayloads(_ message: NFCNDEFMessage) -> [[UInt8]] {
    var payloads: [[UInt8]] = []
    
    for record in message.records {
      payloads.append(byteArrayFromData(record.payload))
    }

    return payloads
  }

  private func byteArrayFromData(_ data: Data) -> [UInt8] {
    var arr: [UInt8] = []
    for b in data {
      arr.append(b)
    }
    return arr
  }
}

class Session {
  let nfcSession: NFCReaderSession
  let invoke: Invoke
  var tagStatus: NFCNDEFStatus?
  var tag: NFCNDEFTag?

  init(_ nfcSession: NFCReaderSession, _ invoke: Invoke) {
    self.nfcSession = nfcSession
    self.invoke = invoke
  }
}

@_cdecl("init_plugin_sage")
func initPlugin() -> Plugin {
  return SagePlugin()
}

@_cdecl("sage_register_webview")
func sageRegisterWebview(_ webview: WKWebView, _ label: SRString) {
  let label = label.toString()

  if Thread.isMainThread {
    SageWebviewRegistry.shared.register(webview, label: label)
  } else {
    DispatchQueue.main.async {
      SageWebviewRegistry.shared.register(webview, label: label)
    }
  }
}
