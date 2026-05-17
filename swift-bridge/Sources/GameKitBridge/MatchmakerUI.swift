import AppKit
import Foundation
import GameKit

public typealias GKMatchmakerViewControllerCallbackFn = @convention(c) (
    UnsafeMutableRawPointer?,
    Int32,
    UnsafePointer<CChar>?,
    UnsafeMutableRawPointer?
) -> Void

public typealias GKTurnBasedMatchmakerViewControllerCallbackFn = @convention(c) (
    UnsafeMutableRawPointer?,
    Int32,
    UnsafePointer<CChar>?
) -> Void

public typealias GKGameCenterViewControllerCallbackFn = @convention(c) (
    UnsafeMutableRawPointer?
) -> Void

func gkMatchRequestPayload(from request: GKMatchRequest) -> GKMatchRequestPayload {
    GKMatchRequestPayload(
        minPlayers: request.minPlayers,
        maxPlayers: request.maxPlayers,
        playerGroup: request.playerGroup,
        playerAttributes: request.playerAttributes,
        recipientIds: request.recipients?.map(\.gamePlayerID) ?? [],
        inviteMessage: request.inviteMessage,
        defaultNumberOfPlayers: request.defaultNumberOfPlayers
    )
}

final class GKMatchmakerViewControllerDelegateImpl: NSObject, GKMatchmakerViewControllerDelegate {
    var callback: GKMatchmakerViewControllerCallbackFn?
    var refcon: UnsafeMutableRawPointer?

    func matchmakerViewControllerWasCancelled(_ viewController: GKMatchmakerViewController) {
        callback?(refcon, 0, nil, nil)
    }

    func matchmakerViewController(
        _ viewController: GKMatchmakerViewController,
        didFailWithError error: Error
    ) {
        guard let callback else { return }
        let nsError = error as NSError
        let payload = GKFrameworkErrorPayload(
            domain: nsError.domain,
            code: nsError.code,
            localizedDescription: nsError.localizedDescription
        )
        guard let json = try? gkEncodeJSON(payload), let cString = gkCString(json) else {
            callback(refcon, 1, nil, nil)
            return
        }
        defer { free(cString) }
        callback(refcon, 1, UnsafePointer(cString), nil)
    }

    func matchmakerViewController(
        _ viewController: GKMatchmakerViewController,
        didFind match: GKMatch
    ) {
        callback?(refcon, 2, nil, gk_retain(GKMatchBox(match: match)))
    }

    func matchmakerViewController(
        _ viewController: GKMatchmakerViewController,
        didFindHostedPlayers players: [GKPlayer]
    ) {
        guard let callback else { return }
        guard let json = try? gkEncodeJSON(players.map(gkPlayerPayload(from:))),
              let cString = gkCString(json)
        else {
            callback(refcon, 3, nil, nil)
            return
        }
        defer { free(cString) }
        callback(refcon, 3, UnsafePointer(cString), nil)
    }

    func matchmakerViewController(
        _ viewController: GKMatchmakerViewController,
        hostedPlayerDidAccept player: GKPlayer
    ) {
        guard let callback else { return }
        guard let json = try? gkEncodeJSON(gkPlayerPayload(from: player)), let cString = gkCString(json) else {
            callback(refcon, 4, nil, nil)
            return
        }
        defer { free(cString) }
        callback(refcon, 4, UnsafePointer(cString), nil)
    }
}

final class GKMatchmakerViewControllerBox {
    let controller: GKMatchmakerViewController
    let delegate: GKMatchmakerViewControllerDelegateImpl

    init(controller: GKMatchmakerViewController) {
        self.controller = controller
        let delegate = GKMatchmakerViewControllerDelegateImpl()
        self.delegate = delegate
        controller.matchmakerDelegate = delegate
    }

    deinit {
        controller.matchmakerDelegate = nil
    }
}

