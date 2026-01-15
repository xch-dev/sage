//
//  CardFilter.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 07.07.2021.
//  Copyright © 2021 Tangem AG. All rights reserved.
//

import Foundation

public struct CardFilter {
    /// Filter that can be used to limit cards that can be interacted with in TangemSdk.
    public var allowedCardTypes: [FirmwareVersion.FirmwareType] = [.release, .sdk]
    
    /// Use this filter to configure cards allowed to work with your app
    public var cardIdFilter: CardIdFilter? = nil
    
    /// Use this filter to configure batches allowed to work with your app
    public var batchIdFilter: ItemFilter? = nil
    
    /// Use this filter to configure issuers allowed to work with your app
    public var issuerFilter: ItemFilter? = nil

    /// Use this filter to configure the highest firmware version allowed to work with your app. Nil to allow all versions.
    public var maxFirmwareVersion: FirmwareVersion? = nil
    
    /// Custom error localized description
    public var localizedDescription: String? = nil
    
    public static var `default`: CardFilter = .init()
    
    private var wrongCardError: TangemSdkError {
        .wrongCardType(localizedDescription)
    }
    
    public func verifyCard(_ card: Card) throws {
        if let maxFirmwareVersion = maxFirmwareVersion,
           card.firmwareVersion > maxFirmwareVersion {
            throw wrongCardError
        }

        if !allowedCardTypes.contains(card.firmwareVersion.type) {
            throw wrongCardError
        }
        
        if let batchIdFilter = batchIdFilter,
           !batchIdFilter.isAllowed(card.batchId) {
            throw wrongCardError
        }
        
        if let issuerFilter = issuerFilter,
           !issuerFilter.isAllowed(card.issuer.name) {
            throw wrongCardError
        }
        
        if let cardIdFilter = cardIdFilter,
           !cardIdFilter.isAllowed(card.cardId) {
            throw wrongCardError
        }
    }
}

public struct CardIdRange {
    public let start: UInt64
    public let end: UInt64

    public init?(start: String, end: String) {
        guard let startCardID = UInt64(start, radix: 16),
              let endCardID = UInt64(end, radix: 16),
              endCardID > startCardID else {
            return nil
        }
        
        self.start = startCardID
        self.end = endCardID
    }
    
    public func contains(_ cardId: String) -> Bool {
        guard let value = UInt64(cardId, radix: 16) else {
            return false
        }

        let range = start...end
        return range.contains(value)
    }
}

public extension CardFilter {
    enum ItemFilter {
        case allow(_ items: Set<String>)
        case deny(_ items: Set<String>)
        
        func isAllowed(_ item: String) -> Bool {
            switch self {
            case .allow(let items):
                return items.contains(item)
            case .deny(let items):
                return !items.contains(item)
            }
        }
    }
    
    enum CardIdFilter {
        case allow(_ items: Set<String>, ranges: [CardIdRange] = [])
        case deny(_ items: Set<String>, ranges: [CardIdRange] = [])
        
        func isAllowed(_ item: String) -> Bool {
            switch self {
            case .allow(let items, let ranges):
                return items.contains(item) || ranges.contains(item)
            case .deny(let items, let ranges):
                return !(items.contains(item) || ranges.contains(item))
            }
        }
    }
}

extension Array where Element == CardIdRange {
    public func contains(_ cardId: String) -> Bool {
        for range in self {
            if range.contains(cardId) {
                return true
            }
        }
        
        return false
    }
}
