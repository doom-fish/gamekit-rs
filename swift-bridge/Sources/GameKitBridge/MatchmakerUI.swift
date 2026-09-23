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

final class GKUICallbackRegistration<Callback> {
    let token: UInt64
    let callback: Callback
    let refcon: UnsafeMutableRawPointer?
    let releaseContext: GKContextCallback?

    init(
        token: UInt64,
        callback: Callback,
        refcon: UnsafeMutableRawPointer?,
        retainContext: GKContextCallback?,
        releaseContext: GKContextCallback?
    ) {
        self.token = token
        self.callback = callback
        self.refcon = refcon
        self.releaseContext = releaseContext
        retainContext?(refcon)
    }

    deinit {
        releaseContext?(refcon)
    }
}

private var gkNextUICallbackToken: UInt64 = 1

private func gkMakeUICallbackToken() -> UInt64 {
    let token = gkNextUICallbackToken
    gkNextUICallbackToken += 1
    return token
}

final class GKMatchmakerViewControllerDelegateImpl: NSObject, GKMatchmakerViewControllerDelegate {
    var registration: GKUICallbackRegistration<GKMatchmakerViewControllerCallbackFn>?
    var knownPlayers: [String: GKPlayer] = [:]

    private func emit(_ kind: Int32, _ json: String?, _ raw: UnsafeMutableRawPointer?) {
        guard let registration else {
            if let raw { gk_release(raw) }
            return
        }
        withExtendedLifetime(registration) {
            guard let json, let cString = gkCString(json) else {
                registration.callback(registration.refcon, kind, nil, raw)
                return
            }
            defer { free(cString) }
            registration.callback(registration.refcon, kind, UnsafePointer(cString), raw)
        }
    }

    func matchmakerViewControllerWasCancelled(_ viewController: GKMatchmakerViewController) {
        emit(0, nil, nil)
    }

    func matchmakerViewController(
        _ viewController: GKMatchmakerViewController,
        didFailWithError error: Error
    ) {
        let nsError = error as NSError
        let payload = GKFrameworkErrorPayload(
            domain: nsError.domain,
            code: nsError.code,
            localizedDescription: nsError.localizedDescription
        )
        emit(1, try? gkEncodeJSON(payload), nil)
    }

    func matchmakerViewController(
        _ viewController: GKMatchmakerViewController,
        didFind match: GKMatch
    ) {
        emit(2, nil, gk_retain(GKMatchBox(match: match)))
    }

    func matchmakerViewController(
        _ viewController: GKMatchmakerViewController,
        didFindHostedPlayers players: [GKPlayer]
    ) {
        for player in players {
            knownPlayers[player.gamePlayerID] = player
        }
        emit(3, try? gkEncodeJSON(players.map(gkPlayerPayload(from:))), nil)
    }

