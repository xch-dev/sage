//
//  CardManager.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 03/10/2019.
//  Copyright © 2019 Tangem AG. All rights reserved.
//

import Foundation
import CoreNFC
import Combine

/// The main interface of Tangem SDK that allows your app to communicate with Tangem cards.
public final class TangemSdk {
    /// Configuration of the SDK. Do not change the default values unless you know what you are doing
    public var config = Config()
    
    private let reader: CardReader
    private let viewDelegate: SessionViewDelegate
    private var cardSession: CardSession?

    /// Default initializer
    /// - Parameters:
    ///   - cardReader: An interface that is responsible for NFC connection and transfer of data to and from the Tangem Card.
    ///   If nil, its default implementation will be used
    ///   - viewDelegate:  An interface that allows interaction with users and shows relevant UI.
    ///   If nil, its default implementation will be used
    ///   - config: Allows to change a number of parameters for communication with Tangem cards.
    ///   Do not change the default values unless you know what you are doing.
    public init(
        cardReader: CardReader? = nil,
        viewDelegate: SessionViewDelegate? = nil,
        config: Config = Config()
    ) {
        let reader = cardReader ?? NFCReader()
        self.reader = reader
        self.viewDelegate = viewDelegate ?? DefaultSessionViewDelegate(reader: reader, style: config.style)
        self.config = config
    }
    
    deinit {
        Log.debug("TangemSdk deinit")
    }
}

//MARK: - Card operations
public extension TangemSdk {
    //MARK: - Common
    
    /// To start using any card, you first need to read it using the `scanCard()` method.
    /// This method launches an NFC session, and once it’s connected with the card,
    /// it obtains the card data. Optionally, if the card contains a wallet (private and public key pair),
    /// it proves that the wallet owns a private key that corresponds to a public one.
    /// After successfull card scan, SDK will attempt to verify release cards online with Tangem backend.
    ///
    /// - Parameters:
    ///   - initialMessage: A custom description that shows at the beginning of the NFC session. If nil, default message will be used
    ///   - networkService: Allows to customize a network layer
    ///   - completion: Returns `Swift.Result<Card,TangemSdkError>`
    func scanCard(initialMessage: Message? = nil,
                  networkService: NetworkService,
                  completion: @escaping CompletionResult<Card>) {
        startSession(with: ScanTask(networkService: networkService), cardId: nil, initialMessage: initialMessage, completion: completion)
    }
    
    /// This method allows you to sign one hash and will return a corresponding signature.
    /// Please note that Tangem cards usually protect the signing with a security delay
    /// that may last up to 45 seconds, depending on a card.
    /// It is for `SessionViewDelegate` to notify users of security delay.
    ///
    /// - Note: `WalletIndex` available for cards with COS v.4.0 and higher
    /// - Parameters:
    ///   - hash: Transaction hash for sign by card.
    ///   - walletPublicKey: Public key of wallet that should sign hash.
    ///   - cardId: CID, Unique Tangem card ID number
    ///   - derivationPath: Derivation path of the wallet. Optional
    ///   - initialMessage: A custom description that shows at the beginning of the NFC session. If nil, default message will be used
    ///   - completion: Returns  `Swift.Result<SignHashResponse,TangemSdkError>`
    func sign(hash: Data,
              walletPublicKey: Data,
              cardId: String? = nil,
              initialMessage: Message? = nil,
              completion: @escaping CompletionResult<SignHashResponse>) {
        let command = SignHashCommand(hash: hash, walletPublicKey: walletPublicKey)
        startSession(with: command,
                     cardId: cardId,
                     initialMessage: initialMessage,
                     completion: completion)
    }
    
