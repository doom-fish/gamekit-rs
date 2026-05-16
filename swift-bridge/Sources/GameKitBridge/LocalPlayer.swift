import Foundation
import GameKit

struct GKLocalPlayerPayload: Codable {
    let isAuthenticated: Bool
    let isUnderage: Bool
    let isMultiplayerGamingRestricted: Bool
    let player: GKPlayerPayload
}

struct GKAuthEventPayload: Encodable {
    let hasViewController: Bool
    let error: GKFrameworkErrorPayload?
}

public typealias GKAuthCallbackFn = @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?) -> Void

@_cdecl("gk_local_player_json")
public func gk_local_player_json(
    out_json: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    out_error: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let player = GKLocalPlayer.local
        let payload = GKLocalPlayerPayload(
            isAuthenticated: player.isAuthenticated,
            isUnderage: player.isUnderage,
            isMultiplayerGamingRestricted: player.isMultiplayerGamingRestricted,
            player: gkPlayerPayload(from: player)
        )
        let json = try gkEncodeJSON(payload)
        out_json?.pointee = gkCString(json)
        return GK_OK
    } catch {
        gkPopulateError(out_error, with: error)
        return gkStatusFor(error)
    }
}

private var gkAuthCallback: GKAuthCallbackFn?
private var gkAuthRefcon: UnsafeMutableRawPointer?

@_cdecl("gk_authenticate_handler_set")
public func gk_authenticate_handler_set(
    callback: GKAuthCallbackFn?,
    refcon: UnsafeMutableRawPointer?
) {
    gkAuthCallback = callback
    gkAuthRefcon = refcon
    
    GKLocalPlayer.local.authenticateHandler = { viewController, error in
        guard let callback = gkAuthCallback else { return }
        
        let authError: GKFrameworkErrorPayload?
        if let error = error {
            let nsError = error as NSError
            authError = GKFrameworkErrorPayload(
                domain: nsError.domain,
                code: nsError.code,
                localizedDescription: nsError.localizedDescription
            )
        } else {
            authError = nil
        }
        
        let payload = GKAuthEventPayload(
            hasViewController: viewController != nil,
            error: authError
        )
        
        if let json = try? gkEncodeJSON(payload), let jsonCString = gkCString(json) {
            callback(gkAuthRefcon, jsonCString)
        }
    }
}

@_cdecl("gk_authenticate_handler_clear")
public func gk_authenticate_handler_clear() {
    GKLocalPlayer.local.authenticateHandler = nil
    gkAuthCallback = nil
    gkAuthRefcon = nil
}
