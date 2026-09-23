import Foundation
import GameKit

struct GKAccessPointFramePayload: Codable {
    let x: Double
    let y: Double
    let width: Double
    let height: Double
}

struct GKAccessPointPayload: Codable {
    let isActive: Bool
    let isVisible: Bool
    let isPresentingGameCenter: Bool
    let location: String
    let frame: GKAccessPointFramePayload
}

private func gkAccessPointLocationString(_ location: GKAccessPoint.Location) -> String {
    switch location {
    case .topTrailing:
        return "topTrailing"
    case .bottomLeading:
        return "bottomLeading"
    case .bottomTrailing:
        return "bottomTrailing"
    default:
        return "topLeading"
    }
}

private func gkAccessPointLocation(from rawValue: Int32) -> GKAccessPoint.Location {
    switch rawValue {
    case 1:
        return .topTrailing
    case 2:
        return .bottomLeading
    case 3:
        return .bottomTrailing
    default:
        return .topLeading
    }
}

private func gkAccessPointState(from rawValue: Int32) -> GKGameCenterViewControllerState {
    switch rawValue {
    case 0:
        return .leaderboards
    case 1:
        return .achievements
    case 2:
        return .challenges
    case 3:
        return .localPlayerProfile
    case 4:
        return .dashboard
    case 5:
        return .localPlayerFriendsList
    default:
        return .default
    }
}

private func gkAccessPointPayload() -> GKAccessPointPayload {
    let accessPoint = GKAccessPoint.shared
    let frame = accessPoint.frameInScreenCoordinates
    return GKAccessPointPayload(
        isActive: accessPoint.isActive,
        isVisible: accessPoint.isVisible,
        isPresentingGameCenter: accessPoint.isPresentingGameCenter,
        location: gkAccessPointLocationString(accessPoint.location),
        frame: GKAccessPointFramePayload(
            x: frame.origin.x,
            y: frame.origin.y,
            width: frame.size.width,
            height: frame.size.height
        )
    )
}

@_cdecl("gk_access_point_json")
public func gk_access_point_json(
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        outJSON?.pointee = gkCString(try gkEncodeJSON(gkAccessPointPayload()))
        return GK_OK
    } catch {
        gkPopulateError(outError, with: error)
        return gkStatusFor(error)
    }
}

@_cdecl("gk_access_point_set_active")
public func gk_access_point_set_active(_ active: Bool) {
    GKAccessPoint.shared.isActive = active
}

@_cdecl("gk_access_point_set_location")
public func gk_access_point_set_location(_ location: Int32) {
    GKAccessPoint.shared.location = gkAccessPointLocation(from: location)
}

public typealias GKAccessPointCompletionFn = @convention(c) (
    UnsafeMutableRawPointer?,
    Int32,
    UnsafeMutablePointer<CChar>?
) -> Void

final class GKAccessPointCompletion: @unchecked Sendable {
    private var callback: GKAccessPointCompletionFn?
    private let refcon: UnsafeMutableRawPointer?
    private let discard: GKContextCallback?

    init(
        callback: GKAccessPointCompletionFn?,
        refcon: UnsafeMutableRawPointer?,
        discard: GKContextCallback?
    ) {
        self.callback = callback
        self.refcon = refcon
        self.discard = discard
    }

    func finish(_ error: Error?) {
        guard let callback else { return }
        self.callback = nil
        guard let error else {
            callback(refcon, GK_OK, nil)
            return
        }
        var message: UnsafeMutablePointer<CChar>?
        gkPopulateError(&message, with: error)
        callback(refcon, gkStatusFor(error), message)
    }

    deinit {
        if callback != nil {
            discard?(refcon)
        }
    }
}

private typealias GKAccessPointAction = (GKAccessPointCompletion) -> Void

private typealias GKAccessPointBlockTrigger = @convention(c) (
    AnyObject,
    Selector,
    (@convention(block) () -> Void)?
) -> Void

private func gkAccessPointUnavailable(_ target: String, _ version: String) -> GKBridgeError {
    .unavailable("triggering the access point for \(target) requires macOS \(version) or newer")
}

private func gkAccessPointIdentifier(_ identifier: String?, _ description: String) throws -> String {
    guard let identifier else {
        throw GKBridgeError.unknown("missing \(description)")
    }
    return identifier
}

private func gkAccessPointPlayerAction(_ gamePlayerID: String) throws -> GKAccessPointAction {
    guard #available(macOS 15.0, *) else {
        throw gkAccessPointUnavailable("a player", "15.0")
    }
    let localPlayer = GKLocalPlayer.local
    if localPlayer.gamePlayerID == gamePlayerID {
        return { completion in
            GKAccessPoint.shared.trigger(player: localPlayer) { completion.finish(nil) }
        }
    }
    try gkRequireBundleIdentifier("triggering the access point for a player")
    return { completion in
        Task { @MainActor in
            do {
                let players = try await gkResolvePlayers(byGameIDs: [gamePlayerID])
                guard let player = players[gamePlayerID] else {
                    throw GKBridgeError.notFound("player \(gamePlayerID) was not found")
                }
                GKAccessPoint.shared.trigger(player: player) { completion.finish(nil) }
            } catch {
                completion.finish(error)
            }
        }
    }
}

