//
//  Int+.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 27/09/2019.
//  Copyright © 2019 Tangem AG. All rights reserved.
//

import Foundation

extension Int {
    /// Convert int to byte, truncatingIfNeeded
    public var byte: Data {
        return Data([Byte(truncatingIfNeeded: self)])
    }
    
    /// return 2 bytes of integer. BigEndian format
    public var bytes2: Data {
        let clamped = UInt16(clamping: self).bigEndian
        let data = withUnsafeBytes(of: clamped) { Data($0) }
        return data
    }
    
    /// return 4 bytes of integer. BigEndian format
    public var bytes4: Data {
        let clamped = UInt32(clamping: self).bigEndian
        let data = withUnsafeBytes(of: clamped) { Data($0) }
        return data
    }
    
    /// return 8 bytes of integer. BigEndian  format
    public var bytes8: Data {
        let data = withUnsafeBytes(of: self.bigEndian) { Data($0) }
        return data
    }
}

extension UInt64 {
    public var bytes8LE: Data{
        let data = withUnsafeBytes(of: self) { Data($0) }
        return data
    }
}

extension UInt32 {
    public var bytes4: Data {
        let data = withUnsafeBytes(of: self.bigEndian) { Data($0) }
        return data
    }
}
