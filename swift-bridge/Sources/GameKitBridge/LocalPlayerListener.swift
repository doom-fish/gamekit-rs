import Foundation
import GameKit

public typealias GKLocalPlayerListenerCallbackFn = @convention(c) (
    UnsafeMutableRawPointer?,
    Int32,
    UnsafePointer<CChar>?,
    UnsafeMutableRawPointer?
) -> Int32

struct GKLocalPlayerEventPayload: Encodable {
    let kind: String
    let player: GKPlayerPayload
    let recipients: [GKPlayerPayload]?
    let playersToInvite: [GKPlayerPayload]?
    let matchSnapshot: GKTurnBasedMatchPayload?
    let didBecomeActive: Bool?
    let exchange: GKTurnBasedExchangePayload?
    let replies: [GKTurnBasedExchangeReplyPayload]?
    let completedExchange: GKTurnBasedExchangePayload?
    let savedGame: GKSavedGamePayload?
    let savedGames: [GKSavedGamePayload]?
    let activity: GKGameActivitySnapshotPayload?
}

final class GKLocalPlayerListenerImpl: NSObject, GKLocalPlayerListener {
    let callback: GKLocalPlayerListenerCallbackFn
    let refcon: UnsafeMutableRawPointer?

    init(callback: @escaping GKLocalPlayerListenerCallbackFn, refcon: UnsafeMutableRawPointer?) {
        self.callback = callback
        self.refcon = refcon
    }

    @discardableResult
    private func emit(
        _ payload: GKLocalPlayerEventPayload,
        rawPointer: UnsafeMutableRawPointer? = nil
    ) -> Int32 {
        guard let json = try? gkEncodeJSON(payload), let cString = gkCString(json) else {
            return 0
        }
        defer { free(cString) }
        return callback(refcon, 0, UnsafePointer(cString), rawPointer)
    }

    func player(_ player: GKPlayer, didAccept invite: GKInvite) {
        _ = emit(
            GKLocalPlayerEventPayload(
                kind: "acceptedInvite",
                player: gkPlayerPayload(from: player),
                recipients: nil,
                playersToInvite: nil,
                matchSnapshot: nil,
                didBecomeActive: nil,
                exchange: nil,
                replies: nil,
                completedExchange: nil,
                savedGame: nil,
                savedGames: nil,
                activity: nil
            ),
            rawPointer: gk_retain(invite)
        )
    }

    func player(_ player: GKPlayer, didRequestMatchWithRecipients recipientPlayers: [GKPlayer]) {
        _ = emit(
            GKLocalPlayerEventPayload(
                kind: "requestedMatchWithRecipients",
                player: gkPlayerPayload(from: player),
                recipients: recipientPlayers.map(gkPlayerPayload(from:)),
                playersToInvite: nil,
                matchSnapshot: nil,
                didBecomeActive: nil,
                exchange: nil,
                replies: nil,
                completedExchange: nil,
                savedGame: nil,
                savedGames: nil,
                activity: nil
            )
        )
    }

    func player(_ player: GKPlayer, didRequestMatchWithOtherPlayers playersToInvite: [GKPlayer]) {
        _ = emit(
            GKLocalPlayerEventPayload(
                kind: "requestedTurnBasedMatchWithOtherPlayers",
                player: gkPlayerPayload(from: player),
                recipients: nil,
                playersToInvite: playersToInvite.map(gkPlayerPayload(from:)),
                matchSnapshot: nil,
                didBecomeActive: nil,
                exchange: nil,
                replies: nil,
                completedExchange: nil,
                savedGame: nil,
                savedGames: nil,
                activity: nil
            )
        )
    }

    func player(
        _ player: GKPlayer,
        receivedTurnEventFor match: GKTurnBasedMatch,
        didBecomeActive: Bool
    ) {
        _ = emit(
            GKLocalPlayerEventPayload(
                kind: "receivedTurnEvent",
                player: gkPlayerPayload(from: player),
                recipients: nil,
                playersToInvite: nil,
                matchSnapshot: try? gkTurnBasedMatchPayload(from: match),
                didBecomeActive: didBecomeActive,
                exchange: nil,
                replies: nil,
                completedExchange: nil,
                savedGame: nil,
                savedGames: nil,
                activity: nil
            )
        )
    }

    func player(_ player: GKPlayer, matchEnded match: GKTurnBasedMatch) {
        _ = emit(
            GKLocalPlayerEventPayload(
                kind: "matchEnded",
                player: gkPlayerPayload(from: player),
                recipients: nil,
                playersToInvite: nil,
                matchSnapshot: try? gkTurnBasedMatchPayload(from: match),
                didBecomeActive: nil,
                exchange: nil,
                replies: nil,
                completedExchange: nil,
                savedGame: nil,
                savedGames: nil,
                activity: nil
            )
        )
    }

    func player(
        _ player: GKPlayer,
        receivedExchangeRequest exchange: GKTurnBasedExchange,
        for match: GKTurnBasedMatch
    ) {
        _ = emit(
            GKLocalPlayerEventPayload(
                kind: "receivedExchangeRequest",
                player: gkPlayerPayload(from: player),
                recipients: nil,
                playersToInvite: nil,
                matchSnapshot: try? gkTurnBasedMatchPayload(from: match),
                didBecomeActive: nil,
                exchange: gkTurnBasedExchangePayload(from: exchange, index: 0, participants: match.participants),
                replies: nil,
                completedExchange: nil,
                savedGame: nil,
                savedGames: nil,
                activity: nil
            )
        )
    }

