// swiftlint:disable identifier_name
import Foundation
import GameKit

let GK_OK: Int32 = 0
let GK_TIMED_OUT: Int32 = -2
let GK_NOT_AUTHENTICATED: Int32 = -3
let GK_FRAMEWORK_ERROR: Int32 = -4
let GK_NOT_FOUND: Int32 = -5
let GK_UNAVAILABLE: Int32 = -6
let GK_WRONG_THREAD: Int32 = -7
let GK_UNKNOWN: Int32 = -99

private let gkDateFormatter: ISO8601DateFormatter = {
    let formatter = ISO8601DateFormatter()
    formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
    return formatter
}()

public typealias GKContextCallback = @convention(c) (UnsafeMutableRawPointer?) -> Void

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
    case wrongThread(String)
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
        case .wrongThread:
            return GK_WRONG_THREAD
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
             .wrongThread(let message),
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
    try gkDecodeJSON(gkCopyString(cString), as: type)
}

func gkDecodeJSON<T: Decodable>(_ string: String?, as type: T.Type) throws -> T {
    guard let string else {
        throw GKBridgeError.unknown("missing JSON payload")
    }
    do {
        return try JSONDecoder().decode(T.self, from: Data(string.utf8))
    } catch {
        throw GKBridgeError.unknown("invalid JSON payload: \(error.localizedDescription)")
    }
}

func gkCopyString(_ cString: UnsafePointer<CChar>?) -> String? {
    cString.map { String(cString: $0) }
}

func gkCopyData(_ bytes: UnsafePointer<UInt8>?, _ count: Int) -> Data {
    guard let bytes, count > 0 else {
        return Data()
    }
    return Data(bytes: bytes, count: count)
}

func gkRequireBundleIdentifier(_ operation: String) throws {
    guard Bundle.main.bundleIdentifier != nil else {
        throw GKBridgeError.unavailable(
            "\(operation) requires the process to have a bundle identifier; GameKit raises an exception without one"
        )
    }
}

func gkRequireMainThread(_ operation: String) throws {
    guard Thread.isMainThread else {
        throw GKBridgeError.wrongThread("\(operation) must be called on the main thread")
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

private final class GKAsyncResultBox<T> {
    private let lock = NSLock()
    private var result: Result<T, Error>?

    func store(_ value: Result<T, Error>) {
        lock.lock()
        result = value
        lock.unlock()
    }

    func load() -> Result<T, Error>? {
        lock.lock()
        defer { lock.unlock() }
        return result
    }
}

func gkBlockOnAsync<T>(
    timeoutSeconds: Int = 30,
    work: @escaping () async throws -> T,
    onSuccess: (T) -> Void,
    onError: (Error) -> Void
) -> Int32 {
    guard !Thread.isMainThread else {
        let error = GKBridgeError.wrongThread(
            "synchronous GameKit calls cannot run on the main thread, which delivers their results; call them from another thread or use the async API"
        )
        onError(error)
        return error.statusCode
    }

    let semaphore = DispatchSemaphore(value: 0)
    let box = GKAsyncResultBox<T>()

    let task = Task {
        do {
            box.store(.success(try await work()))
        } catch {
            box.store(.failure(error))
        }
        semaphore.signal()
    }

    guard semaphore.wait(timeout: .now() + .seconds(timeoutSeconds)) == .success else {
        task.cancel()
        onError(GKBridgeError.timedOut("GameKit operation timed out after \(timeoutSeconds) seconds"))
        return GK_TIMED_OUT
    }

    switch box.load() {
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

    try gkRequireBundleIdentifier("loading players by identifier")
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
