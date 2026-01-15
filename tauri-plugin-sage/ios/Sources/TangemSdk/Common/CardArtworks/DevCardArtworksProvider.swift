//
//  DevCardArtworksProvider.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 25/04/2025.
//  Copyright © 2025 Tangem AG. All rights reserved.
//

import Foundation

struct DevCardArtworksProvider: CardArtworksProvider {
    func loadArtworks() async throws -> Artworks {
        return Artworks(large: Data(), small: Data())
    }
}
