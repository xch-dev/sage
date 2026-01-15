//
//  CryptoUtils.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 09/10/2019.
//  Copyright © 2019 Tangem AG. All rights reserved.
//

import Foundation
import CommonCrypto
import CryptoKit

public enum CryptoUtils {
    
    /**
     * Generates array of random bytes.
     * It is used, among other things, to generate helper private keys
     * (not the one for the blockchains, that one is generated on the card and does not leave the card).
     *
     * - Parameter count: length of the array that is to be generated.
     */
    public static func generateRandomBytes(count: Int) throws -> Data  {
        var bytes = [Byte](repeating: 0, count: count)
        let status = SecRandomCopyBytes(kSecRandomDefault, bytes.count, &bytes)
        if status == errSecSuccess {
            return Data(bytes)
        } else {
            throw TangemSdkError.failedToGenerateRandomSequence
        }
    }

    public static func crypt(operation: Int, algorithm: Int, options: Int, key: Data, dataIn: Data) throws -> Data {
        return try key.withUnsafeBytes { keyUnsafeRawBufferPointer in
            return try dataIn.withUnsafeBytes { dataInUnsafeRawBufferPointer in
                // Give the data out some breathing room for PKCS7's padding.
                let dataOutSize: Int = dataIn.count + kCCBlockSizeAES128*2
                let dataOut = UnsafeMutableRawPointer.allocate(byteCount: dataOutSize,
                                                               alignment: 1)
                defer { dataOut.deallocate() }
                var dataOutMoved: Int = 0
                let status = CCCrypt(CCOperation(operation), CCAlgorithm(algorithm),
                                     CCOptions(options),
                                     keyUnsafeRawBufferPointer.baseAddress, key.count,
                                     nil,
                                     dataInUnsafeRawBufferPointer.baseAddress, dataIn.count,
                                     dataOut, dataOutSize, &dataOutMoved)
                guard status == kCCSuccess else { throw TangemSdkError.cryptoUtilsError("CCCryptor error. Code: \(status)") }
                return Data(bytes: dataOut, count: dataOutMoved)
            }
        }
    }
}

fileprivate struct CustomSha256Digest: Digest {
    static var byteCount: Int { 32 }
    
    let hash: Data
    
    func withUnsafeBytes<R>(_ body: (UnsafeRawBufferPointer) throws -> R) rethrows -> R {
       try hash.withUnsafeBytes(body)
    }
}

// MARK: - Constants
private extension CryptoUtils {
    enum Constants {
        static let p256CompressedKeySize = 33
        static let ed25519PrivateKeySize = 32
    }
}
