//
//  CCCCryptorStatus+Error.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 27.05.2020.
//  Copyright © 2020 Tangem AG. All rights reserved.
//

import Foundation
import CommonCrypto

extension CCCryptorStatus: Error, LocalizedError {
    public var errorDescription: String? {
        return "CCCryptor error. Code: \(self)"
    }
}
