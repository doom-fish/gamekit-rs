// swiftlint:disable identifier_name
import Foundation
import GameKit

let GK_OK: Int32 = 0
let GK_TIMED_OUT: Int32 = -2
let GK_NOT_AUTHENTICATED: Int32 = -3
let GK_FRAMEWORK_ERROR: Int32 = -4
let GK_NOT_FOUND: Int32 = -5
let GK_UNAVAILABLE: Int32 = -6
let GK_UNKNOWN: Int32 = -99

private let gkDateFormatter: ISO8601DateFormatter = {
    let formatter = ISO8601DateFormatter()
    formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
    return formatter
}()

@inline(__always)
func gkCString(_ string: String) -> UnsafeMutablePointer<CChar>? {
    string.withCString { strdup($0) }
}

@_cdecl("gk_string_free")
public func gk_string_free(_ ptr: UnsafeMutablePointer<CChar>?) {
    free(ptr)
}

@inline(__always)
func gk_retain<T: AnyObject>(_ object: T) -> UnsafeMutableRawPointer {
    Unmanaged.passRetained(object).toOpaque()
}

@inline(__always)
func gk_borrow<T: AnyObject>(_ ptr: UnsafeMutableRawPointer, as _: T.Type = T.self) -> T {
    Unmanaged<T>.fromOpaque(ptr).takeUnretainedValue()
}

@inline(__always)
func gk_release(_ ptr: UnsafeMutableRawPointer) {
    Unmanaged<AnyObject>.fromOpaque(ptr).release()
}

enum GKBridgeError: Error, CustomStringConvertible {
    case timedOut(String)
    case notAuthenticated(String)
    case notFound(String)
    case unavailable(String)
    case unknown(String)

    var statusCode: Int32 {
        switch self {
        case .timedOut:
            return GK_TIMED_OUT
        case .notAuthenticated:
            return GK_NOT_AUTHENTICATED
        case .notFound:
            return GK_NOT_FOUND
        case .unavailable:
            return GK_UNAVAILABLE
        case .unknown:
            return GK_UNKNOWN
        }
    }

    var description: String {
        switch self {
        case .timedOut(let message),
             .notAuthenticated(let message),
             .notFound(let message),
             .unavailable(let message),
             .unknown(let message):
            return message
        }
    }
}

struct GKFrameworkErrorPayload: Encodable {
    let kind = "framework"
    let domain: String
    let code: Int
    let localizedDescription: String
}

struct GKBinaryPayload: Encodable {
    let dataBase64: String
}

func gkDateString(_ date: Date?) -> String? {
    guard let date else {
        return nil
    }
    return gkDateFormatter.string(from: date)
}

func gkEncodeJSON<T: Encodable>(_ value: T) throws -> String {
    let encoder = JSONEncoder()
    let data = try encoder.encode(value)
    guard let string = String(data: data, encoding: .utf8) else {
        throw GKBridgeError.unknown("failed to encode JSON as UTF-8")
    }
    return string
}

func gkDecodeJSON<T: Decodable>(_ cString: UnsafePointer<CChar>?, as type: T.Type) throws -> T {
    guard let cString else {
        throw GKBridgeError.unknown("missing JSON payload")
    }
    let data = Data(String(cString: cString).utf8)
    do {
        return try JSONDecoder().decode(T.self, from: data)
    } catch {
        throw GKBridgeError.unknown("invalid JSON payload: \(error.localizedDescription)")
    }
}

func gkBinaryPayloadJSON(_ data: Data) throws -> String {
    try gkEncodeJSON(GKBinaryPayload(dataBase64: data.base64EncodedString()))
}

func gkStatusFor(_ error: Error) -> Int32 {
    if let bridgeError = error as? GKBridgeError {
        return bridgeError.statusCode
    }
    return GK_FRAMEWORK_ERROR
}

func gkPopulateError(
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    with error: Error
) {
    let message: String
    if let bridgeError = error as? GKBridgeError {
        message = bridgeError.description
    } else {
        let nsError = error as NSError
        let payload = GKFrameworkErrorPayload(
            domain: nsError.domain,
            code: nsError.code,
            localizedDescription: nsError.localizedDescription
        )
        message = (try? gkEncodeJSON(payload)) ?? nsError.localizedDescription
    }
    outError?.pointee = gkCString(message)
}

func gkBlockOnAsync<T>(
    timeoutSeconds: Int = 30,
    work: @escaping () async throws -> T,
    onSuccess: @escaping (T) -> Void,
    onError: @escaping (Error) -> Void
) -> Int32 {
    let semaphore = DispatchSemaphore(value: 0)
    var result: Result<T, Error>?

    Task {
        do {
            result = .success(try await work())
        } catch {
            result = .failure(error)
        }
        semaphore.signal()
    }

    guard semaphore.wait(timeout: .now() + .seconds(timeoutSeconds)) == .success else {
        onError(GKBridgeError.timedOut("GameKit operation timed out after \(timeoutSeconds) seconds"))
        return GK_TIMED_OUT
    }

    switch result {
    case .success(let value):
        onSuccess(value)
        return GK_OK
    case .failure(let error):
        onError(error)
        return gkStatusFor(error)
    case .none:
        let error = GKBridgeError.unknown("GameKit operation completed without a result")
        onError(error)
        return error.statusCode
    }
}

func gkLoadPlayers(identifiedBy ids: [String]) async throws -> [GKPlayer] {
    if ids.isEmpty {
        return []
    }

    return try await withCheckedThrowingContinuation { continuation in
        GKLocalPlayer.local.loadFriends(identifiedBy: ids) { players, error in
            if let error {
                continuation.resume(throwing: error)
            } else {
                continuation.resume(returning: players ?? [])
            }
        }
    }
}

func gkResolvePlayers(byGameIDs ids: [String], preferred: [GKPlayer] = []) async throws -> [String: GKPlayer] {
    var resolved = Dictionary(uniqueKeysWithValues: preferred.map { ($0.gamePlayerID, $0) })
    let unresolved = ids.filter { resolved[$0] == nil }
    if !unresolved.isEmpty {
        let loaded = try await gkLoadPlayers(identifiedBy: unresolved)
        for player in loaded {
            resolved[player.gamePlayerID] = player
        }
    }
    return resolved
}
