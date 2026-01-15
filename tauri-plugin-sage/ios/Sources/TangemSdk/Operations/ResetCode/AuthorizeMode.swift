//
//  AuthorizeMode.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 03.09.2021.
//  Copyright © 2021 Tangem AG. All rights reserved.
//

import Foundation

enum AuthorizeMode: Byte, InteractionMode {
    case fileOwnerGetChallenge = 0x01
    case fileOwnerAuthenticate = 0x02
    case tokenGet = 0x03
    case tokenSign = 0x04
    case tokenAuthenticate = 0x05
}
