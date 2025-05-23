//
//  LAContext+.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 22.08.2022.
//  Copyright © 2022 Tangem AG. All rights reserved.
//

import LocalAuthentication

public extension LAContext {
    static var `default`: LAContext {
        let context = LAContext()
        context.localizedFallbackTitle = "" // hiding the "Enter Password" fallback button
        return context
    }
}
