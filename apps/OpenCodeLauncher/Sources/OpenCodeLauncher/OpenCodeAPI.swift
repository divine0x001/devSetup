import Foundation

struct OpenCodeAPI {
    let base: URL
    /// Répertoire de travail transmis via l'en-tête `x-opencode-directory` :
    /// le serveur stocke les sessions par projet (clé = dossier), donc chaque
    /// requête doit viser explicitement le dossier sélectionné.
    let directory: String?

    init(base: URL, directory: String? = nil) {
        self.base = base
        self.directory = directory
    }

    struct APIError: Error, LocalizedError {
        let message: String
        var errorDescription: String? { message }
    }

    private func request(_ method: String, _ path: String, body: [String: Any]? = nil, timeout: TimeInterval = 30) async throws -> Data {
        guard let url = URL(string: path, relativeTo: base) else {
            throw APIError(message: "URL invalide : \(path)")
        }
        var req = URLRequest(url: url)
        req.httpMethod = method
        req.timeoutInterval = timeout
        if let directory, !directory.isEmpty {
            req.setValue(directory, forHTTPHeaderField: "x-opencode-directory")
        }
        if let body {
            req.setValue("application/json", forHTTPHeaderField: "Content-Type")
            req.httpBody = try JSONSerialization.data(withJSONObject: body)
        }
        let (data, resp) = try await URLSession.shared.data(for: req)
        guard let http = resp as? HTTPURLResponse else {
            throw APIError(message: "réponse HTTP invalide")
        }
        guard (200..<300).contains(http.statusCode) else {
            let detail = String(data: data, encoding: .utf8) ?? ""
            throw APIError(message: "HTTP \(http.statusCode) — \(detail.prefix(300))")
        }
        return data
    }

    func ping() async -> Bool {
        do {
            _ = try await request("GET", "/session", timeout: 3)
            return true
        } catch {
            return false
        }
    }

    /// Liste les sessions du dossier courant (le serveur scope via
    /// `x-opencode-directory`). `rootOnly` exclut les sous-sessions d'agents.
    func listSessions(limit: Int = 200, rootOnly: Bool = true) async throws -> [SessionInfo] {
        var query = "/session?limit=\(limit)&order=desc"
        if rootOnly { query += "&parentID=null" }
        let data = try await request("GET", query)
        return try JSONDecoder().decode([SessionInfo].self, from: data)
    }

    struct SessionStatus: Decodable { let type: String }

    func sessionStatus() async throws -> [String: SessionStatus] {
        let data = try await request("GET", "/session/status")
        return try JSONDecoder().decode([String: SessionStatus].self, from: data)
    }

    func deleteSession(id: String) async throws {
        _ = try await request("DELETE", "/session/\(id)")
    }

    func createSession(title: String) async throws -> String {
        let data = try await request("POST", "/session", body: ["title": title])
        guard let obj = try JSONSerialization.jsonObject(with: data) as? [String: Any],
              let id = obj["id"] as? String else {
            throw APIError(message: "identifiant de session manquant dans la réponse")
        }
        return id
    }

    func promptAsync(sessionID: String, text: String, agent: String?, noReply: Bool = false) async throws {
        var body: [String: Any] = ["parts": [["type": "text", "text": text]]]
        if let agent, !agent.isEmpty { body["agent"] = agent }
        if noReply { body["noReply"] = true }
        _ = try await request("POST", "/session/\(sessionID)/prompt_async", body: body, timeout: 60)
    }
}
