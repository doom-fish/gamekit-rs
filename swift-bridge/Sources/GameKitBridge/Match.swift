import Foundation
import GameKit

public typealias GKMatchDataCallbackFn = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafePointer<CChar>?,
    UnsafePointer<UInt8>?,
    Int
) -> Void

public typealias GKMatchStateCallbackFn = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafePointer<CChar>?,
    Int32
) -> Void

public typealias GKMatchFailureCallbackFn = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafePointer<CChar>?
) -> Void

final class GKMatchDelegateImpl: NSObject, GKMatchDelegate {
    var dataCallback: GKMatchDataCallbackFn?
    var stateCallback: GKMatchStateCallbackFn?
    var failureCallback: GKMatchFailureCallbackFn?
    var refcon: UnsafeMutableRawPointer?

    func match(_ match: GKMatch, didReceive data: Data, fromRemotePlayer player: GKPlayer) {
        guard let callback = dataCallback,
              let playerJSON = try? gkEncodeJSON(gkPlayerPayload(from: player)),
              let playerCString = gkCString(playerJSON)
        else {
            return
        }

        defer { free(playerCString) }
        data.withUnsafeBytes { bytes in
            callback(refcon, UnsafePointer(playerCString), bytes.bindMemory(to: UInt8.self).baseAddress, data.count)
        }
    }

    func match(_ match: GKMatch, player: GKPlayer, didChange state: GKPlayerConnectionState) {
        guard let callback = stateCallback,
              let playerJSON = try? gkEncodeJSON(gkPlayerPayload(from: player)),
              let playerCString = gkCString(playerJSON)
        else {
            return
        }

        defer { free(playerCString) }
        let stateValue: Int32
        switch state {
        case .connected:
            stateValue = 1
        case .disconnected:
            stateValue = 2
        default:
            stateValue = 0
        }
        callback(refcon, UnsafePointer(playerCString), stateValue)
    }

    func match(_ match: GKMatch, didFailWithError error: Error?) {
        guard let callback = failureCallback else {
            return
        }

        guard let error else {
            callback(refcon, nil)
            return
        }

        let nsError = error as NSError
        let payload = GKFrameworkErrorPayload(
            domain: nsError.domain,
            code: nsError.code,
            localizedDescription: nsError.localizedDescription
        )
        guard let errorJSON = try? gkEncodeJSON(payload), let errorCString = gkCString(errorJSON) else {
            callback(refcon, nil)
            return
        }

        defer { free(errorCString) }
        callback(refcon, UnsafePointer(errorCString))
    }
}

final class GKMatchBox {
    let match: GKMatch
    let delegate: GKMatchDelegateImpl

    init(match: GKMatch) {
        self.match = match
        let delegate = GKMatchDelegateImpl()
        self.delegate = delegate
        match.delegate = delegate
    }

    deinit {
        match.delegate = nil
    }
}

@_cdecl("gk_match_retain")
public func gk_match_retain(_ ptr: UnsafeMutableRawPointer?) -> UnsafeMutableRawPointer? {
    guard let ptr else { return nil }
    return gk_retain(gk_borrow(ptr, as: GKMatchBox.self))
}

@_cdecl("gk_match_release")
public func gk_match_release(_ ptr: UnsafeMutableRawPointer?) {
    guard let ptr else { return }
    gk_release(ptr)
}

@_cdecl("gk_match_players_json")
public func gk_match_players_json(
    _ ptr: UnsafeMutableRawPointer?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null match pointer"))
        return GK_UNKNOWN
    }

    do {
        let box = gk_borrow(ptr, as: GKMatchBox.self)
        outJSON?.pointee = gkCString(try gkEncodeJSON(box.match.players.map(gkPlayerPayload(from:))))
        return GK_OK
    } catch {
        gkPopulateError(outError, with: error)
        return gkStatusFor(error)
    }
}

@_cdecl("gk_match_connected_players_json")
public func gk_match_connected_players_json(
    _ ptr: UnsafeMutableRawPointer?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gk_match_players_json(ptr, outJSON, outError)
}

@_cdecl("gk_match_expected_player_count")
public func gk_match_expected_player_count(_ ptr: UnsafeMutableRawPointer?) -> Int {
    guard let ptr else { return 0 }
    let box = gk_borrow(ptr, as: GKMatchBox.self)
    return box.match.expectedPlayerCount
}

