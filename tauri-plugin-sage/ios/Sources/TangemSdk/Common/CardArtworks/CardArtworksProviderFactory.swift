//
//  CardArtworksProviderFactory.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 28/03/2025.
//  Copyright © 2025 Tangem AG. All rights reserved.
//

import Foundation

public struct CardArtworksProviderFactory {
    private let networkService: NetworkService

    public init(networkService: NetworkService) {
        self.networkService = networkService
    }

    public func makeArtworksProvider(for card: Card) -> CardArtworksProvider {
        return makeArtworksProvider(
            cardId: card.cardId,
            cardPublicKey: card.cardPublicKey,
            issuerPublicKey: card.issuer.publicKey,
            firmwareVersionType: card.firmwareVersion.type
        )
    }

    public func makeArtworksProvider(
        cardId: String,
        cardPublicKey: Data,
        issuerPublicKey: Data,
        firmwareVersionType: FirmwareVersion.FirmwareType
    ) -> CardArtworksProvider {

        if firmwareVersionType == .sdk {
            return DevCardArtworksProvider()
        }

        return CommonCardArtworksProvider(
            cardId: cardId,
            cardPublicKey: cardPublicKey,
            networkService: networkService
        )
    }
}