    /// This method allows you to sign multiple hashes.
    /// Simultaneous signing of array of hashes in a single `SignCommand` is required to support
    /// Bitcoin-type multi-input blockchains (UTXO).
    /// The `SignCommand` will return a corresponding array of signatures.
    /// Please note that Tangem cards usually protect the signing with a security delay
    /// that may last up to 45 seconds, depending on a card.
    /// It is for `SessionViewDelegate` to notify users of security delay.
    ///
    /// - Note: `WalletIndex` available for cards with COS v. 4.0 and higher
    /// - Parameters:
    ///   - hashes: Array of transaction hashes. It can be from one or up to ten hashes of the same length.
    ///   - walletPublicKey: Public key of wallet that should sign hashes.
    ///   - cardId: CID, Unique Tangem card ID number
    ///   - derivationPath: Derivation path of the wallet. Optional. COS v. 4.28 and higher,
    ///   - initialMessage: A custom description that shows at the beginning of the NFC session. If nil, default message will be used
    ///   - completion: Returns  `Swift.Result<SignHashesResponse,TangemSdkError>`
    func sign(hashes: [Data],
              walletPublicKey: Data,
              cardId: String? = nil,
              initialMessage: Message? = nil,
              completion: @escaping CompletionResult<SignHashesResponse>) {
        let command = SignCommand(hashes: hashes, walletPublicKey: walletPublicKey)
        startSession(with: command,
                     cardId: cardId,
                     initialMessage: initialMessage,
                     completion: completion)
    }
    
    /// This command deletes all wallet data. If Is_Reusable flag is enabled during personalization,
    /// the card changes state to ‘Empty’ and a new wallet can be created by `CREATE_WALLET` command.
    /// If Is_Reusable flag is disabled, the card switches to ‘Purged’ state.
    /// ‘Purged’ state is final, it makes the card useless.
    ///
    /// - Note: Wallet index available for cards with COS v.4.0 or higher
    /// - Parameters:
    ///   - walletPublicKey: Public key of wallet that should be purged.
    ///   - cardId: CID, Unique Tangem card ID number.
    ///   - initialMessage: A custom description that shows at the beginning of the NFC session. If nil, default message will be used
    ///   - completion: Returns `Swift.Result<SuccessResponse,TangemSdkError>`
    func purgeWallet(walletPublicKey: Data,
                     cardId: String,
                     initialMessage: Message? = nil,
                     completion: @escaping CompletionResult<SuccessResponse>) {
        startSession(with: PurgeWalletCommand(publicKey: walletPublicKey), cardId: cardId, initialMessage: initialMessage, completion: completion)
    }
    
    /// Set or change card's access code
    /// - Parameters:
    ///   - accessCode: Access code to set. If nil, the user will be prompted to enter code before operation
    ///   - cardId: CID, Unique Tangem card ID number.
    ///   - initialMessage: A custom description that shows at the beginning of the NFC session. If nil, default message will be used
    ///   - completion: Returns `Swift.Result<UserCodeCommandResponse,TangemSdkError>`
    func setAccessCode(_ accessCode: String? = nil,
                       cardId: String,
                       initialMessage: Message? = nil,
                       completion: @escaping CompletionResult<SuccessResponse>) {
        let command = SetUserCodeCommand(accessCode: accessCode)
        startSession(with: command, cardId: cardId, initialMessage: initialMessage, completion: completion)
    }
    
    /// Set or change card's passcode
    /// - Parameters:
    ///   - passcode: Passcode to set. If nil, the user will be prompted to enter code before operation
    ///   - cardId: CID, Unique Tangem card ID number.
    ///   - initialMessage: A custom description that shows at the beginning of the NFC session. If nil, default message will be used
    ///   - completion: Returns `Swift.Result<SuccessResponse,TangemSdkError>`
    func setPasscode(_ passcode: String? = nil,
                     cardId: String,
                     initialMessage: Message? = nil,
                     completion: @escaping CompletionResult<SuccessResponse>) {
        let command = SetUserCodeCommand(passcode: passcode)
        startSession(with: command, cardId: cardId, initialMessage: initialMessage, completion: completion)
    }
    
