import CoreNFC
import SwiftRs
import Tauri
import UIKit
import WebKit

class SagePlugin: Plugin, NFCNDEFReaderSessionDelegate {
  var session: Session?

    @objc public func testTangem(_ invoke: Invoke) throws {
        invoke.resolve(["output": "Hello, world!"])
    }
    
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
