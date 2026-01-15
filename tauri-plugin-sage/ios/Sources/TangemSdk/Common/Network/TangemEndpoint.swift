//
//  TangemEndpoint.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 07.08.2020.
//  Copyright © 2020 Tangem AG. All rights reserved.
//

import Foundation

public enum TangemEndpoint: NetworkEndpoint {
    case cardData(cardId: String, cardPublicKey: Data)
    case artworks(cardId: String, cardPublicKey: Data)

    public var baseUrl: String {
        return Config.useDevApi ? "https://api.tests-d.com/" : "https://api.tangem.org/"
    }
    
    public var path: String {
        switch self {
        case .cardData:
            return "card"
        case .artworks:
            return "card/artworks"
        }
    }
    
    public var method: String {
        switch self {
        case .cardData, .artworks:
            return "GET"
        }
    }
    
    public var queryItems: [URLQueryItem]? {
        return nil
    }
    
    public var body: Data? {
        return nil
    }
    
    public var headers: [String : String] {
        var headers = ["Content-Type" : "application/json"]

        switch self {
        case .cardData(let cardId, let cardPublicKey),
                .artworks(let cardId, let cardPublicKey):
            headers["card_id"] = cardId
            headers["card_public_key"] = cardPublicKey.hexString
        }
        
        return headers
    }
}

