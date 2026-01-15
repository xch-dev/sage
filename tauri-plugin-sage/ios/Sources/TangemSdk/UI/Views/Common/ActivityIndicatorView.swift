//
//  ActivityIndicatorView.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 13.08.2021.
//  Copyright © 2021 Tangem AG. All rights reserved.
//

import Foundation
import SwiftUI

struct ActivityIndicatorView: UIViewRepresentable {
    private var isAnimating: Bool
    private var style: UIActivityIndicatorView.Style
    private var color: UIColor

    init(isAnimating: Bool = true, style: UIActivityIndicatorView.Style = .medium, color: Color) {
        self.isAnimating = isAnimating
        self.style = style
        let components = getComponents(color)
        self.color = UIColor(red: components.r, green: components.g, blue: components.b, alpha: components.a)
    }
    
    func makeUIView(context: UIViewRepresentableContext<ActivityIndicatorView>) -> UIActivityIndicatorView {
        let indicator = UIActivityIndicatorView(style: style)
        indicator.color = color
        return indicator
    }

    func updateUIView(_ uiView: UIActivityIndicatorView, context: UIViewRepresentableContext<ActivityIndicatorView>) {
        isAnimating ? uiView.startAnimating() : uiView.stopAnimating()
    }
}

private func getComponents(_ color: Color) -> (r: CGFloat, g: CGFloat, b: CGFloat, a: CGFloat) {
    let scanner = Scanner(string: color.description.trimmingCharacters(in: CharacterSet.alphanumerics.inverted))
    var hexNumber: UInt64 = 0
    var r: CGFloat = 0.0, g: CGFloat = 0.0, b: CGFloat = 0.0, a: CGFloat = 0.0
    
    let result = scanner.scanHexInt64(&hexNumber)
    if result {
        r = CGFloat((hexNumber & 0xff000000) >> 24) / 255
        g = CGFloat((hexNumber & 0x00ff0000) >> 16) / 255
        b = CGFloat((hexNumber & 0x0000ff00) >> 8) / 255
        a = CGFloat(hexNumber & 0x000000ff) / 255
    }
    return (r, g, b, a)
}
