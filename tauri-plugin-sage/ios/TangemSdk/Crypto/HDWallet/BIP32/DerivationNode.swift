//
//  DerivationNode.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 02.08.2021.
//  Copyright © 2021 Tangem AG. All rights reserved.
//

import Foundation

public enum DerivationNode: Equatable, Hashable {
    case hardened(UInt32)
    case nonHardened(UInt32)
    
    public var pathDescription: String {
        switch self {
        case .hardened(let index):
            return "\(index)\(BIP32.Constants.hardenedSymbol)"
        case .nonHardened(let index):
            return "\(index)"
        }
    }
    
    public var index: UInt32 {
        switch self {
        case .hardened(let index):
            let hardenedIndex = index + BIP32.Constants.hardenedOffset
            return hardenedIndex
        case .nonHardened(let index):
            return index
        }
    }
    
    public var isHardened: Bool {
        switch self {
        case .hardened:
            return true
        case .nonHardened:
            return false
        }
    }
    
    public static func fromIndex(_ index: UInt32) -> DerivationNode {
        if index < BIP32.Constants.hardenedOffset {
            return .nonHardened(index)
        } else {
            return .hardened(index - BIP32.Constants.hardenedOffset)
        }
    }
}

extension DerivationNode {
    func serialize() -> Data {
        index.bytes4
    }
    
    static func deserialize(from data: Data) -> DerivationNode? {
        guard let intValue = data.toInt() else { return nil }

        let index = UInt32(intValue)

        if index >= BIP32.Constants.hardenedOffset {
            return .hardened(index - BIP32.Constants.hardenedOffset)
        }
        
        return .nonHardened(index)
    }
}
