import Foundation
import GameKit

@_cdecl("gk_notification_show")
public func gk_notification_show(
    _ title: UnsafePointer<CChar>?,
    _ message: UnsafePointer<CChar>?
) {
    GKNotificationBanner.show(withTitle: title.map(String.init(cString:)), message: message.map(String.init(cString:))) {}
}

@_cdecl("gk_notification_show_with_duration")
public func gk_notification_show_with_duration(
    _ title: UnsafePointer<CChar>?,
    _ message: UnsafePointer<CChar>?,
    _ durationSeconds: Double
) {
    GKNotificationBanner.show(withTitle: title.map(String.init(cString:)), message: message.map(String.init(cString:)), duration: durationSeconds) {}
}
