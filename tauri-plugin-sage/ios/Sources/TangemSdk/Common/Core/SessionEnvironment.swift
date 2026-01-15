//
//  SessionEnvironment.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 02/10/2019.
//  Copyright © 2019 Tangem AG. All rights reserved.
//

import Foundation

/// Contains data relating to a Tangem card. It is used in constructing all the commands,
/// and commands can return modified `SessionEnvironment`.
public struct SessionEnvironment {
    /// Current card, read by preflight `Read` command
    public internal(set) var card: Card? = nil
    
    /// Current card's wallet data, read by preflight `Read` command
    public internal(set) var walletData: WalletData? = nil
    
    public internal(set) var config: Config
    
    var encryptionMode: EncryptionMode = .none
    
    var encryptionKey: Data? = nil
    
    var currentSecurityDelay: Float? = nil
    
    var cvc: Data? = nil //todo: remove
    
    var accessCode: UserCode = .init(.accessCode)
    
    var passcode: UserCode = .init(.passcode)
    
    var legacyMode: Bool { config.legacyMode ?? NFCUtils.isPoorNfcQualityDevice }
    
    func isUserCodeSet(_ type: UserCodeType) -> Bool {
        switch type {
        case .accessCode:
            return accessCode.value != type.defaultValue.sha256()
        case .passcode:
            return passcode.value != type.defaultValue.sha256()
        }
    }

    mutating func resetCodes() {
        accessCode = .init(.accessCode)
        passcode = .init(.passcode)
    }
}
