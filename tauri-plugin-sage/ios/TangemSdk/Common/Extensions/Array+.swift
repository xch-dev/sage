//
//  Array+.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 19.08.2021.
//  Copyright © 2021 Tangem AG. All rights reserved.
//

import Foundation

extension Array {
    func chunked(into size: Int) -> [[Element]] {
        return stride(from: 0, to: count, by: size).map {
            Array(self[$0 ..< Swift.min($0 + size, count)])
        }
    }
    
    func reversedChunked(into size: Int) -> [[Element]] {
        return stride(from: count, to: 0, by: -size).map {
            Array(self[Swift.max($0 - size, 0) ..< $0])
        }
    }
}