    /// Reset all user codes
    /// - Parameters:
    ///   - cardId: CID, Unique Tangem card ID number.
    ///   - initialMessage: A custom description that shows at the beginning of the NFC session. If nil, default message will be used
    ///   - completion: Returns `Swift.Result<SuccessResponse,TangemSdkError>`
    func resetUserCodes(cardId: String,
                        initialMessage: Message? = nil,
                        completion: @escaping CompletionResult<SuccessResponse>) {
        startSession(with: SetUserCodeCommand.resetUserCodes, cardId: cardId, initialMessage: initialMessage, completion: completion)
    }

    /// Set if card allowed to reset user code
    /// - Parameters:
    ///   - isAllowed:Is this card can reset user codes on tte other linked card or not
    ///   - cardId: CID, Unique Tangem card ID number.
    ///   - initialMessage: A custom description that shows at the beginning of the NFC session. If nil, default message will be used
    ///   - completion: Returns `Swift.Result<SuccessResponse,TangemSdkError>`
    func setUserCodeRecoveryAllowed(_ isAllowed: Bool,
                                    cardId: String,
                                    initialMessage: Message? = nil,
                                    completion: @escaping CompletionResult<SuccessResponse>) {
        let task = SetUserCodeRecoveryAllowedTask(isAllowed: isAllowed)
        startSession(with: task, cardId: cardId, initialMessage: initialMessage, completion: completion)
    }
}

//MARK: - Session start
extension TangemSdk {
    /// Allows running a custom bunch of commands in one NFC Session by creating a custom task. Tangem SDK will start a card session, perform preflight `Read` command,
    /// invoke the `run ` method of `CardSessionRunnable` and close the session.
    /// You can find the current card in the `environment` property of the `CardSession`
    /// - Parameters:
    ///   - runnable: A custom task, adopting `CardSessionRunnable` protocol
    ///   - completion: Standart completion handler. Invoked on the main thread. `(Swift.Result<CardSessionRunnable.Response, TangemSdkError>) -> Void`.
    public func startSession<T>(with runnable: T,
                                completion: @escaping CompletionResult<T.Response>)
    where T : CardSessionRunnable {
        do {
            try checkSession()
        } catch {
            completion(.failure(error.toTangemSdkError()))
            return
        }

        configure()
        cardSession = makeSession(with: config,
                                  filter: nil,
                                  initialMessage: nil,
                                  accessCode: nil)
        cardSession!.start(with: runnable, completion: completion)
    }

    /// Allows running a custom bunch of commands in one NFC Session by creating a custom task. Tangem SDK will start a card session, perform preflight `Read` command,
    /// invoke the `run ` method of `CardSessionRunnable` and close the session.
    /// You can find the current card in the `environment` property of the `CardSession`
    /// - Parameters:
    ///   - runnable: A custom task, adopting `CardSessionRunnable` protocol
    ///   - filter: Filters card to be read. Optional.
    ///   - initialMessage: A custom description that shows at the beginning of the NFC session. If nil, default message will be used.
    ///   - accessCode: Access code that will be used for a card session initialization. If nil, Tangem SDK will handle it automatically.
    ///   - completion: Standart completion handler. Invoked on the main thread. `(Swift.Result<CardSessionRunnable.Response, TangemSdkError>) -> Void`.
    public func startSession<T>(with runnable: T,
                                filter: SessionFilter?,
                                initialMessage: Message? = nil,
                                accessCode: String? = nil,
                                completion: @escaping CompletionResult<T.Response>)
    where T : CardSessionRunnable {
        do {
            try checkSession()
        } catch {
            completion(.failure(error.toTangemSdkError()))
            return
        }

        configure()
        cardSession = makeSession(with: config,
                                  filter: filter,
                                  initialMessage: initialMessage,
                                  accessCode: accessCode)
        cardSession!.start(with: runnable, completion: completion)
    }

