//
//  InputStream+.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 27/09/2019.
//  Copyright © 2019 Tangem AG. All rights reserved.
//

import Foundation

extension InputStream {
    func readBytes(count: Int) -> Data? {
        var buffer: [Byte] = Array(repeating: 0x00, count: count)
        let bytesRead = self.read(&buffer, maxLength: count)
        return bytesRead == count ? Data(buffer) : nil
    }
    
    func readByte() -> Byte? {
        return readBytes(count: 1)?.first
    }
}