    func player(
        _ player: GKPlayer,
        receivedExchangeCancellation exchange: GKTurnBasedExchange,
        for match: GKTurnBasedMatch
    ) {
        _ = emit(
            GKLocalPlayerEventPayload(
                kind: "receivedExchangeCancellation",
                player: gkPlayerPayload(from: player),
                recipients: nil,
                playersToInvite: nil,
                matchSnapshot: try? gkTurnBasedMatchPayload(from: match),
                didBecomeActive: nil,
                exchange: gkTurnBasedExchangePayload(from: exchange, index: 0, participants: match.participants),
                replies: nil,
                completedExchange: nil,
                savedGame: nil,
                savedGames: nil,
                activity: nil
            )
        )
    }

    func player(
        _ player: GKPlayer,
        receivedExchangeReplies replies: [GKTurnBasedExchangeReply],
        forCompletedExchange exchange: GKTurnBasedExchange,
        for match: GKTurnBasedMatch
    ) {
        _ = emit(
            GKLocalPlayerEventPayload(
                kind: "receivedExchangeReplies",
                player: gkPlayerPayload(from: player),
                recipients: nil,
                playersToInvite: nil,
                matchSnapshot: try? gkTurnBasedMatchPayload(from: match),
                didBecomeActive: nil,
                exchange: nil,
                replies: replies.map { gkTurnBasedExchangeReplyPayload(from: $0, participants: match.participants) },
                completedExchange: gkTurnBasedExchangePayload(from: exchange, index: 0, participants: match.participants),
                savedGame: nil,
                savedGames: nil,
                activity: nil
            )
        )
    }

    func player(_ player: GKPlayer, wantsToQuitMatch match: GKTurnBasedMatch) {
        _ = emit(
            GKLocalPlayerEventPayload(
                kind: "wantsToQuitMatch",
                player: gkPlayerPayload(from: player),
                recipients: nil,
                playersToInvite: nil,
                matchSnapshot: try? gkTurnBasedMatchPayload(from: match),
                didBecomeActive: nil,
                exchange: nil,
                replies: nil,
                completedExchange: nil,
                savedGame: nil,
                savedGames: nil,
                activity: nil
            )
        )
    }

    func player(_ player: GKPlayer, didModifySavedGame savedGame: GKSavedGame) {
        _ = emit(
            GKLocalPlayerEventPayload(
                kind: "modifiedSavedGame",
                player: gkPlayerPayload(from: player),
                recipients: nil,
                playersToInvite: nil,
                matchSnapshot: nil,
                didBecomeActive: nil,
                exchange: nil,
                replies: nil,
                completedExchange: nil,
                savedGame: gkSavedGamePayload(from: savedGame),
                savedGames: nil,
                activity: nil
            )
        )
    }

    func player(_ player: GKPlayer, hasConflictingSavedGames savedGames: [GKSavedGame]) {
        _ = emit(
            GKLocalPlayerEventPayload(
                kind: "conflictingSavedGames",
                player: gkPlayerPayload(from: player),
                recipients: nil,
                playersToInvite: nil,
                matchSnapshot: nil,
                didBecomeActive: nil,
                exchange: nil,
                replies: nil,
                completedExchange: nil,
                savedGame: nil,
                savedGames: savedGames.map(gkSavedGamePayload(from:)),
                activity: nil
            )
        )
    }

    #if GAMEKIT_HAS_MACOS26_SDK
    @available(macOS 26.0, *)
    func player(
        _ player: GKPlayer,
        wantsToPlay activity: GKGameActivity,
        completionHandler: @escaping (Bool) -> Void
    ) {
        let handled = emit(
            GKLocalPlayerEventPayload(
                kind: "wantsToPlayGameActivity",
                player: gkPlayerPayload(from: player),
                recipients: nil,
                playersToInvite: nil,
                matchSnapshot: nil,
                didBecomeActive: nil,
                exchange: nil,
                replies: nil,
                completedExchange: nil,
                savedGame: nil,
                savedGames: nil,
                activity: gkGameActivityPayload(from: activity)
            )
        ) != 0
        completionHandler(handled)
    }
    #endif
}

@_cdecl("gk_local_player_listener_register")
public func gk_local_player_listener_register(
    _ callback: GKLocalPlayerListenerCallbackFn?,
    _ refcon: UnsafeMutableRawPointer?,
    _ outListenerPtr: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let callback else {
        gkPopulateError(outError, with: GKBridgeError.unknown("missing local-player listener callback"))
        return GK_UNKNOWN
    }

    let listener = GKLocalPlayerListenerImpl(callback: callback, refcon: refcon)
    GKLocalPlayer.local.register(listener)
    outListenerPtr?.pointee = gk_retain(listener)
    return GK_OK
}

@_cdecl("gk_local_player_listener_unregister")
public func gk_local_player_listener_unregister(_ ptr: UnsafeMutableRawPointer?) {
    guard let ptr else { return }
    let listener = gk_borrow(ptr, as: GKLocalPlayerListenerImpl.self)
    GKLocalPlayer.local.unregisterListener(listener)
    gk_release(ptr)
}

@_cdecl("gk_local_player_unregister_all_listeners")
public func gk_local_player_unregister_all_listeners() {
    GKLocalPlayer.local.unregisterAllListeners()
}
