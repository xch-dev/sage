//
//  OnlineAttestationResponse.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 01.12.2021.
//  Copyright © 2021 Tangem AG. All rights reserved.
//

import Foundation

public struct OnlineAttestationResponse: Codable {
    public let manufacturerSignature: Data?
    public let issuerSignature: Data
}