final class GKTurnBasedMatchmakerViewControllerDelegateImpl: NSObject, GKTurnBasedMatchmakerViewControllerDelegate {
    var callback: GKTurnBasedMatchmakerViewControllerCallbackFn?
    var refcon: UnsafeMutableRawPointer?

    func turnBasedMatchmakerViewControllerWasCancelled(
        _ viewController: GKTurnBasedMatchmakerViewController
    ) {
        callback?(refcon, 0, nil)
    }

    func turnBasedMatchmakerViewController(
        _ viewController: GKTurnBasedMatchmakerViewController,
        didFailWithError error: Error
    ) {
        guard let callback else { return }
        let nsError = error as NSError
        let payload = GKFrameworkErrorPayload(
            domain: nsError.domain,
            code: nsError.code,
            localizedDescription: nsError.localizedDescription
        )
        guard let json = try? gkEncodeJSON(payload), let cString = gkCString(json) else {
            callback(refcon, 1, nil)
            return
        }
        defer { free(cString) }
        callback(refcon, 1, UnsafePointer(cString))
    }
}

final class GKTurnBasedMatchmakerViewControllerBox {
    let controller: GKTurnBasedMatchmakerViewController
    let delegate: GKTurnBasedMatchmakerViewControllerDelegateImpl

    init(controller: GKTurnBasedMatchmakerViewController) {
        self.controller = controller
        let delegate = GKTurnBasedMatchmakerViewControllerDelegateImpl()
        self.delegate = delegate
        controller.turnBasedMatchmakerDelegate = delegate
    }

    deinit {
        controller.turnBasedMatchmakerDelegate = nil
    }
}

final class GKGameCenterViewControllerDelegateImpl: NSObject, GKGameCenterControllerDelegate {
    var callback: GKGameCenterViewControllerCallbackFn?
    var refcon: UnsafeMutableRawPointer?

    func gameCenterViewControllerDidFinish(_ gameCenterViewController: GKGameCenterViewController) {
        _ = try? gkDialogControllerForPresentation().dismiss(NSNull())
        callback?(refcon)
    }
}

final class GKGameCenterViewControllerBox {
    let controller: GKGameCenterViewController
    let delegate: GKGameCenterViewControllerDelegateImpl

    init(controller: GKGameCenterViewController) {
        self.controller = controller
        let delegate = GKGameCenterViewControllerDelegateImpl()
        self.delegate = delegate
        controller.gameCenterDelegate = delegate
    }

    deinit {
        controller.gameCenterDelegate = nil
    }
}

private func gkMakeGameCenterViewController(state: Int32) throws -> GKGameCenterViewController {
    let viewState: GKGameCenterViewControllerState
    switch state {
    case 0:
        viewState = .leaderboards
    case 1:
        viewState = .achievements
    case 2:
        viewState = .challenges
    case 3:
        viewState = .localPlayerProfile
    case 4:
        viewState = .dashboard
    case 5:
        guard #available(macOS 12.0, *) else {
            throw GKBridgeError.unavailable("the Game Center friends list requires macOS 12.0 or newer")
        }
        viewState = .localPlayerFriendsList
    default:
        viewState = .default
    }

    return GKGameCenterViewController(state: viewState)
}

private func gkDialogControllerForPresentation() throws -> GKDialogController {
    let dialog = GKDialogController.shared()
    guard let window = NSApplication.shared.mainWindow ?? NSApplication.shared.keyWindow else {
        throw GKBridgeError.notFound("no main or key window is available for Game Center dialog presentation")
    }
    dialog.parentWindow = window
    return dialog
}

@_cdecl("gk_invite_retain")
public func gk_invite_retain(_ ptr: UnsafeMutableRawPointer?) -> UnsafeMutableRawPointer? {
    guard let ptr else { return nil }
    return gk_retain(gk_borrow(ptr, as: GKInvite.self))
}

@_cdecl("gk_invite_release")
public func gk_invite_release(_ ptr: UnsafeMutableRawPointer?) {
    guard let ptr else { return }
    gk_release(ptr)
}

