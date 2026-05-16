import Foundation
import GameKit

struct GKSavedGamePayload: Codable {
    let name: String?
    let deviceName: String?
    let modificationDate: String?
}

func gkSavedGamePayload(from savedGame: GKSavedGame) -> GKSavedGamePayload {
    GKSavedGamePayload(
        name: savedGame.name,
        deviceName: savedGame.deviceName,
        modificationDate: gkDateString(savedGame.modificationDate)
    )
}

private func gkLoadAllSavedGames() async throws -> [GKSavedGame] {
    try await withCheckedThrowingContinuation { continuation in
        GKLocalPlayer.local.fetchSavedGames { savedGames, error in
            if let error {
                continuation.resume(throwing: error)
            } else {
                continuation.resume(returning: savedGames ?? [])
            }
        }
    }
}

private func gkFindSavedGames(matching payloads: [GKSavedGamePayload]) async throws -> [GKSavedGame] {
    let savedGames = try await gkLoadAllSavedGames()
    return savedGames.filter { savedGame in
        let payload = gkSavedGamePayload(from: savedGame)
        return payloads.contains(where: {
            $0.name == payload.name &&
                $0.deviceName == payload.deviceName &&
                $0.modificationDate == payload.modificationDate
        })
    }
}

@_cdecl("gk_saved_games_json")
public func gk_saved_games_json(
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: { try await gkLoadAllSavedGames() },
        onSuccess: { (savedGames: [GKSavedGame]) in
            outJSON?.pointee = gkCString((try? gkEncodeJSON(savedGames.map(gkSavedGamePayload(from:)))) ?? "[]")
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_saved_game_load_data_json")
public func gk_saved_game_load_data_json(
    _ savedGameJSON: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            let payload = try gkDecodeJSON(savedGameJSON, as: GKSavedGamePayload.self)
            guard let savedGame = try await gkFindSavedGames(matching: [payload]).first else {
                throw GKBridgeError.notFound("saved game not found")
            }
            let data = try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Data, Error>) in
                savedGame.loadData { data, error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else {
                        continuation.resume(returning: data ?? Data())
                    }
                }
            }
            return data
        },
        onSuccess: { data in
            outJSON?.pointee = gkCString((try? gkBinaryPayloadJSON(data)) ?? "{}")
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_saved_game_save_json")
public func gk_saved_game_save_json(
    _ name: UnsafePointer<CChar>?,
    _ data: UnsafePointer<UInt8>?,
    _ len: Int,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            guard let name else {
                throw GKBridgeError.unknown("missing saved game name")
            }
            let payload = data.map { Data(bytes: $0, count: len) } ?? Data()
            return try await withCheckedThrowingContinuation { continuation in
                GKLocalPlayer.local.saveGameData(payload, withName: String(cString: name)) { savedGame, error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else if let savedGame {
                        continuation.resume(returning: savedGame)
                    } else {
                        continuation.resume(throwing: GKBridgeError.unknown("saveGameData returned nil"))
                    }
                }
            }
        },
        onSuccess: { savedGame in
            outJSON?.pointee = gkCString((try? gkEncodeJSON(gkSavedGamePayload(from: savedGame))) ?? "{}")
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_saved_game_delete")
public func gk_saved_game_delete(
    _ name: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            guard let name else {
                throw GKBridgeError.unknown("missing saved game name")
            }
            return try await withCheckedThrowingContinuation { continuation in
                GKLocalPlayer.local.deleteSavedGames(withName: String(cString: name)) { error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else {
                        continuation.resume(returning: ())
                    }
                }
            }
        },
        onSuccess: { (_: Void) in },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}

@_cdecl("gk_saved_game_resolve_conflicts_json")
public func gk_saved_game_resolve_conflicts_json(
    _ savedGamesJSON: UnsafePointer<CChar>?,
    _ data: UnsafePointer<UInt8>?,
    _ len: Int,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    gkBlockOnAsync(
        work: {
            let payloads = try gkDecodeJSON(savedGamesJSON, as: [GKSavedGamePayload].self)
            let conflictingGames = try await gkFindSavedGames(matching: payloads)
            let mergedData = data.map { Data(bytes: $0, count: len) } ?? Data()
            return try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<[GKSavedGame], Error>) in
                GKLocalPlayer.local.resolveConflictingSavedGames(conflictingGames, with: mergedData) { savedGames, error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else {
                        continuation.resume(returning: savedGames ?? [])
                    }
                }
            }
        },
        onSuccess: { (savedGames: [GKSavedGame]) in
            outJSON?.pointee = gkCString((try? gkEncodeJSON(savedGames.map(gkSavedGamePayload(from:)))) ?? "[]")
        },
        onError: { error in
            gkPopulateError(outError, with: error)
        }
    )
}
