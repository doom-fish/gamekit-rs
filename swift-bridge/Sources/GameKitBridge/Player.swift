import Foundation
import GameKit

struct GKPlayerPayload: Codable {
    let gamePlayerID: String
    let teamPlayerID: String
    let alias: String
    let displayName: String
    let playerID: String?
    let guestIdentifier: String?
    let isInvitable: Bool
    let scopedIDsArePersistent: Bool
}

func gkPlayerPayload(from player: GKPlayer) -> GKPlayerPayload {
    GKPlayerPayload(
        gamePlayerID: player.gamePlayerID,
        teamPlayerID: player.teamPlayerID,
        alias: player.alias,
        displayName: player.displayName,
        playerID: nil,
        guestIdentifier: player.guestIdentifier,
        isInvitable: player.isInvitable,
        scopedIDsArePersistent: player.scopedIDsArePersistent()
    )
}

@_cdecl("gk_player_anonymous_guest_json")
public func gk_player_anonymous_guest_json(
    _ identifier: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        guard let identifier else {
            throw GKBridgeError.unknown("missing guest identifier")
        }
        let player = GKPlayer.anonymousGuestPlayer(withIdentifier: String(cString: identifier))
        let payload = gkPlayerPayload(from: player)
        outJSON?.pointee = gkCString(try gkEncodeJSON(payload))
        return GK_OK
    } catch {
        gkPopulateError(outError, with: error)
        return gkStatusFor(error)
    }
}
