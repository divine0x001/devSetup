import Foundation

/// Session renvoyée par `GET /session` du serveur opencode.
struct SessionInfo: Identifiable, Decodable, Hashable {
    let id: String
    let title: String?
    let directory: String?
    let agent: String?
    let parentID: String?
    let time: Time?

    struct Time: Decodable, Hashable {
        let created: Double?
        let updated: Double?
    }

    var displayTitle: String {
        if let title, !title.isEmpty { return title }
        return "Sans titre"
    }

    var projectName: String {
        guard let directory, !directory.isEmpty else { return "—" }
        return (directory as NSString).lastPathComponent
    }

    var updatedAt: Date? {
        guard let ms = time?.updated ?? time?.created else { return nil }
        return Date(timeIntervalSince1970: ms / 1000)
    }

    var isSubagent: Bool { parentID != nil }

    var relativeUpdated: String {
        guard let date = updatedAt else { return "" }
        let formatter = RelativeDateTimeFormatter()
        formatter.unitsStyle = .short
        return formatter.localizedString(for: date, relativeTo: Date())
    }
}
