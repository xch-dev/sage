//
//  MnemonicError.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 01.03.2023.
//  Copyright © 2023 Tangem AG. All rights reserved.
//

import Foundation

public enum MnemonicError: Error {
    case invalidEntropyLength
    case invalidWordCount
    case invalidWordsFile
    case invalidCheksum
    case invalidMnemonic
    case mnenmonicCreationFailed
    case normalizationFailed
    case wrongWordCount
    case unsupportedLanguage
    case invalidWords(words: [String])
}
