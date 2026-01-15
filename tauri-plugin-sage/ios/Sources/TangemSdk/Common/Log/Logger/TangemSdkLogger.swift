//
//  TangemSdkLogger.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 09.02.2021.
//  Copyright © 2021 Tangem AG. All rights reserved.
//

import Foundation

public protocol TangemSdkLogger {
    func log(_ message: String, level: Log.Level)
}
