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
        guard let callback = dataCallback else { return }
        
        let playerPayload = gkPlayerPayload(from: player)
        if let playerJSON = try? gkEncodeJSON(playerPayload), let playerCString = gkCString(playerJSON) {
            data.withUnsafeBytes { (bytes: UnsafeRawBufferPointer) in
                if let baseAddress = bytes.baseAddress {
                    callback(refcon, playerCString, baseAddress.assumingMemoryBound(to: UInt8.self), data.count)
                }
            }
        }
    }
    
    func match(_ match: GKMatch, player: GKPlayer, didChange state: GKPlayerConnectionState) {
        guard let callback = stateCallback else { return }
        
        let stateValue: Int32
        switch state {
        case .connected:
            stateValue = 1
        case .disconnected:
            stateValue = 2
        default:
            stateValue = 0
        }
        
        let playerPayload = gkPlayerPayload(from: player)
        if let playerJSON = try? gkEncodeJSON(playerPayload), let playerCString = gkCString(playerJSON) {
            callback(refcon, playerCString, stateValue)
        }
    }
    
    func match(_ match: GKMatch, didFailWithError error: Error?) {
        guard let callback = failureCallback else { return }
        
        if let error = error {
            let nsError = error as NSError
            let payload = GKFrameworkErrorPayload(
                domain: nsError.domain,
                code: nsError.code,
                localizedDescription: nsError.localizedDescription
            )
            if let errorJSON = try? gkEncodeJSON(payload), let errorCString = gkCString(errorJSON) {
                callback(refcon, errorCString)
            } else {
                callback(refcon, nil)
            }
        } else {
            callback(refcon, nil)
        }
    }
}

final class GKMatchBox {
    let match: GKMatch
    let delegate: GKMatchDelegateImpl
    
    init(match: GKMatch) {
        self.match = match
        let d = GKMatchDelegateImpl()
        self.delegate = d
        match.delegate = d
    }
    
    deinit {
        match.delegate = nil
    }
}

@_cdecl("gk_match_retain")
public func gk_match_retain(_ ptr: UnsafeMutableRawPointer?) -> UnsafeMutableRawPointer? {
    guard let ptr = ptr else { return nil }
    return gk_retain(gk_borrow(ptr, as: GKMatchBox.self))
}

@_cdecl("gk_match_release")
public func gk_match_release(_ ptr: UnsafeMutableRawPointer?) {
    guard let ptr = ptr else { return }
    gk_release(ptr)
}

@_cdecl("gk_match_connected_players_json")
public func gk_match_connected_players_json(
    _ ptr: UnsafeMutableRawPointer?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr = ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null match pointer"))
        return GK_UNKNOWN
    }
    
    do {
        let box = gk_borrow(ptr, as: GKMatchBox.self)
        let payloads = box.match.players.map(gkPlayerPayload(from:))
        let json = try gkEncodeJSON(payloads)
        outJSON?.pointee = gkCString(json)
        return GK_OK
    } catch {
        gkPopulateError(outError, with: error)
        return gkStatusFor(error)
    }
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
    guard let ptr = ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null match pointer"))
        return GK_UNKNOWN
    }
    
    guard let data = data else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null data pointer"))
        return GK_UNKNOWN
    }
    
    do {
        let box = gk_borrow(ptr, as: GKMatchBox.self)
        let playerIDs = try gkDecodeJSON(playerIDsJSON, as: [String].self)
        
        let targetPlayers = box.match.players.filter { playerIDs.contains($0.gamePlayerID) }
        
        let dataToSend = Data(bytes: data, count: len)
        let dataMode: GKMatch.SendDataMode = mode == 0 ? .reliable : .unreliable
        
        try box.match.send(dataToSend, to: targetPlayers, dataMode: dataMode)
        return GK_OK
    } catch {
        gkPopulateError(outError, with: error)
        return GK_FRAMEWORK_ERROR
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
    guard let ptr = ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null match pointer"))
        return GK_UNKNOWN
    }
    
    guard let data = data else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null data pointer"))
        return GK_UNKNOWN
    }
    
    do {
        let box = gk_borrow(ptr, as: GKMatchBox.self)
        let dataToSend = Data(bytes: data, count: len)
        let dataMode: GKMatch.SendDataMode = mode == 0 ? .reliable : .unreliable
        
        try box.match.sendData(toAllPlayers: dataToSend, with: dataMode)
        return GK_OK
    } catch {
        gkPopulateError(outError, with: error)
        return GK_FRAMEWORK_ERROR
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
    guard let ptr = ptr else { return }
    
    let box = gk_borrow(ptr, as: GKMatchBox.self)
    box.delegate.dataCallback = dataCb
    box.delegate.stateCallback = stateCb
    box.delegate.failureCallback = failureCb
    box.delegate.refcon = refcon
}

@_cdecl("gk_match_clear_callbacks")
public func gk_match_clear_callbacks(_ ptr: UnsafeMutableRawPointer?) {
    guard let ptr = ptr else { return }
    
    let box = gk_borrow(ptr, as: GKMatchBox.self)
    box.delegate.dataCallback = nil
    box.delegate.stateCallback = nil
    box.delegate.failureCallback = nil
    box.delegate.refcon = nil
}

@_cdecl("gk_match_disconnect")
public func gk_match_disconnect(_ ptr: UnsafeMutableRawPointer?) {
    guard let ptr = ptr else { return }
    
    let box = gk_borrow(ptr, as: GKMatchBox.self)
    box.match.disconnect()
}

struct GKMatchRequestPayload: Codable {
    let minPlayers: Int
    let maxPlayers: Int
    let playerGroup: Int
    let playerAttributes: Int
}

@_cdecl("gk_matchmaker_find_match_json")
public func gk_matchmaker_find_match_json(
    _ requestJSON: UnsafePointer<CChar>?,
    _ outMatchPtr: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    return gkBlockOnAsync(
        work: {
            let requestPayload = try gkDecodeJSON(requestJSON, as: GKMatchRequestPayload.self)
            let request = GKMatchRequest()
            request.minPlayers = requestPayload.minPlayers
            request.maxPlayers = requestPayload.maxPlayers
            request.playerGroup = requestPayload.playerGroup
            request.playerAttributes = UInt32(requestPayload.playerAttributes)
            
            return try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<GKMatch, Error>) in
                GKMatchmaker.shared().findMatch(for: request) { match, error in
                    if let error = error {
                        continuation.resume(throwing: error)
                    } else if let match = match {
                        continuation.resume(returning: match)
                    } else {
                        continuation.resume(throwing: GKBridgeError.unknown("findMatch returned nil"))
                    }
                }
            }
        },
        onSuccess: { (match: GKMatch) in
            let box = GKMatchBox(match: match)
            outMatchPtr?.pointee = gk_retain(box)
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_matchmaker_cancel")
public func gk_matchmaker_cancel() {
    GKMatchmaker.shared().cancel()
}
