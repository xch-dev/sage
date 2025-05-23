//
//  UIColor+.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 09.06.2020.
//  Copyright © 2020 Tangem AG. All rights reserved.
//

import Foundation
import UIKit

extension UIColor {
    struct LightPalette {        
        static var indicatorBackground: UIColor {
            .init(red: 240.0/255.0,
                  green: 241.0/255.0,
                  blue: 242.0/255.0,
                  alpha: 0.9)
        }
        
        static var cardColor: UIColor {
            .init(red: 176.0/255.0,
                  green: 176.0/255.0,
                  blue: 176.0/255.0,
                  alpha: 1)
        }
        
        static var secondaryButtonBackground: UIColor {
            .init(red: 224.0/255.0,
                  green: 224/255.0,
                  blue: 224.0/255.0,
                  alpha: 1)
        }
        
        static var secondaryButtonForeground: UIColor {
            .init(red: 58.0/255.0,
                  green: 58/255.0,
                  blue: 60.0/255.0,
                  alpha: 1)
        }
    }
}

extension UIColor {
    enum DarkPalette {
        static var cardColor: UIColor {
            .init(red: 101.0/255.0,
                  green: 101.0/255.0,
                  blue: 101.0/255.0,
                  alpha: 1)
        }
    }
}
