import Foundation

/// Un « groupe » de sessions opencode : un projet, un serveur (port) et un
/// nombre de workers. Plusieurs groupes peuvent tourner en parallèle pour
/// travailler sur plusieurs projets en même temps.
struct LaunchGroup: Identifiable, Codable, Equatable {
    var id: UUID = UUID()
    var name: String = "Nouveau groupe"
    var projectDir: String = NSHomeDirectory() + "/Documents/devSetup"
    var port: Int = 4096
    var workerCount: Int = 3
    var startServer: Bool = true
    var seedWorkers: Bool = true
    var orchestratorAgent: String = "orchestrator"
    var workerAgent: String = "worker"
    var terminalKind: TerminalKind = .iterm2
    /// Active l'autopilot pour ce groupe : le serveur reçoit les variables
    /// d'environnement qui scopent le superviseur sur ce projet et ces workers.
    var autopilot: Bool = false
    /// Intervalle entre deux cycles d'autopilot (minutes).
    var autopilotIntervalMin: Int = 30

    /// Utilise le meilleur terminal disponible sur la machine.
    init() {
        if let best = TerminalKind.available().first {
            terminalKind = best
        }
    }

    /// Décodage tolérant : les groupes enregistrés avant l'ajout de nouveaux
    /// champs restent lisibles (valeurs par défaut au lieu d'un échec).
    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        id = try c.decodeIfPresent(UUID.self, forKey: .id) ?? UUID()
        name = try c.decodeIfPresent(String.self, forKey: .name) ?? "Nouveau groupe"
        projectDir = try c.decodeIfPresent(String.self, forKey: .projectDir) ?? NSHomeDirectory() + "/Documents/devSetup"
        port = try c.decodeIfPresent(Int.self, forKey: .port) ?? 4096
        workerCount = try c.decodeIfPresent(Int.self, forKey: .workerCount) ?? 3
        startServer = try c.decodeIfPresent(Bool.self, forKey: .startServer) ?? true
        seedWorkers = try c.decodeIfPresent(Bool.self, forKey: .seedWorkers) ?? true
        orchestratorAgent = try c.decodeIfPresent(String.self, forKey: .orchestratorAgent) ?? "orchestrator"
        workerAgent = try c.decodeIfPresent(String.self, forKey: .workerAgent) ?? "worker"
        terminalKind = try c.decodeIfPresent(TerminalKind.self, forKey: .terminalKind)
            ?? (TerminalKind.available().first ?? .terminal)
        autopilot = try c.decodeIfPresent(Bool.self, forKey: .autopilot) ?? false
        autopilotIntervalMin = try c.decodeIfPresent(Int.self, forKey: .autopilotIntervalMin) ?? 30
    }

    /// Titre distinctif du panneau orchestrateur, utilisé pour détecter une
    /// grille iTerm2 déjà ouverte pour CE groupe (et éviter les doublons).
    var paneMarker: String { "Orchestrateur · \(name)" }

    var agentCount: Int { workerCount + 1 }
}
