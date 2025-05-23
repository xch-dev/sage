//
//  Storage.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 06.07.2020.
//  Copyright © 2020 Tangem AG. All rights reserved.


import Foundation

public class Storage {
    static let prefix = "tng_"

    enum Keys: String {
        case refreshedTrustedCardsRepo
        case hasClearedAccessCodeRepoOnFirstLaunch
    }

    func bool(forKey: Storage.Keys) -> Bool {
        UserDefaults.standard.bool(forKey: rawKey(forKey, nil))
    }

    func set(boolValue: Bool, forKey: Storage.Keys) {
        UserDefaults.standard.set(boolValue, forKey: rawKey(forKey, nil))
    }

    func string(forKey: Storage.Keys) -> String? {
        UserDefaults.standard.string(forKey: rawKey(forKey, nil))
    }

    func set(stringValue: String, forKey: Storage.Keys) {
        UserDefaults.standard.set(stringValue, forKey: rawKey(forKey, nil))
    }

    func object<T: Decodable>(forKey: Storage.Keys, postfix: String? = nil) -> T? {
        if let data =  UserDefaults.standard.data(forKey: rawKey(forKey, postfix)) {
            let decoder = JSONDecoder()
            return try? decoder.decode(T.self, from: data)
        }
        return nil
    }

    func set<T: Encodable>(object: T, forKey: Storage.Keys, postfix: String? = nil) {
        let encoder = JSONEncoder()
        let data = try? encoder.encode(object)
        UserDefaults.standard.set(data, forKey: rawKey(forKey, postfix))
    }

    private func rawKey(_ forKey: Storage.Keys, _ postfix: String?) -> String {
        let baseKey = "\(Storage.prefix)\(forKey.rawValue)"
        if let postfix = postfix {
            return baseKey + "_\(postfix)"
        } else {
            return baseKey
        }
    }
}
