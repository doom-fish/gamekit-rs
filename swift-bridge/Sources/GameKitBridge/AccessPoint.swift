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

@_cdecl("gk_access_point_trigger")
public func gk_access_point_trigger(
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            await withCheckedContinuation { continuation in
                GKAccessPoint.shared.trigger {
                    continuation.resume(returning: ())
                }
            }
        },
        onSuccess: { (_: Void) in },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_access_point_trigger_state")
public func gk_access_point_trigger_state(
    _ state: Int32,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            await withCheckedContinuation { continuation in
                GKAccessPoint.shared.trigger(state: gkAccessPointState(from: state)) {
                    continuation.resume(returning: ())
                }
            }
        },
        onSuccess: { (_: Void) in },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}
