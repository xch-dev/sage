//
//  SignCommand.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 03/10/2019.
//  Copyright © 2019 Tangem AG. All rights reserved.
//

import Foundation

/// Response for `SignCommand`.
public struct SignResponse: JSONStringConvertible {
    /// CID, Unique Tangem card ID number
    public let cardId: String
    /// Signed hashes (array of resulting signatures)
    public let signatures: [Data]
    /// Total number of signed  hashes returned by the wallet since its creation. COS: 1.16+
    public let totalSignedHashes: Int?
}

/// Signs transaction hashes using a wallet private key, stored on the card.
class SignCommand: Command {
    typealias Response = SignResponse
    typealias CommandResponse = PartialSignResponse

    var requiresPasscode: Bool { return true }
    
    private let walletPublicKey: Data

    private var chunkHashesHelper: ChunkedHashesContainer

    /// Command initializer
    /// - Parameters:
    ///   - hashes: Array of transaction hashes.
    ///   - walletPublicKey: Public key of the wallet, using for sign.
    ///   - derivationPath: Derivation path of the wallet. Optional. COS v. 4.28 and higher,
    init(hashes: [Data], walletPublicKey: Data) {
        self.walletPublicKey = walletPublicKey
        self.chunkHashesHelper = ChunkedHashesContainer(hashes: hashes)
    }
    
    deinit {
        Log.debug("SignCommand deinit")
    }
    
    func performPreCheck(_ card: Card) -> TangemSdkError? {
        guard let wallet = card.wallets[walletPublicKey] else {
            return .walletNotFound
        }
        
        //Before v4
        if let remainingSignatures = wallet.remainingSignatures,
           remainingSignatures == 0 {
            return .noRemainingSignatures
        }
        
        if let defaultSigningMethods = card.settings.defaultSigningMethods {
            if !defaultSigningMethods.contains(.signHash) {
                return .signHashesNotAvailable
            }
        }
        
        if card.firmwareVersion.doubleValue < 2.28, card.settings.securityDelay > 15000 {
            return .oldCard
        }
        
        return nil
    }
    
    func run(in session: CardSession, completion: @escaping CompletionResult<SignResponse>) {
        if chunkHashesHelper.isEmpty {
            completion(.failure(.emptyHashes))
            return
        }
        
        sign(in: session, completion: completion)
    }
    
    func mapError(_ card: Card?, _ error: TangemSdkError) -> TangemSdkError {        
        if case .unknownStatus = error {
            return .nfcStuck
        }
        
        return error
    }
    
    func sign(in session: CardSession, completion: @escaping CompletionResult<SignResponse>) {
        if chunkHashesHelper.chunksCount > 1 {
            session.viewDelegate.showAlertMessage("sign_multiple_chunks_part".localized([chunkHashesHelper.currentChunkIndex + 1, chunkHashesHelper.chunksCount]))
        }
        
        transceive(in: session) { result in
            switch result {
            case .success(let response):
                self.chunkHashesHelper.addSignedChunk(response.signedChunk)

                if self.chunkHashesHelper.currentChunkIndex >= self.chunkHashesHelper.chunksCount {
                    session.environment.card?.wallets[self.walletPublicKey]?.totalSignedHashes = response.totalSignedHashes

                    do {
                        let signatures = try self.processSignatures(with: session.environment)

                        if let remainingSignatures = session.environment.card?.wallets[self.walletPublicKey]?.remainingSignatures {
                            session.environment.card?.wallets[self.walletPublicKey]?.remainingSignatures = remainingSignatures - signatures.count
                        }

                        completion(.success(SignResponse(cardId: response.cardId,
                                                         signatures: signatures,
                                                         totalSignedHashes: response.totalSignedHashes)))
                    } catch {
                        completion(.failure(error.toTangemSdkError()))
                    }
                    
                    return
                }

                if let firmwareVersion = session.environment.card?.firmwareVersion,
                   firmwareVersion < .keysImportAvailable {
                    session.restartPolling(silent: true)
                }

                self.sign(in: session, completion: completion)
            case .failure(let error):
                completion(.failure(error))
            }
        }
    }
    
    func serialize(with environment: SessionEnvironment) throws -> CommandApdu {
        guard let walletIndex = environment.card?.wallets[walletPublicKey]?.index else {
            throw TangemSdkError.walletNotFound
        }

        let chunk = try chunkHashesHelper.getCurrentChunk()
        
        let hashSize = chunk.hashSize
        let hashSizeData = hashSize > 255 ? hashSize.bytes2 : hashSize.byte

        let flattenHashes = Data(chunk.hashes.flatMap { $0.data })
        let tlvBuilder = try createTlvBuilder(legacyMode: environment.legacyMode)
            .append(.pin, value: environment.accessCode.value)
            .append(.pin2, value: environment.passcode.value)
            .append(.cardId, value: environment.card?.cardId)
            .append(.transactionOutHashSize, value: hashSizeData)
            .append(.transactionOutHash, value: flattenHashes)
            //Wallet index works only on COS v.4.0 and higher. For previous version index will be ignored
            .append(.walletIndex, value: walletIndex)
        
        if let cvc = environment.cvc {
            try tlvBuilder.append(.cvc, value: cvc)
        }
        
        return CommandApdu(.sign, tlv: tlvBuilder.serialize())
    }
    
    func deserialize(with environment: SessionEnvironment, from apdu: ResponseApdu) throws -> PartialSignResponse {
        guard let tlv = apdu.getTlvData(encryptionKey: environment.encryptionKey) else {
            throw TangemSdkError.deserializeApduFailed
        }
        
        let decoder = TlvDecoder(tlv: tlv)
        let chunk = try chunkHashesHelper.getCurrentChunk()

        let signatureBLOB: Data = try decoder.decode(.walletSignature)
        let signatures = splitSignatureBLOB(signatureBLOB, numberOfSignatures: chunk.hashes.count)

        let signedHashes = zip(chunk.hashes, signatures).map { (hash, signature) in
            SignedHash(
                index: hash.index,
                data: hash.data,
                signature: signature
            )
        }

        let signedChunk = SignedChunk(signedHashes: signedHashes)

        let response = PartialSignResponse(
            cardId: try decoder.decode(.cardId),
            signedChunk: signedChunk,
            totalSignedHashes: try decoder.decode(.walletSignedHashes)
        )

        return response
    }
    
    private func processSignatures(with environment: SessionEnvironment) throws -> [Data] {
        return chunkHashesHelper.getSignatures()
    }
    
    private func splitSignatureBLOB(_ signature: Data, numberOfSignatures: Int) -> [Data] {
        var signatures = [Data]()
        let signatureSize = signature.count / numberOfSignatures
        for index in 0..<numberOfSignatures {
            let offsetMin = index * signatureSize
            let offsetMax = offsetMin + signatureSize
            
            let sig = signature[offsetMin..<offsetMax]
            signatures.append(sig)
        }
        
        return signatures
    }
}

// MARK: - PartialSignResponse

struct PartialSignResponse {
    /// CID, Unique Tangem card ID number
    let cardId: String
    /// Signed hashes (array of resulting signatures)
    let signedChunk: SignedChunk
    /// Total number of signed  hashes returned by the wallet since its creation. COS: 1.16+
    let totalSignedHashes: Int?
}


