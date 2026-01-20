//
//  ScanTask.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 03/10/2019.
//  Copyright © 2019 Tangem AG. All rights reserved.
//

import Foundation

/// Task that allows to read Tangem card and verify its private key.
/// Returns data from a Tangem card after successful completion of `ReadCommand` and `AttestWalletKeyTask`, subsequently.
public final class ScanTask: CardSessionRunnable {
    public var shouldAskForAccessCode: Bool { false }

    public init() {
        
    }

    deinit {
        Log.debug("ScanTask deinit")
    }
    
    public func run(in session: CardSession, completion: @escaping CompletionResult<Card>) {
        guard let card = session.environment.card  else {
            completion(.failure(.missingPreflightRead))
            return
        }
        
        //We have to retrieve passcode status information for cards with COS before v4.01 with checkUserCodes command for backward compatibility.
        //checkUserCodes command for cards with COS <=1.19 not supported because of persistent SD.
        //We cannot run checkUserCodes command for cards whose `isRemovingUserCodesAllowed` is set to false because of an error
        if card.firmwareVersion < .isPasscodeStatusAvailable
            && card.firmwareVersion.doubleValue > 1.19
            && card.settings.isRemovingUserCodesAllowed {
            checkUserCodes(session, completion)
        }
    }

    private func checkUserCodes(_ session: CardSession, _ completion: @escaping CompletionResult<Card>) {
        let checkCodesCommand = CheckUserCodesCommand()
        checkCodesCommand.run(in: session) { result in
            switch result {
            case .success(let response):
                session.environment.card?.isPasscodeSet = response.isPasscodeSet
                guard let card = session.environment.card  else {
                    completion(.failure(.missingPreflightRead))
                    return
                }
                completion(.success(card))
            case .failure(let error):
                completion(.failure(error))
            }

            withExtendedLifetime(checkCodesCommand) {}
        }
    }
}
