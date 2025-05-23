//
//  EntropyLength.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 01.03.2023.
//  Copyright © 2023 Tangem AG. All rights reserved.
//

import Foundation

public enum EntropyLength: Int, CaseIterable {
    case bits128 = 128
    case bits160 = 160
    case bits192 = 192
    case bits224 = 224
    case bits256 = 256

    public var wordCount: Int {
        switch self {
        case .bits128: return 12
        case .bits160: return 15
        case .bits192: return 18
        case .bits224: return 21
        case .bits256: return 24
        }
    }

    var cheksumBitsCount: Int {
        rawValue / 32
    }
}