@_cdecl("gk_invite_sender_json")
public func gk_invite_sender_json(
    _ ptr: UnsafeMutableRawPointer?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null invite pointer"))
        return GK_UNKNOWN
    }

    do {
        let invite = gk_borrow(ptr, as: GKInvite.self)
        outJSON?.pointee = gkCString(try gkEncodeJSON(gkPlayerPayload(from: invite.sender)))
        return GK_OK
    } catch {
        gkPopulateError(outError, with: error)
        return gkStatusFor(error)
    }
}

@_cdecl("gk_invite_is_hosted")
public func gk_invite_is_hosted(_ ptr: UnsafeMutableRawPointer?) -> Bool {
    guard let ptr else { return false }
    return gk_borrow(ptr, as: GKInvite.self).isHosted
}

@_cdecl("gk_invite_player_group")
public func gk_invite_player_group(_ ptr: UnsafeMutableRawPointer?) -> Int {
    guard let ptr else { return 0 }
    return Int(gk_borrow(ptr, as: GKInvite.self).playerGroup)
}

@_cdecl("gk_invite_player_attributes")
public func gk_invite_player_attributes(_ ptr: UnsafeMutableRawPointer?) -> UInt32 {
    guard let ptr else { return 0 }
    return gk_borrow(ptr, as: GKInvite.self).playerAttributes
}

@_cdecl("gk_matchmaker_view_controller_create")
public func gk_matchmaker_view_controller_create(
    _ requestJSON: UnsafePointer<CChar>?,
    _ outPtr: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            let payload = try gkDecodeJSON(requestJSON, as: GKMatchRequestPayload.self)
            let request = try await gkMakeMatchRequest(from: payload)
            guard let controller = GKMatchmakerViewController(matchRequest: request) else {
                throw GKBridgeError.unknown("GKMatchmakerViewController init(matchRequest:) returned nil")
            }
            return controller
        },
        onSuccess: { controller in
            outPtr?.pointee = gk_retain(GKMatchmakerViewControllerBox(controller: controller))
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_matchmaker_view_controller_create_with_invite")
public func gk_matchmaker_view_controller_create_with_invite(
    _ invitePtr: UnsafeMutableRawPointer?,
    _ outPtr: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let invitePtr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null invite pointer"))
        return GK_UNKNOWN
    }

    do {
        let invite = gk_borrow(invitePtr, as: GKInvite.self)
        guard let controller = GKMatchmakerViewController(invite: invite) else {
            throw GKBridgeError.unknown("GKMatchmakerViewController init(invite:) returned nil")
        }
        outPtr?.pointee = gk_retain(GKMatchmakerViewControllerBox(controller: controller))
        return GK_OK
    } catch {
        gkPopulateError(outError, with: error)
        return gkStatusFor(error)
    }
}

@_cdecl("gk_matchmaker_view_controller_retain")
public func gk_matchmaker_view_controller_retain(
    _ ptr: UnsafeMutableRawPointer?
) -> UnsafeMutableRawPointer? {
    guard let ptr else { return nil }
    return gk_retain(gk_borrow(ptr, as: GKMatchmakerViewControllerBox.self))
}

@_cdecl("gk_matchmaker_view_controller_release")
public func gk_matchmaker_view_controller_release(_ ptr: UnsafeMutableRawPointer?) {
    guard let ptr else { return }
    gk_release(ptr)
}

@_cdecl("gk_matchmaker_view_controller_match_request_json")
public func gk_matchmaker_view_controller_match_request_json(
    _ ptr: UnsafeMutableRawPointer?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null matchmaker view controller pointer"))
        return GK_UNKNOWN
    }

    do {
        let box = gk_borrow(ptr, as: GKMatchmakerViewControllerBox.self)
        outJSON?.pointee = gkCString(try gkEncodeJSON(gkMatchRequestPayload(from: box.controller.matchRequest)))
        return GK_OK
    } catch {
        gkPopulateError(outError, with: error)
        return gkStatusFor(error)
    }
}

