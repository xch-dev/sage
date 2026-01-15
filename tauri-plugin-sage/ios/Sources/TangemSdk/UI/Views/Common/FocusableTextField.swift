//
//  FocusableTextField.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 11.10.2022.
//  Copyright © 2022 Tangem AG. All rights reserved.
//

import Foundation
import SwiftUI
import Combine

struct FocusableTextField: View {
    let shouldBecomeFirstResponder: Bool
    let text: Binding<String>
    var onCommit: () -> Void = {}
    
    @ObservedObject private var model: FocusableTextFieldModel
    
    var body: some View {
        if #available(iOS 15.0, *) {
            ModernFocusableTextField(
                shouldBecomeFirstResponder: shouldBecomeFirstResponder,
                text: text,
                onCommit: onCommit,
                model: model
            )
        } else {
            LegacyFocusableTextField(
                shouldBecomeFirstResponder: shouldBecomeFirstResponder,
                text: text,
                onCommit: onCommit,
                model: model
            )
        }
    }
    
    init(shouldBecomeFirstResponder: Bool,
         text: Binding<String>,
         onCommit: @escaping () -> Void = {}
    ) {
        self.shouldBecomeFirstResponder = shouldBecomeFirstResponder
        self.text = text
        self.onCommit = onCommit
        self.model = FocusableTextFieldModel()
    }
}

// MARK: - iOS 15+ Implementation

@available(iOS 15.0, *)
private struct ModernFocusableTextField: View {
    let shouldBecomeFirstResponder: Bool
    let text: Binding<String>
    var onCommit: () -> Void
    @ObservedObject var model: FocusableTextFieldModel
    
    @FocusState private var focusedField: Field?
    
    var body: some View {
        SecureField("", text: text, onCommit: onCommit)
            .focused($focusedField, equals: .secure)
            .keyboardType(.default)
            .writingToolsBehaviorDisabled()
            .autocorrectionDisabled()
            .textInputAutocapitalization(.never)
            .onAppear(perform: model.onAppear)
            .onReceive(model.focusPublisher) { _ in
                if shouldBecomeFirstResponder {
                    focusedField = .secure
                }
            }
    }
    
    enum Field: Hashable {
        case secure
    }
}

// MARK: - iOS 13-14 Implementation

private struct LegacyFocusableTextField: View {
    let shouldBecomeFirstResponder: Bool
    let text: Binding<String>
    var onCommit: () -> Void
    @ObservedObject var model: FocusableTextFieldModel
    
    var body: some View {
        LegacySecureFieldWrapper(
            text: text,
            shouldBecomeFirstResponder: shouldBecomeFirstResponder,
            onCommit: onCommit,
            focusPublisher: model.focusPublisher
        )
        .keyboardType(.default)
        .disableAutocorrection(true)
        .autocapitalization(.none)
        .onAppear(perform: model.onAppear)
    }
}

// MARK: - UIKit Wrapper for iOS 13-14

private struct LegacySecureFieldWrapper: UIViewRepresentable {
    @Binding var text: String
    let shouldBecomeFirstResponder: Bool
    let onCommit: () -> Void
    let focusPublisher: PassthroughSubject<Void, Never>
    
    func makeUIView(context: Context) -> UITextField {
        let textField = UITextField()
        textField.isSecureTextEntry = true
        textField.autocorrectionType = .no
        textField.autocapitalizationType = .none
        textField.delegate = context.coordinator
        textField.addTarget(context.coordinator, action: #selector(Coordinator.textFieldDidChange(_:)), for: .editingChanged)
        return textField
    }
    
    func updateUIView(_ uiView: UITextField, context: Context) {
        uiView.text = text
    }
    
    func makeCoordinator() -> Coordinator {
        Coordinator(text: $text, onCommit: onCommit, shouldBecomeFirstResponder: shouldBecomeFirstResponder, focusPublisher: focusPublisher)
    }
    
    class Coordinator: NSObject, UITextFieldDelegate {
        @Binding var text: String
        let onCommit: () -> Void
        let shouldBecomeFirstResponder: Bool
        var cancellable: AnyCancellable?
        
        init(text: Binding<String>, onCommit: @escaping () -> Void, shouldBecomeFirstResponder: Bool, focusPublisher: PassthroughSubject<Void, Never>) {
            self._text = text
            self.onCommit = onCommit
            self.shouldBecomeFirstResponder = shouldBecomeFirstResponder
            super.init()
            
            cancellable = focusPublisher.sink { [weak self] _ in
                guard let self = self, self.shouldBecomeFirstResponder else { return }
                // The textField reference will be set when makeUIView is called
                DispatchQueue.main.async {
                    self.textField?.becomeFirstResponder()
                }
            }
        }
        
        weak var textField: UITextField?
        
        @objc func textFieldDidChange(_ textField: UITextField) {
            self.textField = textField
            text = textField.text ?? ""
        }
        
        func textFieldShouldReturn(_ textField: UITextField) -> Bool {
            self.textField = textField
            onCommit()
            return true
        }
    }
}

// MARK: - Shared Model

fileprivate class FocusableTextFieldModel: ObservableObject {
    var focusPublisher: PassthroughSubject<Void, Never> = .init()
    
    private var appearPublisher: CurrentValueSubject<Bool, Never> = .init(false)
    private var activePublisher: CurrentValueSubject<Bool, Never> = .init(UIApplication.shared.isActive)
    private var bag: Set<AnyCancellable> = .init()
    
    private var becomeActivePublisher: AnyPublisher<Void, Never> {
        NotificationCenter.default
            .publisher(for: UIApplication.didBecomeActiveNotification)
            .map { _ in () }
            .eraseToAnyPublisher()
    }
    
    /// This is the minimum allowable delay, calculated empirically for all iOS versions prior 16.
    private var appearDelay: Int {
        if #available(iOS 16.0, *) {
            return 0
        } else {
            return 500
        }
    }
    
    init() {
        bind()
    }
    
    func onAppear() {
        appearPublisher.send(true)
    }
    
    private func bind() {
        becomeActivePublisher
            .sink { [weak self] _ in
                self?.activePublisher.send(true)
            }
            .store(in: &bag)
        
        appearPublisher
            .filter { $0 }
            .delay(for: .milliseconds(appearDelay), scheduler: DispatchQueue.main)
            .combineLatest(activePublisher.filter{ $0 })
            .sink { [weak self] _ in
                self?.focusPublisher.send(())
            }
            .store(in: &bag)
    }
}

fileprivate extension UIApplication {
    var isActive: Bool {
        applicationState == .active
    }
}

fileprivate extension View {
    @ViewBuilder
    func writingToolsBehaviorDisabled() -> some View {
        if #available(iOS 18.0, *) {
            self.writingToolsBehavior(.disabled)
        } else {
            self
        }
    }
}
