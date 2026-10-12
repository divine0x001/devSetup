import Foundation

/// Session lue directement dans la base SQLite d'opencode (`opencode db`).
/// Fonctionne toujours : aucun serveur requis, aucun dossier courant requis,
/// toutes les sessions de tous les projets d'un coup.
struct DBSession: Identifiable, Decodable, Hashable {
    let id: String
    let title: String?
    let directory: String?
    let parent_id: String?
    let agent: String?
    let time_created: Double?
    let time_updated: Double?
    let cost: Double?

    var displayTitle: String {
        if let title, !title.isEmpty { return title }
        return "Sans titre"
    }

    var projectName: String {
        guard let directory, !directory.isEmpty else { return "—" }
        return (directory as NSString).lastPathComponent
    }

    var updatedAt: Date? {
        guard let ms = time_updated ?? time_created else { return nil }
        return Date(timeIntervalSince1970: ms / 1000)
    }

    var relativeUpdated: String {
        guard let date = updatedAt else { return "" }
        let formatter = RelativeDateTimeFormatter()
        formatter.unitsStyle = .short
        return formatter.localizedString(for: date, relativeTo: Date())
    }
}

/// Erreur simple et Sendable pour traverser la frontière des tâches.
struct DBError: Error, Sendable {
    let message: String
}

enum DatabaseSessions {
    /// Résout le chemin absolu du binaire opencode (ou nil s'il est absent).
    static func opencodePath() -> String? {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/bin/zsh")
        process.arguments = ["-lc", "command -v opencode"]
        let pipe = Pipe()
        process.standardOutput = pipe
        process.standardError = FileHandle.nullDevice
        do {
            try process.run()
            process.waitUntilExit()
            let data = pipe.fileHandleForReading.readDataToEndOfFile()
            let path = String(data: data, encoding: .utf8)?
                .trimmingCharacters(in: .whitespacesAndNewlines)
            if let path, !path.isEmpty { return path }
        } catch {}
        return nil
    }

    /// Charge toutes les sessions (racines par défaut) depuis la base, via
    /// `opencode db --format json`. Renvoie un Result pour rester Sendable.
    static func load(limit: Int = 500, includeSubagents: Bool = false) -> Result<[DBSession], DBError> {
        guard let opencode = opencodePath() else {
            return .failure(DBError(message: "binaire opencode introuvable dans le PATH"))
        }
        let sql = """
        SELECT id, title, directory, parent_id, agent, time_created, time_updated, cost \
        FROM session \
        \(includeSubagents ? "" : "WHERE parent_id IS NULL ") \
        ORDER BY time_updated DESC LIMIT \(limit)
        """
        let process = Process()
        process.executableURL = URL(fileURLWithPath: opencode)
        process.arguments = ["db", "--format", "json", sql]
        let outPipe = Pipe()
        let errPipe = Pipe()
        process.standardOutput = outPipe
        process.standardError = errPipe
        do {
            try process.run()
            process.waitUntilExit()
        } catch {
            return .failure(DBError(message: error.localizedDescription))
        }
        let data = outPipe.fileHandleForReading.readDataToEndOfFile()
        if process.terminationStatus != 0 {
            let message = String(data: errPipe.fileHandleForReading.readDataToEndOfFile(), encoding: .utf8)?
                .trimmingCharacters(in: .whitespacesAndNewlines)
            let text = (message?.isEmpty == false ? message! : "opencode db a échoué")
            return .failure(DBError(message: text))
        }
        do {
            return .success(try JSONDecoder().decode([DBSession].self, from: data))
        } catch {
            return .failure(DBError(message: "réponse illisible : \(error.localizedDescription)"))
        }
    }

    static func delete(id: String) -> Result<Void, DBError> {
        guard let opencode = opencodePath() else {
            return .failure(DBError(message: "binaire opencode introuvable dans le PATH"))
        }
        let process = Process()
        process.executableURL = URL(fileURLWithPath: opencode)
        process.arguments = ["session", "delete", id]
        process.standardOutput = FileHandle.nullDevice
        process.standardError = FileHandle.nullDevice
        do {
            try process.run()
            process.waitUntilExit()
        } catch {
            return .failure(DBError(message: error.localizedDescription))
        }
        return process.terminationStatus == 0 ? .success(()) : .failure(DBError(message: "opencode session delete a échoué"))
    }
}