@_cdecl("gk_matchmaker_view_controller_is_hosted")
public func gk_matchmaker_view_controller_is_hosted(_ ptr: UnsafeMutableRawPointer?) -> Bool {
    guard let ptr else { return false }
    return gk_borrow(ptr, as: GKMatchmakerViewControllerBox.self).controller.isHosted
}

@_cdecl("gk_matchmaker_view_controller_set_hosted")
public func gk_matchmaker_view_controller_set_hosted(
    _ ptr: UnsafeMutableRawPointer?,
    _ hosted: Bool
) {
    guard let ptr else { return }
    gk_borrow(ptr, as: GKMatchmakerViewControllerBox.self).controller.isHosted = hosted
}

@_cdecl("gk_matchmaker_view_controller_matchmaking_mode")
public func gk_matchmaker_view_controller_matchmaking_mode(_ ptr: UnsafeMutableRawPointer?) -> Int32 {
    guard let ptr else { return 0 }
    return Int32(gk_borrow(ptr, as: GKMatchmakerViewControllerBox.self).controller.matchmakingMode.rawValue)
}

@_cdecl("gk_matchmaker_view_controller_set_matchmaking_mode")
public func gk_matchmaker_view_controller_set_matchmaking_mode(
    _ ptr: UnsafeMutableRawPointer?,
    _ mode: Int32
) {
    guard let ptr else { return }
    guard let value = GKMatchmakingMode(rawValue: Int(mode)) else { return }
    gk_borrow(ptr, as: GKMatchmakerViewControllerBox.self).controller.matchmakingMode = value
}

@_cdecl("gk_matchmaker_view_controller_can_start_with_minimum_players")
public func gk_matchmaker_view_controller_can_start_with_minimum_players(
    _ ptr: UnsafeMutableRawPointer?
) -> Bool {
    guard let ptr else { return false }
    return gk_borrow(ptr, as: GKMatchmakerViewControllerBox.self).controller.canStartWithMinimumPlayers
}

@_cdecl("gk_matchmaker_view_controller_set_can_start_with_minimum_players")
public func gk_matchmaker_view_controller_set_can_start_with_minimum_players(
    _ ptr: UnsafeMutableRawPointer?,
    _ enabled: Bool
) {
    guard let ptr else { return }
    gk_borrow(ptr, as: GKMatchmakerViewControllerBox.self).controller.canStartWithMinimumPlayers = enabled
}

@_cdecl("gk_matchmaker_view_controller_add_players_to_match")
public func gk_matchmaker_view_controller_add_players_to_match(
    _ ptr: UnsafeMutableRawPointer?,
    _ matchPtr: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr, let matchPtr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("missing controller or match pointer"))
        return GK_UNKNOWN
    }

    let controller = gk_borrow(ptr, as: GKMatchmakerViewControllerBox.self).controller
    let match = gk_borrow(matchPtr, as: GKMatchBox.self).match
    controller.addPlayers(to: match)
    return GK_OK
}

