//
//  UIScreen+.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 18.08.2021.
//  Copyright © 2021 Tangem AG. All rights reserved.
//

import Foundation
import UIKit

extension UIScreen {
    var isZoomedMode: Bool { UIScreen.main.scale != UIScreen.main.nativeScale }
}