private func gkAccessPointArcadeAction() throws -> GKAccessPointAction {
    let selector = NSSelectorFromString("triggerAccessPointForArcadeWithHandler:")
    guard #available(macOS 26.2, *), GKAccessPoint.instancesRespond(to: selector),
          let implementation = GKAccessPoint.instanceMethod(for: selector) else {
        throw gkAccessPointUnavailable("Arcade", "26.2")
    }
    let trigger = unsafeBitCast(implementation, to: GKAccessPointBlockTrigger.self)
    return { completion in
        trigger(GKAccessPoint.shared, selector) { completion.finish(nil) }
    }
}

#if GAMEKIT_HAS_MACOS26_SDK
private func gkAccessPointMacOS26Action(
    kind: Int32,
    identifier: String?,
    activityPtr: UnsafeMutableRawPointer?
) throws -> GKAccessPointAction {
    guard #available(macOS 26.0, *) else {
        throw gkAccessPointUnavailable("this content", "26.0")
    }
    switch kind {
    case 6:
        return { completion in GKAccessPoint.shared.triggerForPlayTogether { completion.finish(nil) } }
    case 7:
        return { completion in GKAccessPoint.shared.triggerForChallenges { completion.finish(nil) } }
    case 8:
        let definitionID = try gkAccessPointIdentifier(identifier, "challenge definition identifier")
        return { completion in
            GKAccessPoint.shared.trigger(challengeDefinitionID: definitionID) { completion.finish(nil) }
        }
    case 9:
        let definitionID = try gkAccessPointIdentifier(identifier, "game activity definition identifier")
        return { completion in
            GKAccessPoint.shared.trigger(gameActivityDefinitionID: definitionID) { completion.finish(nil) }
        }
    case 10:
        guard let activityPtr else {
            throw GKBridgeError.unknown("null game activity pointer")
        }
        let activity = gk_borrow(activityPtr, as: GKGameActivity.self)
        return { completion in
            GKAccessPoint.shared.trigger(gameActivity: activity) { completion.finish(nil) }
        }
    default:
        return { completion in GKAccessPoint.shared.triggerForFriending { completion.finish(nil) } }
    }
}
#endif

private func gkAccessPointAction(
    kind: Int32,
    state: Int32,
    identifier: String?,
    playerScope: Int32,
    timeScope: Int32,
    activityPtr: UnsafeMutableRawPointer?
) throws -> GKAccessPointAction {
    switch kind {
    case 0:
        return { completion in GKAccessPoint.shared.trigger { completion.finish(nil) } }
    case 1:
        let viewState = gkAccessPointState(from: state)
        return { completion in GKAccessPoint.shared.trigger(state: viewState) { completion.finish(nil) } }
    case 2:
        let achievementID = try gkAccessPointIdentifier(identifier, "achievement identifier")
        guard #available(macOS 15.0, *) else {
            throw gkAccessPointUnavailable("an achievement", "15.0")
        }
        return { completion in
            GKAccessPoint.shared.trigger(achievementID: achievementID) { completion.finish(nil) }
        }
    case 3:
        let leaderboardSetID = try gkAccessPointIdentifier(identifier, "leaderboard set identifier")
        guard #available(macOS 15.0, *) else {
            throw gkAccessPointUnavailable("a leaderboard set", "15.0")
        }
        return { completion in
            GKAccessPoint.shared.trigger(leaderboardSetID: leaderboardSetID) { completion.finish(nil) }
        }
    case 4:
        let leaderboardID = try gkAccessPointIdentifier(identifier, "leaderboard identifier")
        guard #available(macOS 15.0, *) else {
            throw gkAccessPointUnavailable("a leaderboard", "15.0")
        }
        let playerScope = GKLeaderboard.PlayerScope(rawValue: Int(playerScope)) ?? .global
        let timeScope = GKLeaderboard.TimeScope(rawValue: Int(timeScope)) ?? .allTime
        return { completion in
            GKAccessPoint.shared.trigger(
                leaderboardID: leaderboardID,
                playerScope: playerScope,
                timeScope: timeScope
            ) { completion.finish(nil) }
        }
    case 5:
        return try gkAccessPointPlayerAction(gkAccessPointIdentifier(identifier, "player gamePlayerID"))
    case 6...11:
        #if GAMEKIT_HAS_MACOS26_SDK
        return try gkAccessPointMacOS26Action(kind: kind, identifier: identifier, activityPtr: activityPtr)
        #else
        throw GKBridgeError.unavailable("this access point target requires building against the macOS 26 SDK")
        #endif
    case 12:
        return try gkAccessPointArcadeAction()
    default:
        throw GKBridgeError.unknown("unknown access point target \(kind)")
    }
}

@_cdecl("gk_access_point_trigger_target")
public func gk_access_point_trigger_target(
    _ kind: Int32,
    _ state: Int32,
    _ identifier: UnsafePointer<CChar>?,
    _ playerScope: Int32,
    _ timeScope: Int32,
    _ activityPtr: UnsafeMutableRawPointer?,
    _ callback: GKAccessPointCompletionFn?,
    _ refcon: UnsafeMutableRawPointer?,
    _ discard: GKContextCallback?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let completion = GKAccessPointCompletion(callback: callback, refcon: refcon, discard: discard)
    do {
        let action = try gkAccessPointAction(
            kind: kind,
            state: state,
            identifier: gkCopyString(identifier),
            playerScope: playerScope,
            timeScope: timeScope,
            activityPtr: activityPtr
        )
        if Thread.isMainThread {
            action(completion)
        } else {
            DispatchQueue.main.async { action(completion) }
        }
        return GK_OK
    } catch {
        gkPopulateError(outError, with: error)
        return gkStatusFor(error)
    }
}