    func matchmakerViewController(
        _ viewController: GKMatchmakerViewController,
        hostedPlayerDidAccept player: GKPlayer
    ) {
        knownPlayers[player.gamePlayerID] = player
        emit(4, try? gkEncodeJSON(gkPlayerPayload(from: player)), nil)
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
    var registration: GKUICallbackRegistration<GKTurnBasedMatchmakerViewControllerCallbackFn>?

    private func emit(_ kind: Int32, _ json: String?) {
        guard let registration else { return }
        withExtendedLifetime(registration) {
            guard let json, let cString = gkCString(json) else {
                registration.callback(registration.refcon, kind, nil)
                return
            }
            defer { free(cString) }
            registration.callback(registration.refcon, kind, UnsafePointer(cString))
        }
    }

    func turnBasedMatchmakerViewControllerWasCancelled(
        _ viewController: GKTurnBasedMatchmakerViewController
    ) {
        emit(0, nil)
    }

    func turnBasedMatchmakerViewController(
        _ viewController: GKTurnBasedMatchmakerViewController,
        didFailWithError error: Error
    ) {
        let nsError = error as NSError
        let payload = GKFrameworkErrorPayload(
            domain: nsError.domain,
            code: nsError.code,
            localizedDescription: nsError.localizedDescription
        )
        emit(1, try? gkEncodeJSON(payload))
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
    var registration: GKUICallbackRegistration<GKGameCenterViewControllerCallbackFn>?

    func gameCenterViewControllerDidFinish(_ gameCenterViewController: GKGameCenterViewController) {
        _ = try? gkDialogControllerForPresentation().dismiss(NSNull())
        guard let registration else { return }
        withExtendedLifetime(registration) {
            registration.callback(registration.refcon)
        }
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

func gkDialogControllerForPresentation() throws -> GKDialogController {
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

private let gkRecipientsOnMainThreadMessage =
    "a matchmaking view controller is created on the main thread, where GameKit cannot look up recipients; "
    + "leave recipients empty and invite players from its UI"

@_cdecl("gk_matchmaker_view_controller_create")
public func gk_matchmaker_view_controller_create(
    _ requestJSON: UnsafePointer<CChar>?,
    _ outPtr: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        try gkRequireMainThread("creating a matchmaking view controller")
        let payload = try gkDecodeJSON(requestJSON, as: GKMatchRequestPayload.self)
        guard payload.recipientIds.isEmpty else {
            throw GKBridgeError.unavailable(gkRecipientsOnMainThreadMessage)
        }
        guard let controller = GKMatchmakerViewController(matchRequest: gkBaseMatchRequest(from: payload)) else {
            throw GKBridgeError.unknown("GKMatchmakerViewController init(matchRequest:) returned nil")
        }
        outPtr?.pointee = gk_retain(GKMatchmakerViewControllerBox(controller: controller))
        return GK_OK
    } catch {
        gkPopulateError(outError, with: error)
        return gkStatusFor(error)
    }
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
        try gkRequireMainThread("creating a matchmaking view controller")
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
    do {
        try gkRequireMainThread("adding players from the matchmaking view controller")
    } catch {
        gkPopulateError(outError, with: error)
        return gkStatusFor(error)
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
    do {
        try gkRequireMainThread("updating a hosted player's connection state")
        guard let ptr else {
            throw GKBridgeError.unknown("null matchmaker view controller pointer")
        }
        guard let gamePlayerID = gkCopyString(playerGameID) else {
            throw GKBridgeError.unknown("missing hosted player gamePlayerID")
        }
        let box = gk_borrow(ptr, as: GKMatchmakerViewControllerBox.self)
        guard let player = box.delegate.knownPlayers[gamePlayerID] else {
            throw GKBridgeError.notFound(
                "the matchmaking view controller has not reported hosted player \(gamePlayerID)"
            )
        }
        box.controller.setHostedPlayer(player, didConnect: connected)
        return GK_OK
    } catch {
        gkPopulateError(outError, with: error)
        return gkStatusFor(error)
    }
}

@_cdecl("gk_matchmaker_view_controller_set_callbacks")
public func gk_matchmaker_view_controller_set_callbacks(
    _ ptr: UnsafeMutableRawPointer?,
    _ callback: GKMatchmakerViewControllerCallbackFn?,
    _ refcon: UnsafeMutableRawPointer?,
    _ retainContext: GKContextCallback?,
    _ releaseContext: GKContextCallback?
) -> UInt64 {
    guard let ptr, let callback, Thread.isMainThread else { return 0 }
    let registration = GKUICallbackRegistration(
        token: gkMakeUICallbackToken(),
        callback: callback,
        refcon: refcon,
        retainContext: retainContext,
        releaseContext: releaseContext
    )
    gk_borrow(ptr, as: GKMatchmakerViewControllerBox.self).delegate.registration = registration
    return registration.token
}

@_cdecl("gk_matchmaker_view_controller_clear_callbacks")
public func gk_matchmaker_view_controller_clear_callbacks(_ ptr: UnsafeMutableRawPointer?, _ token: UInt64) {
    guard let ptr, Thread.isMainThread else { return }
    let delegate = gk_borrow(ptr, as: GKMatchmakerViewControllerBox.self).delegate
    if delegate.registration?.token == token {
        delegate.registration = nil
    }
}

@_cdecl("gk_turn_based_matchmaker_view_controller_create")
public func gk_turn_based_matchmaker_view_controller_create(
    _ requestJSON: UnsafePointer<CChar>?,
    _ outPtr: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        try gkRequireMainThread("creating a turn-based matchmaking view controller")
        let payload = try gkDecodeJSON(requestJSON, as: GKTurnBasedMatchRequestPayload.self)
        guard payload.recipientIds.isEmpty else {
            throw GKBridgeError.unavailable(gkRecipientsOnMainThreadMessage)
        }
        let controller = GKTurnBasedMatchmakerViewController(matchRequest: gkBaseTurnBasedRequest(from: payload))
        outPtr?.pointee = gk_retain(GKTurnBasedMatchmakerViewControllerBox(controller: controller))
        return GK_OK
    } catch {
        gkPopulateError(outError, with: error)
        return gkStatusFor(error)
    }
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
    _ refcon: UnsafeMutableRawPointer?,
    _ retainContext: GKContextCallback?,
    _ releaseContext: GKContextCallback?
) -> UInt64 {
    guard let ptr, let callback, Thread.isMainThread else { return 0 }
    let registration = GKUICallbackRegistration(
        token: gkMakeUICallbackToken(),
        callback: callback,
        refcon: refcon,
        retainContext: retainContext,
        releaseContext: releaseContext
    )
    gk_borrow(ptr, as: GKTurnBasedMatchmakerViewControllerBox.self).delegate.registration = registration
    return registration.token
}

@_cdecl("gk_turn_based_matchmaker_view_controller_clear_callbacks")
public func gk_turn_based_matchmaker_view_controller_clear_callbacks(
    _ ptr: UnsafeMutableRawPointer?,
    _ token: UInt64
) {
    guard let ptr, Thread.isMainThread else { return }
    let delegate = gk_borrow(ptr, as: GKTurnBasedMatchmakerViewControllerBox.self).delegate
    if delegate.registration?.token == token {
        delegate.registration = nil
    }
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
        try gkRequireMainThread("presenting a matchmaking view controller")
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
        try gkRequireMainThread("presenting a turn-based matchmaking view controller")
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
    _ retainContext: GKContextCallback?,
    _ releaseContext: GKContextCallback?,
    _ outPtr: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        try gkRequireMainThread("presenting the Game Center view controller")
        let dialog = try gkDialogControllerForPresentation()
        let controller = try gkMakeGameCenterViewController(state: state)
        let box = GKGameCenterViewControllerBox(controller: controller)
        if let callback {
            box.delegate.registration = GKUICallbackRegistration(
                token: gkMakeUICallbackToken(),
                callback: callback,
                refcon: refcon,
                retainContext: retainContext,
                releaseContext: releaseContext
            )
        }
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
    guard let ptr, Thread.isMainThread else { return }
    gk_borrow(ptr, as: GKGameCenterViewControllerBox.self).delegate.registration = nil
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
    do {
        try gkRequireMainThread("dismissing a Game Center dialog")
        GKDialogController.shared().dismiss(NSNull())
        return GK_OK
    } catch {
        gkPopulateError(outError, with: error)
        return gkStatusFor(error)
    }
}
