//
//  NetworkService.swift
//  TangemSdk
//
//  Created by Alexander Osokin on 07.08.2020.
//  Copyright © 2020 Tangem AG. All rights reserved.
//

import Foundation
import Combine

public class NetworkService {
    private let session: URLSession
    private let additionalHeaders: [String: String]

    public init(session: URLSession, additionalHeaders: [String:String]) {
        self.session = session
        self.additionalHeaders = additionalHeaders
    }

    deinit {
        Log.debug("NetworkService deinit")
    }

    private func prepareRequest(from endpoint: NetworkEndpoint) -> URLRequest? {
        guard var urlComponents = URLComponents(string: endpoint.baseUrl + endpoint.path) else {
            return nil
        }

        urlComponents.queryItems = endpoint.queryItems

        guard let url = urlComponents.url else {
            return nil
        }

        var urlRequest = URLRequest(url: url)
        urlRequest.httpMethod = endpoint.method
        urlRequest.httpBody = endpoint.body

        for header in endpoint.headers {
            urlRequest.addValue(header.value, forHTTPHeaderField: header.key)
        }

        for header in additionalHeaders {
            urlRequest.addValue(header.value, forHTTPHeaderField: header.key)
        }

        return urlRequest
    }

    private func map<T: Decodable>(_ data: Data, type: T.Type) -> T? {
        try? JSONDecoder().decode(T.self, from: data)
    }
}

// MARK: - async/await

extension NetworkService {
    public func request<T: Decodable>(_ endpoint: NetworkEndpoint) async throws -> T {
        guard let request = prepareRequest(from: endpoint) else {
            throw NetworkServiceError.failedToMakeRequest
        }

        let responseData = try await requestData(request: request)

        do {
            return try JSONDecoder.tangemSdkDecoder.decode(T.self, from: responseData)
        }
        catch {
            throw NetworkServiceError.mappingError(error)
        }
    }

    func requestData(request: URLRequest) async throws -> Data {
        do {
            Log.network(request.requestDescription)

            let (data, response) = try await session.data(for: request)
            return try NetworkService.mapResponseData(data: data, response: response)
        } catch {
            throw NetworkService.mapError(error)
        }
    }
}

// MARK: - Combine

extension NetworkService {
    public func requestPublisher<T: Decodable>(_ endpoint: NetworkEndpoint) -> AnyPublisher<T, NetworkServiceError> {
        guard let request = prepareRequest(from: endpoint) else {
            return Fail(error: NetworkServiceError.failedToMakeRequest).eraseToAnyPublisher()
        }

        return requestDataPublisher(request: request)
            .tryMap { data -> T in
                do {
                    return try JSONDecoder.tangemSdkDecoder.decode(T.self, from: data)
                }
                catch {
                    throw NetworkServiceError.mappingError(error)
                }
            }
            .mapError { NetworkService.mapError($0) }
            .eraseToAnyPublisher()
    }

    func requestDataPublisher(request: URLRequest) -> AnyPublisher<Data, NetworkServiceError> {
        Log.network(request.requestDescription)

        return session
            .dataTaskPublisher(for: request)
            .subscribe(on: DispatchQueue.global())
            .tryMap { try NetworkService.mapResponseData(data: $0, response: $1) }
            .mapError { NetworkService.mapError($0) }
            .eraseToAnyPublisher()
    }
}

// MARK: - Helpers

private extension NetworkService {
    static func mapError(_ error: Error) -> NetworkServiceError {
        return error as? NetworkServiceError ?? NetworkServiceError.urlSessionError(error)
    }

    static func mapResponseData(data: Data, response: URLResponse) throws -> Data {
        Log.network("response: \(response)")

        guard let response = response as? HTTPURLResponse else {
            let error = NetworkServiceError.emptyResponse
            Log.network(error.localizedDescription)
            throw error
        }

        guard (200 ..< 300) ~= response.statusCode else {
            let error = NetworkServiceError.statusCode(response.statusCode, String(data: data, encoding: .utf8))
            Log.network(error.localizedDescription)
            throw error
        }

        Log.network("status code: \(response.statusCode), response: \(String(data: data, encoding: .utf8) ?? "" )")
        return data
    }
}


// MARK: - Constants

fileprivate extension URLSessionConfiguration {
    static var defaultTangemSDKConfiguration: URLSessionConfiguration {
        let configuration = URLSessionConfiguration.default
        configuration.timeoutIntervalForRequest = 10
        configuration.timeoutIntervalForResource = 30
        return configuration
    }
}

// MARK: - NetworkEndpoint

public protocol NetworkEndpoint {
    var baseUrl: String {get}
    var path: String {get}
    var queryItems: [URLQueryItem]? {get}
    var method: String {get}
    var body: Data? {get}
    var headers: [String:String] {get}
}

// MARK: - NetworkServiceError

public enum NetworkServiceError: Error, LocalizedError {
    case emptyResponse
    case statusCode(Int, String?)
    case urlSessionError(Error)
    case emptyResponseData
    case mappingError(Error)
    case failedToMakeRequest

    public var errorDescription: String? {
        switch self {
        case .urlSessionError(let error):
            return error.localizedDescription
        default:
            return "\(self)"
        }
    }

}

fileprivate extension URLRequest {
    var requestDescription: String {
        return "request: \(url?.absoluteString ?? "nil"), headers: \(allHTTPHeaderFields?.description ?? "[]")"
    }
}
