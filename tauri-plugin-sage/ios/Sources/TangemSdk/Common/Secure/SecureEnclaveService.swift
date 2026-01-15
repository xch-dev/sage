//
//  SecureEnclaveService.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 25.06.2021.
//  Copyright © 2021 Tangem AG. All rights reserved.
//

import Foundation
import LocalAuthentication

public struct SecureEnclaveService {
    private let config: Config

    public init(config: Config) {
        self.config = config
    }

    private func makeOrRestoreKey(tag: String) throws -> SecKey {
        if let restoredKey = restoreKey(tag: tag) {
            return restoredKey
        }

        return try makeKey(tag: tag)
    }

    private func makeKey(tag: String) throws -> SecKey {
        let accessControl = SecAccessControlCreateWithFlags(
            kCFAllocatorDefault,
            kSecAttrAccessibleWhenPasscodeSetThisDeviceOnly,
            config.secAccessFlags,
            nil)!

        let attributes: NSDictionary = [
            kSecAttrKeyType: kSecAttrKeyTypeECSECPrimeRandom,
            kSecAttrKeySizeInBits: 256,
            kSecAttrTokenID: kSecAttrTokenIDSecureEnclave,
            kSecPrivateKeyAttrs: [
                kSecAttrIsPermanent: true,
                kSecAttrApplicationTag: tag,
                kSecAttrAccessControl: accessControl,
            ]
        ]

        var error: Unmanaged<CFError>?
        guard let privateKey = SecKeyCreateRandomKey(attributes, &error) else {
            throw error!.takeRetainedValue() as Error
        }

        return privateKey
    }

    private func restoreKey(tag: String) -> SecKey? {
        var query: [String: Any] = [
            kSecClass as String                 : kSecClassKey,
            kSecAttrApplicationTag as String    : tag,
            kSecAttrKeyType as String           : kSecAttrKeyTypeECSECPrimeRandom,
            kSecReturnRef as String             : true,
        ]

        if let laContext = config.laContext {
            query[kSecUseAuthenticationContext as String] = laContext
        }

        var item: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &item)
        guard status == errSecSuccess else {
            return nil
        }

        return (item as! SecKey)
    }
}

// MARK: - Signing

public extension SecureEnclaveService {
    func sign(data: Data, keyTag: String) throws -> Data {
        let privateKey = try makeOrRestoreKey(tag: keyTag)

        let algorithm: SecKeyAlgorithm = .ecdsaSignatureMessageX962SHA512
        guard SecKeyIsAlgorithmSupported(privateKey, .sign, algorithm) else {
            throw SecureEnclaveServiceError.algorithmNotSupported
        }

        var error: Unmanaged<CFError>?
        guard let signature = SecKeyCreateSignature(privateKey, algorithm, data as CFData,  &error) as Data? else {
            let underliyingError = error!.takeRetainedValue() as Error
            throw SecureEnclaveServiceError.signingFailed(underliyingError: underliyingError)
        }

        return signature
    }

    func verify(signature: Data, message: Data, keyTag: String) throws -> Bool {
        let privateKey = try makeOrRestoreKey(tag: keyTag)

        guard let publicKey = SecKeyCopyPublicKey(privateKey) else {
            throw SecureEnclaveServiceError.publicKeyGenerationFailed
        }

        let algorithm: SecKeyAlgorithm = .ecdsaSignatureMessageX962SHA512
        guard SecKeyIsAlgorithmSupported(privateKey, .sign, algorithm) else {
            throw SecureEnclaveServiceError.algorithmNotSupported
        }


        var error: Unmanaged<CFError>?
        guard SecKeyVerifySignature(publicKey, algorithm, message as CFData, signature as CFData, &error) else {
            let verificationError = error!.takeRetainedValue() as Error
            Log.error(verificationError)
            return false
        }

        return true
    }
}


// MARK: - ECIES

public extension SecureEnclaveService {
    func encryptData(_ data: Data, keyTag: String) throws -> Data {
        let privateKey = try makeOrRestoreKey(tag: keyTag)

        guard let publicKey = SecKeyCopyPublicKey(privateKey) else {
            throw SecureEnclaveServiceError.publicKeyGenerationFailed
        }

        let algorithm: SecKeyAlgorithm = .eciesEncryptionCofactorVariableIVX963SHA256AESGCM
        guard SecKeyIsAlgorithmSupported(publicKey, .encrypt, algorithm) else {
            throw SecureEnclaveServiceError.algorithmNotSupported
        }

        var error: Unmanaged<CFError>?
        let encryptedData = SecKeyCreateEncryptedData(publicKey, algorithm, data as CFData, &error) as Data?

        guard let encryptedData else {
            let underliyingError = error!.takeRetainedValue() as Error
            throw SecureEnclaveServiceError.encryptionFailed(underliyingError: underliyingError)
        }

        return encryptedData
    }

    /// SecKeyCreateDecryptedData call is blocking when the used key
    /// is protected by biometry authentication.
    func decryptData(_ data: Data, keyTag: String) throws -> Data {
        let privateKey = try makeOrRestoreKey(tag: keyTag)

        guard let publicKey = SecKeyCopyPublicKey(privateKey) else {
            throw SecureEnclaveServiceError.publicKeyGenerationFailed
        }

        let algorithm: SecKeyAlgorithm = .eciesEncryptionCofactorVariableIVX963SHA256AESGCM
        guard SecKeyIsAlgorithmSupported(publicKey, .encrypt, algorithm) else {
            throw SecureEnclaveServiceError.algorithmNotSupported
        }

        var error: Unmanaged<CFError>?
        let decryptedData = SecKeyCreateDecryptedData(privateKey, algorithm, data as CFData, &error) as Data?

        guard let decryptedData else {
            let underliyingError = error!.takeRetainedValue() as Error
            throw SecureEnclaveServiceError.decryptionnFailed(underliyingError: underliyingError)
        }

        return decryptedData
    }
}

extension SecureEnclaveService {
    enum SecureEnclaveServiceError: Error {
        case publicKeyGenerationFailed
        case algorithmNotSupported
        case signingFailed(underliyingError: Error)
        case verificationFailed(underliyingError: Error)
        case encryptionFailed(underliyingError: Error)
        case decryptionnFailed(underliyingError: Error)
    }
}

public extension SecureEnclaveService {
    enum Config {
        case `default`
        case biometrics(LAContext)

        fileprivate var laContext: LAContext? {
            switch self {
            case .default:
                return nil
            case .biometrics(let context):
                return context
            }
        }

        fileprivate var secAccessFlags: SecAccessControlCreateFlags {
            switch self {
            case .default:
                return [.privateKeyUsage]
            case .biometrics:
                return [.privateKeyUsage, .biometryCurrentSet]
            }
        }
    }
}

// MARK: SecureStorageKey

extension SecureEnclaveService {
    func sign(data: Data, storageKey: SecureStorageKey) throws -> Data {
        try sign(data: data, keyTag: storageKey.rawValue)
    }

    func verify(signature: Data, message: Data, storageKey: SecureStorageKey) throws -> Bool {
        try verify(signature: signature, message: message, keyTag: storageKey.rawValue)
    }

    func encryptData(_ data: Data, storageKey: SecureStorageKey) throws -> Data {
        try encryptData(data, keyTag: storageKey.rawValue)
    }

    func decryptData(_ data: Data, storageKey: SecureStorageKey) throws -> Data {
        try decryptData(data, keyTag: storageKey.rawValue)
    }
}
