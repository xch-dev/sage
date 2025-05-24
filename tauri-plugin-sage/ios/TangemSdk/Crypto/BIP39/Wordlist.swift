//
//  Wordlist.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 01.03.2023.
//  Copyright © 2023 Tangem AG. All rights reserved.
//

import Foundation

extension BIP39 {
    public enum Wordlist: CaseIterable {
        case en
    }
}

extension BIP39.Wordlist {
    /// This var reads a big array from a file
    public var words: [String] {
        (try? readWords(from: fileName)) ?? []
    }

    private var fileName: String {
        switch self {
        case .en:
            return "english"
        }
    }

    private func readWords(from fileName: String) throws -> [String] {
        return []
    }
}