    /// Allows running a custom bunch of commands in one NFC Session by creating a custom task. Tangem SDK will start a card session, perform preflight `Read` command,
    /// invoke the `run ` method of `CardSessionRunnable` and close the session.
    /// You can find the current card in the `environment` property of the `CardSession`
    /// - Parameters:
    ///   - runnable: A custom task, adopting `CardSessionRunnable` protocol
    ///   - cardId: CID, Unique Tangem card ID number. If not nil, the SDK will check that you tapped the card with this cardID and will return the `wrongCard` error otherwise
    ///   - initialMessage: A custom description that shows at the beginning of the NFC session. If nil, default message will be used.
    ///   - accessCode: Access code that will be used for a card session initialization. If nil, Tangem SDK will handle it automatically.
    ///   - completion: Standard completion handler. Invoked on the main thread. `(Swift.Result<CardSessionRunnable.Response, TangemSdkError>) -> Void`.
    public func startSession<T>(with runnable: T,
                                cardId: String? = nil,
                                initialMessage: Message? = nil,
                                accessCode: String? = nil,
                                completion: @escaping CompletionResult<T.Response>)
    where T : CardSessionRunnable {
        startSession(with: runnable,
                     filter: .init(from: cardId),
                     initialMessage: initialMessage,
                     accessCode: accessCode,
                     completion: completion)
    }
    
    /// Allows running  a custom bunch of commands in one NFC Session with lightweight closure syntax. Tangem SDK will start a card sesion and perform preflight `Read` command.
    /// - Parameters:
    ///   - cardId: CID, Unique Tangem card ID number. If not nil, the SDK will check that you tapped the  card with this cardID and will return the `wrongCard` error' otherwise
    ///   - initialMessage: A custom description that shows at the beginning of the NFC session. If nil, default message will be used
    ///   - accessCode: Access code that will be used for a card session initialization. If nil, Tangem SDK will handle it automatically.
    ///   - callback: At first, you should check that the `TangemSdkError` is not nil, then you can use the `CardSession` to interact with a card.
    ///   You can find the current card in the `environment` property of the `CardSession`
    ///   If you need to interact with UI, you should dispatch to the main thread manually
    public func startSession(cardId: String? = nil,
                             initialMessage: Message? = nil,
                             accessCode: String? = nil,
                             callback: @escaping (CardSession, TangemSdkError?) -> Void) {
        do {
            try checkSession()
        } catch {
            callback(cardSession!, error.toTangemSdkError())
            return
        }
        
        configure()
        cardSession = makeSession(with: config,
                                  filter: .init(from: cardId),
                                  initialMessage: initialMessage,
                                  accessCode: accessCode)
        cardSession?.start(callback)
    }
}

//MARK: - Private
extension TangemSdk {
    private func checkSession() throws {
        if let existingSession = cardSession, existingSession.state == .active  {
            throw TangemSdkError.busy
        }
    }
    
    private func configure() {
        Log.config = config.logConfig
    }
    
    private func makeAccessCodeRepository(with config: Config) -> AccessCodeRepository? {
        if case .alwaysWithBiometrics = config.accessCodeRequestPolicy,
           BiometricsUtil.isAvailable {
            return AccessCodeRepository()
        }

        Log.debug("Failed to initialize AccessCodeRepository. Biometrics is unavailable.")
        
        return nil
    }
    
    func makeSession(with config: Config,
                     filter: SessionFilter?,
                     initialMessage: Message?,
                     accessCode: String? = nil) -> CardSession {
        var env = SessionEnvironment(config: config)
        
        if let accessCode = accessCode {
            env.accessCode = .init(.accessCode, stringValue: accessCode)
        }
        
        return CardSession(environment: env,
                           filter: filter,
                           initialMessage: initialMessage,
                           cardReader: reader,
                           viewDelegate: viewDelegate,
                           accessCodeRepository: makeAccessCodeRepository(with: config))
    }
}
