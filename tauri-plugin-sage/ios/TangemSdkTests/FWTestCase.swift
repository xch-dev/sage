//
//  FWTestCase.swift
//  TangemSdkTests
//
//  Created by Alexander Osokin on 07.08.2023.
//  Copyright © 2023 Tangem AG. All rights reserved.
//

import Foundation

// MARK: - FWTestable

protocol FWTestCase {}

extension FWTestCase {
    func printEquals<T: Equatable>(_ lhs: T, _ rhs: T) {
        let equals = lhs == rhs
        print("🐞 \(equals ? "✅" : "❌")")
    }
}