@_cdecl("gk_matchmaker_view_controller_set_hosted_player_connected")
public func gk_matchmaker_view_controller_set_hosted_player_connected(
    _ ptr: UnsafeMutableRawPointer?,
    _ playerGameID: UnsafePointer<CChar>?,
    _ connected: Bool,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null matchmaker view controller pointer"))
        return GK_UNKNOWN
    }

    return gkBlockOnAsync(
        work: {
            guard let playerGameID else {
                throw GKBridgeError.unknown("missing hosted player gamePlayerID")
            }
            let playersByID = try await gkResolvePlayers(byGameIDs: [String(cString: playerGameID)])
            guard let player = playersByID[String(cString: playerGameID)] else {
                throw GKBridgeError.notFound("hosted player was not found")
            }
            let controller = gk_borrow(ptr, as: GKMatchmakerViewControllerBox.self).controller
            controller.setHostedPlayer(player, didConnect: connected)
            return ()
        },
        onSuccess: { (_: Void) in },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_matchmaker_view_controller_set_callbacks")
public func gk_matchmaker_view_controller_set_callbacks(
    _ ptr: UnsafeMutableRawPointer?,
    _ callback: GKMatchmakerViewControllerCallbackFn?,
    _ refcon: UnsafeMutableRawPointer?
) {
    guard let ptr else { return }
    let box = gk_borrow(ptr, as: GKMatchmakerViewControllerBox.self)
    box.delegate.callback = callback
    box.delegate.refcon = refcon
}

@_cdecl("gk_matchmaker_view_controller_clear_callbacks")
public func gk_matchmaker_view_controller_clear_callbacks(_ ptr: UnsafeMutableRawPointer?) {
    guard let ptr else { return }
    let box = gk_borrow(ptr, as: GKMatchmakerViewControllerBox.self)
    box.delegate.callback = nil
    box.delegate.refcon = nil
}

@_cdecl("gk_turn_based_matchmaker_view_controller_create")
public func gk_turn_based_matchmaker_view_controller_create(
    _ requestJSON: UnsafePointer<CChar>?,
    _ outPtr: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            let payload = try gkDecodeJSON(requestJSON, as: GKTurnBasedMatchRequestPayload.self)
            let request = try await gkMakeTurnBasedRequest(from: payload)
            let controller = GKTurnBasedMatchmakerViewController(matchRequest: request)
            return controller
        },
        onSuccess: { controller in
            outPtr?.pointee = gk_retain(GKTurnBasedMatchmakerViewControllerBox(controller: controller))
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_turn_based_matchmaker_view_controller_retain")
public func gk_turn_based_matchmaker_view_controller_retain(
    _ ptr: UnsafeMutableRawPointer?
) -> UnsafeMutableRawPointer? {
    guard let ptr else { return nil }
    return gk_retain(gk_borrow(ptr, as: GKTurnBasedMatchmakerViewControllerBox.self))
}

@_cdecl("gk_turn_based_matchmaker_view_controller_release")
public func gk_turn_based_matchmaker_view_controller_release(_ ptr: UnsafeMutableRawPointer?) {
    guard let ptr else { return }
    gk_release(ptr)
}

@_cdecl("gk_turn_based_matchmaker_view_controller_show_existing_matches")
public func gk_turn_based_matchmaker_view_controller_show_existing_matches(
    _ ptr: UnsafeMutableRawPointer?
) -> Bool {
    guard let ptr else { return false }
    return gk_borrow(ptr, as: GKTurnBasedMatchmakerViewControllerBox.self).controller.showExistingMatches
}

@_cdecl("gk_turn_based_matchmaker_view_controller_set_show_existing_matches")
public func gk_turn_based_matchmaker_view_controller_set_show_existing_matches(
    _ ptr: UnsafeMutableRawPointer?,
    _ showExistingMatches: Bool
) {
    guard let ptr else { return }
    gk_borrow(ptr, as: GKTurnBasedMatchmakerViewControllerBox.self).controller.showExistingMatches = showExistingMatches
}

@_cdecl("gk_turn_based_matchmaker_view_controller_matchmaking_mode")
public func gk_turn_based_matchmaker_view_controller_matchmaking_mode(
    _ ptr: UnsafeMutableRawPointer?
) -> Int32 {
    guard let ptr else { return 0 }
    return Int32(gk_borrow(ptr, as: GKTurnBasedMatchmakerViewControllerBox.self).controller.matchmakingMode.rawValue)
}

@_cdecl("gk_turn_based_matchmaker_view_controller_set_matchmaking_mode")
public func gk_turn_based_matchmaker_view_controller_set_matchmaking_mode(
    _ ptr: UnsafeMutableRawPointer?,
    _ mode: Int32
) {
    guard let ptr else { return }
    guard let value = GKMatchmakingMode(rawValue: Int(mode)) else { return }
    gk_borrow(ptr, as: GKTurnBasedMatchmakerViewControllerBox.self).controller.matchmakingMode = value
}

@_cdecl("gk_turn_based_matchmaker_view_controller_set_callbacks")
public func gk_turn_based_matchmaker_view_controller_set_callbacks(
    _ ptr: UnsafeMutableRawPointer?,
    _ callback: GKTurnBasedMatchmakerViewControllerCallbackFn?,
    _ refcon: UnsafeMutableRawPointer?
) {
    guard let ptr else { return }
    let box = gk_borrow(ptr, as: GKTurnBasedMatchmakerViewControllerBox.self)
    box.delegate.callback = callback
    box.delegate.refcon = refcon
}

@_cdecl("gk_turn_based_matchmaker_view_controller_clear_callbacks")
public func gk_turn_based_matchmaker_view_controller_clear_callbacks(
    _ ptr: UnsafeMutableRawPointer?
) {
    guard let ptr else { return }
    let box = gk_borrow(ptr, as: GKTurnBasedMatchmakerViewControllerBox.self)
    box.delegate.callback = nil
    box.delegate.refcon = nil
}

@_cdecl("gk_dialog_present_matchmaker_view_controller")
public func gk_dialog_present_matchmaker_view_controller(
    _ ptr: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null matchmaker view controller pointer"))
        return GK_UNKNOWN
    }

    do {
        let dialog = try gkDialogControllerForPresentation()
        let box = gk_borrow(ptr, as: GKMatchmakerViewControllerBox.self)
        guard dialog.present(box.controller) else {
            throw GKBridgeError.unknown("GKDialogController refused to present the matchmaking controller")
        }
        return GK_OK
    } catch {
        gkPopulateError(outError, with: error)
        return gkStatusFor(error)
    }
}

@_cdecl("gk_dialog_present_turn_based_matchmaker_view_controller")
public func gk_dialog_present_turn_based_matchmaker_view_controller(
    _ ptr: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr else {
        gkPopulateError(outError, with: GKBridgeError.unknown("null turn-based matchmaker view controller pointer"))
        return GK_UNKNOWN
    }

    do {
        let dialog = try gkDialogControllerForPresentation()
        let box = gk_borrow(ptr, as: GKTurnBasedMatchmakerViewControllerBox.self)
        guard dialog.present(box.controller) else {
            throw GKBridgeError.unknown("GKDialogController refused to present the turn-based matchmaking controller")
        }
        return GK_OK
    } catch {
        gkPopulateError(outError, with: error)
        return gkStatusFor(error)
    }
}

@_cdecl("gk_dialog_present_game_center_view")
public func gk_dialog_present_game_center_view(
    _ state: Int32,
    _ callback: GKGameCenterViewControllerCallbackFn?,
    _ refcon: UnsafeMutableRawPointer?,
    _ outPtr: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let dialog = try gkDialogControllerForPresentation()
        let controller = try gkMakeGameCenterViewController(state: state)
        let box = GKGameCenterViewControllerBox(controller: controller)
        box.delegate.callback = callback
        box.delegate.refcon = refcon
        guard dialog.present(box.controller) else {
            throw GKBridgeError.unknown("GKDialogController refused to present the Game Center controller")
        }
        outPtr?.pointee = gk_retain(box)
        return GK_OK
    } catch {
        gkPopulateError(outError, with: error)
        return gkStatusFor(error)
    }
}

@_cdecl("gk_game_center_controller_clear_callback")
public func gk_game_center_controller_clear_callback(_ ptr: UnsafeMutableRawPointer?) {
    guard let ptr else { return }
    let box = gk_borrow(ptr, as: GKGameCenterViewControllerBox.self)
    box.delegate.callback = nil
    box.delegate.refcon = nil
}

@_cdecl("gk_game_center_controller_release")
public func gk_game_center_controller_release(_ ptr: UnsafeMutableRawPointer?) {
    guard let ptr else { return }
    gk_release(ptr)
}

@_cdecl("gk_dialog_dismiss")
public func gk_dialog_dismiss(
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    GKDialogController.shared().dismiss(NSNull())
    return GK_OK
}
