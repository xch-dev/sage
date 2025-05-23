//
//  BIP44.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 03.08.2021.
//  Copyright © 2021 Tangem AG. All rights reserved.
//

import Foundation

public struct BIP44 {
    public let coinType: UInt32
    public let account: UInt32
    public let change: Chain
    public let addressIndex: UInt32
    
    public static let purpose: UInt32 = 44
    
    public init(coinType: UInt32, account: UInt32, change: Chain, addressIndex: UInt32) {
        self.coinType = coinType
        self.account = account
        self.change = change
        self.addressIndex = addressIndex
    }
    
    public init(coinType: UInt32) {
        self.coinType = coinType
        self.account = 0
        self.change = .external
        self.addressIndex = 0
    }
    
    /// Build path
    /// - Returns: Path according BIP32
    public func buildPath() -> DerivationPath {
        let nodes: [DerivationNode] = [.hardened(BIP44.purpose),
                                       .hardened(coinType),
                                       .hardened(account),
                                       .nonHardened(change.index),
                                       .nonHardened(addressIndex)]
        return DerivationPath(nodes: nodes)
    }
}

public extension BIP44 {
    enum Chain {
        case external
        case `internal`
        
        public var index: UInt32 {
            switch self {
            case .external:
                return 0
            case .internal:
                return 1
            }
        }
    }
}
