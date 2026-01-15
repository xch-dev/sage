//
//  Date+.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 14.11.2019.
//  Copyright © 2019 Tangem AG. All rights reserved.
//

import Foundation

extension Date {
    public func toString(style: DateFormatter.Style = .medium, locale: Locale = .current) -> String {
        let formatter = DateFormatter()
        formatter.dateStyle = style
        formatter.locale = locale
        return formatter.string(from: self)
    }
}
