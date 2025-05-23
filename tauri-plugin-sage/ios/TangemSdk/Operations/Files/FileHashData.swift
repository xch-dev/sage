//
//  FileHashData.swift
//  TangemSdk
//
//  Created by Andrew Son on 10/20/20.
//  Copyright © 2020 Tangem AG. All rights reserved.
//

import Foundation

/// Helping structure that holds hashes and signatures required for writing files protected by issuer signature
public struct FileHashData: JSONStringConvertible {
    public let startingHash: Data
    public let finalizingHash: Data
    
    public var startingSignature: Data?
    public var finalizingSignature: Data?
    
    public init(startingHash: Data, startingSignature: Data?, finalizingHash: Data, finalizingSignature: Data?) {
        self.startingHash = startingHash
        self.startingSignature = startingSignature
        self.finalizingHash = finalizingHash
        self.finalizingSignature = finalizingSignature
    }
}
