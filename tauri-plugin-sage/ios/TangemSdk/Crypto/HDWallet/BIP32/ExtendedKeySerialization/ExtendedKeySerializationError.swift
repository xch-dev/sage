//
//  ExtendedKeySerializationError.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 16.01.2023.
//  Copyright © 2023 Tangem AG. All rights reserved.
//

import Foundation

public enum ExtendedKeySerializationError: String, Error, LocalizedError {
    case wrongLength
    case decodingFailed
    case wrongVersion
    case wrongKey

    public var errorDescription: String? { rawValue }
}
