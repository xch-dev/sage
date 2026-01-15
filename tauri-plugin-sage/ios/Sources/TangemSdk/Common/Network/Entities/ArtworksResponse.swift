//
//  Untitled.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 28/03/2025.
//  Copyright © 2025 Tangem AG. All rights reserved.
//

import Foundation

struct ArtworksResponse: Decodable {
    let imageSmallUrl: URL?
    let imageSmallSignature: Data?
    let imageLargeUrl: URL
    let imageLargeSignature: Data
}
