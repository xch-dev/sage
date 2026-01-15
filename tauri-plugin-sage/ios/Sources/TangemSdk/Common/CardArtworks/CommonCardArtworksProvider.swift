//
//  CommonCardArtworksProvider.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 28/03/2025.
//  Copyright © 2025 Tangem AG. All rights reserved.
//

import Combine
import Foundation

/// Only for production cards
struct CommonCardArtworksProvider: CardArtworksProvider {
    private let cardId: String
    private let cardPublicKey: Data
    private let networkService: NetworkService

    init(
        cardId: String,
        cardPublicKey: Data,
        networkService: NetworkService
    ) {
        self.cardId = cardId
        self.cardPublicKey = cardPublicKey
        self.networkService = networkService
    }

    public func loadArtworks() async throws -> Artworks {
        let endpoint = TangemEndpoint.artworks(cardId: cardId, cardPublicKey: cardPublicKey)
        let response: ArtworksResponse = try await networkService.request(endpoint)

        let imageLargeRequest = URLRequest(url: response.imageLargeUrl)
        let imageSmallRequest = response.imageSmallUrl.map { URLRequest(url: $0) }

        async let imageLargeResponse = try? networkService.requestData(request: imageLargeRequest)
        async let imageSmallResponse = imageSmallRequest == nil ? nil : try? networkService.requestData(request: imageSmallRequest!)

        let (imageLargeData, imageSmallData) = await (imageLargeResponse, imageSmallResponse)

        guard let imageLargeData else {
            throw Error.imageLoadingFailed
        }

        return Artworks(large: imageLargeData, small: imageSmallData)
    }
}

extension CommonCardArtworksProvider {
    enum SignaturePrefixBuilder {
        case smallImage
        case largeImage

        fileprivate func prefix(for url: URL) -> Data {
            let artworkId = getArtworkID(from: url)
            return prefixString(artworkId: artworkId).data(using: .utf8)!
        }

        private func getArtworkID(from url: URL) -> String {
            NSString(string: url.lastPathComponent).deletingPathExtension
        }

        private func prefixString(artworkId: String) -> String {
            switch self {
            case .smallImage:
                return "artwork|small|\(artworkId)|"
            case .largeImage:
                return "artwork|large|\(artworkId)|"
            }
        }
    }
}

extension CommonCardArtworksProvider {
    enum Error: String, Swift.Error, LocalizedError {
        case imageLoadingFailed

        public var errorDescription: String? {
            rawValue
        }
    }
}