@_cdecl("gk_match_send_data")
public func gk_match_send_data(
    _ ptr: UnsafeMutableRawPointer?,
    _ data: UnsafePointer<UInt8>?,
    _ len: Int,
    _ playerIDsJSON: UnsafePointer<CChar>?,
    _ mode: Int32,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null match pointer"))
        return GK_UNKNOWN
    }
    guard let data else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null data pointer"))
        return GK_UNKNOWN
    }

    do {
        let playerIDs = try gkDecodeJSON(playerIDsJSON, as: [String].self)
        let box = gk_borrow(ptr, as: GKMatchBox.self)
        let players = box.match.players.filter { playerIDs.contains($0.gamePlayerID) }
        let dataToSend = Data(bytes: data, count: len)
        try box.match.send(dataToSend, to: players, dataMode: mode == 0 ? .reliable : .unreliable)
        return GK_OK
    } catch {
        gkPopulateError(outError, with: error)
        return gkStatusFor(error)
    }
}

@_cdecl("gk_match_send_data_to_all")
public func gk_match_send_data_to_all(
    _ ptr: UnsafeMutableRawPointer?,
    _ data: UnsafePointer<UInt8>?,
    _ len: Int,
    _ mode: Int32,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null match pointer"))
        return GK_UNKNOWN
    }
    guard let data else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null data pointer"))
        return GK_UNKNOWN
    }

    do {
        let box = gk_borrow(ptr, as: GKMatchBox.self)
        let dataToSend = Data(bytes: data, count: len)
        try box.match.sendData(toAllPlayers: dataToSend, with: mode == 0 ? .reliable : .unreliable)
        return GK_OK
    } catch {
        gkPopulateError(outError, with: error)
        return gkStatusFor(error)
    }
}

@_cdecl("gk_match_set_callbacks")
public func gk_match_set_callbacks(
    _ ptr: UnsafeMutableRawPointer?,
    _ dataCb: GKMatchDataCallbackFn?,
    _ stateCb: GKMatchStateCallbackFn?,
    _ failureCb: GKMatchFailureCallbackFn?,
    _ refcon: UnsafeMutableRawPointer?
) {
    guard let ptr else { return }
    let box = gk_borrow(ptr, as: GKMatchBox.self)
    box.delegate.dataCallback = dataCb
    box.delegate.stateCallback = stateCb
    box.delegate.failureCallback = failureCb
    box.delegate.refcon = refcon
}

@_cdecl("gk_match_clear_callbacks")
public func gk_match_clear_callbacks(_ ptr: UnsafeMutableRawPointer?) {
    guard let ptr else { return }
    let box = gk_borrow(ptr, as: GKMatchBox.self)
    box.delegate.dataCallback = nil
    box.delegate.stateCallback = nil
    box.delegate.failureCallback = nil
    box.delegate.refcon = nil
}

@_cdecl("gk_match_disconnect")
public func gk_match_disconnect(_ ptr: UnsafeMutableRawPointer?) {
    guard let ptr else { return }
    let box = gk_borrow(ptr, as: GKMatchBox.self)
    box.match.disconnect()
}

@_cdecl("gk_match_choose_best_hosting_player_json")
public func gk_match_choose_best_hosting_player_json(
    _ ptr: UnsafeMutableRawPointer?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null match pointer"))
        return GK_UNKNOWN
    }

    return gkBlockOnAsync(
        work: {
            let box = gk_borrow(ptr, as: GKMatchBox.self)
            return await withCheckedContinuation { continuation in
                box.match.chooseBestHostingPlayer { player in
                    continuation.resume(returning: player)
                }
            }
        },
        onSuccess: { (player: GKPlayer?) in
            let payload = player.map(gkPlayerPayload(from:))
            outJSON?.pointee = gkCString((try? gkEncodeJSON(payload)) ?? "null")
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_match_rematch")
public func gk_match_rematch(
    _ ptr: UnsafeMutableRawPointer?,
    _ outMatchPtr: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null match pointer"))
        return GK_UNKNOWN
    }

    return gkBlockOnAsync(
        work: {
            let box = gk_borrow(ptr, as: GKMatchBox.self)
            return try await withCheckedThrowingContinuation { continuation in
                box.match.rematch { match, error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else if let match {
                        continuation.resume(returning: match)
                    } else {
                        continuation.resume(throwing: GKBridgeError.unknown("rematch returned nil"))
                    }
                }
            }
        },
        onSuccess: { match in
            outMatchPtr?.pointee = gk_retain(GKMatchBox(match: match))
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}
