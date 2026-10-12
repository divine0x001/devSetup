import Foundation
import AppKit

enum SessionSource: String, CaseIterable, Identifiable {
    case all = "Toutes"
    case group = "Groupe"
    var id: String { rawValue }
}

@MainActor
final class LauncherModel: ObservableObject {
    /// Groupes configurés (persistés). Chaque groupe = un projet + un serveur.
    @Published var groups: [LaunchGroup] = [] {
        didSet { persist() }
    }
    @Published var selectedGroupID: UUID?
    @Published var logs: String = ""
    /// Vrai pendant un « Lancer tout » (désactive les boutons globaux).
    @Published var busy: Bool = false
    /// Groupes dont le lancement est en cours.
    @Published var busyGroupIDs: Set<UUID> = []
    /// Groupes lancés avec succès.
    @Published var runningGroupIDs: Set<UUID> = []

    /// Sessions existantes du groupe sélectionné (récupérées du serveur).
    @Published var sessions: [SessionInfo] = []
    /// Statut par session : "busy" / "idle".
    @Published var sessionStatus: [String: String] = [:]
    @Published var sessionsLoading: Bool = false

    /// Source affichée dans le panneau Sessions.
    @Published var sessionSource: SessionSource = .all
    /// Toutes les sessions, lues dans la base SQLite (aucun serveur requis).
    @Published var allSessions: [DBSession] = []
    @Published var allSessionsLoading: Bool = false
    @Published var sessionFilter: String = ""

    /// Serveurs lancés par l'app, indexés par port.
    private var servers: [Int: Process] = [:]
    private let logPath = "/tmp/ocl-logs.txt"
    private static let storageKey = "ocl.groups.v1"

    init() {
        if let data = UserDefaults.standard.data(forKey: Self.storageKey),
           let saved = try? JSONDecoder().decode([LaunchGroup].self, from: data),
           !saved.isEmpty {
            groups = saved
        } else {
            groups = [LaunchGroup()]
        }
        selectedGroupID = groups.first?.id
    }

    // MARK: - Gestion des groupes

    var selectedIndex: Int? {
        groups.firstIndex { $0.id == selectedGroupID }
    }

    var selectedGroup: LaunchGroup? {
        guard let index = selectedIndex else { return nil }
        return groups[index]
    }

    private func api(for group: LaunchGroup) -> OpenCodeAPI {
        OpenCodeAPI(base: URL(string: "http://127.0.0.1:\(group.port)")!, directory: group.projectDir)
    }

    // MARK: - Sessions existantes

    /// Récupère les sessions du serveur du groupe sélectionné.
    func refreshSessions(silent: Bool = false) async {
        guard let group = selectedGroup else {
            sessions = []
            sessionStatus = [:]
            return
        }
        sessionsLoading = true
        defer { sessionsLoading = false }
        let client = api(for: group)
        do {
            sessions = try await client.listSessions()
            let status = try await client.sessionStatus()
            sessionStatus = status.mapValues(\.type)
        } catch {
            sessions = []
            sessionStatus = [:]
            if !silent {
                log("[\(group.name)] Sessions indisponibles : \(error.localizedDescription)")
            }
        }
    }

    func status(for session: SessionInfo) -> String? {
        sessionStatus[session.id]
    }

    /// Rouvre une session existante dans une fenêtre de terminal.
    func attach(session: SessionInfo) {
        guard let group = selectedGroup else { return }
        let opencode = resolveOpencodePath()
        let title = "\(session.displayTitle) · \(group.name)"
        let panel = TerminalPanel(
            title: title,
            command: attachCommand(group: group, sessionID: session.id, opencode: opencode, title: session.displayTitle)
        )
        do {
            try TerminalLauncher.openGrid(panels: [panel], kind: group.terminalKind)
            log("[\(group.name)] Session rouverte : \(session.displayTitle)")
        } catch {
            log("[\(group.name)] Échec de réouverture : \(error.localizedDescription)")
        }
    }

    // MARK: - Sessions globales (base SQLite, aucun serveur requis)

    var filteredAllSessions: [DBSession] {
        guard !sessionFilter.isEmpty else { return allSessions }
        let needle = sessionFilter.lowercased()
        return allSessions.filter {
            $0.displayTitle.lowercased().contains(needle)
                || $0.projectName.lowercased().contains(needle)
                || ($0.directory?.lowercased().contains(needle) ?? false)
                || ($0.agent?.lowercased().contains(needle) ?? false)
        }
    }

    func refreshAllSessions(silent: Bool = false) async {
        allSessionsLoading = true
        let result = await Task.detached(priority: .userInitiated) {
            DatabaseSessions.load()
        }.value
        allSessionsLoading = false
        switch result {
        case .success(let list):
            allSessions = list
        case .failure(let error):
            allSessions = []
            if !silent { log("Sessions globales indisponibles : \(error.message)") }
        }
    }

    /// Ouvre n'importe quelle session dans un terminal, sans serveur ni groupe :
    /// `opencode <dossier> -s <id>` démarre l'instance du projet et reprend la
    /// conversation là où elle en était.
    func reopen(session: DBSession) {
        guard let dir = session.directory, !dir.isEmpty else {
            log("Session « \(session.displayTitle) » sans dossier : impossible de rouvrir.")
            return
        }
        let opencode = resolveOpencodePath()
        let kind = selectedGroup?.terminalKind ?? TerminalKind.available().first ?? .terminal
        let title = session.displayTitle
        let escapedTitle = title.replacingOccurrences(of: "'", with: "'\\''")
        let base = "cd \(shellQuote(dir)) && \(shellQuote(opencode)) -s \(shellQuote(session.id))"
        let command = "\(base) || { echo; echo '⚠ Échec de la réouverture de session.'; echo 'Appuie sur Entrée pour fermer cette fenêtre.'; read -k1; }"
        do {
            try TerminalLauncher.openGrid(
                panels: [TerminalPanel(title: title, command: "printf '\\033]0;\(escapedTitle)\\007'; \(command)")],
                kind: kind
            )
            log("Session rouverte (globale) : \(title) — \(dir)")
        } catch {
            log("Échec de réouverture : \(error.localizedDescription)")
        }
    }

    func deleteGlobalSession(_ session: DBSession) async {
        let result = await Task.detached(priority: .userInitiated) {
            DatabaseSessions.delete(id: session.id)
        }.value
        switch result {
        case .success:
            allSessions.removeAll { $0.id == session.id }
            log("Session supprimée : \(session.displayTitle)")
        case .failure(let error):
            log("Échec de suppression : \(error.message)")
        }
    }

    func deleteSession(_ session: SessionInfo) async {
        guard let group = selectedGroup else { return }
        do {
            try await api(for: group).deleteSession(id: session.id)
            sessions.removeAll { $0.id == session.id }
            log("[\(group.name)] Session supprimée : \(session.displayTitle)")
        } catch {
            log("[\(group.name)] Échec de suppression : \(error.localizedDescription)")
        }
    }

    func addGroup() {
        var group = LaunchGroup()
        group.name = "Projet \(groups.count + 1)"
        group.projectDir = NSHomeDirectory() + "/Documents"
        let used = Set(groups.map(\.port))
        var port = 4096
        while used.contains(port) { port += 1 }
        group.port = port
        groups.append(group)
        selectedGroupID = group.id
    }

    func removeSelectedGroup() {
        guard let id = selectedGroupID,
              let index = groups.firstIndex(where: { $0.id == id }) else { return }
        stop(groupID: id)
        groups.remove(at: index)
        selectedGroupID = groups.first?.id
    }

    func chooseDirectory(for groupID: UUID) {
        guard let index = groups.firstIndex(where: { $0.id == groupID }) else { return }
        let panel = NSOpenPanel()
        panel.canChooseDirectories = true
        panel.canChooseFiles = false
        panel.allowsMultipleSelection = false
        panel.directoryURL = URL(fileURLWithPath: groups[index].projectDir)
        if panel.runModal() == .OK, let url = panel.url {
            groups[index].projectDir = url.path
        }
    }

    // MARK: - Journal

    private static let timeFormatter: DateFormatter = {
        let f = DateFormatter()
        f.dateFormat = "HH:mm:ss"
        return f
    }()

    func log(_ line: String) {
        let stamp = Self.timeFormatter.string(from: Date())
        let msg = "[\(stamp)] \(line)\n"
        logs += msg
        if let data = msg.data(using: .utf8) {
            let url = URL(fileURLWithPath: logPath)
            if FileManager.default.fileExists(atPath: logPath) {
                if let handle = try? FileHandle(forWritingTo: url) {
                    handle.seekToEndOfFile()
                    handle.write(data)
                    try? handle.close()
                }
            } else {
                try? data.write(to: url)
            }
        }
    }

    // MARK: - Lancement

    func launch(groupID: UUID) async {
        guard let index = groups.firstIndex(where: { $0.id == groupID }) else { return }
        logs = ""
        await launch(group: groups[index])
    }

    /// Lance tous les groupes configurés (chacun son port/serveur/projet).
    func launchAll() async {
        guard !busy else { return }
        busy = true
        logs = ""
        for group in groups {
            await launch(group: group)
        }
        busy = false
    }

    private func launch(group: LaunchGroup) async {
        busyGroupIDs.insert(group.id)
        defer { busyGroupIDs.remove(group.id) }

        let tag = "[\(group.name)]"
        let baseURL = URL(string: "http://127.0.0.1:\(group.port)")!
        let api = OpenCodeAPI(base: baseURL, directory: group.projectDir)

        do {
            let opencode = resolveOpencodePath()
            log("\(tag) Binaire opencode : \(opencode)")

            if group.terminalKind == .iterm2,
               TerminalLauncher.iterm2GridAlreadyOpen(marker: group.paneMarker) {
                log("\(tag) Agents déjà ouverts dans iTerm2 (panneau « \(group.paneMarker) » présent).")
                log("\(tag) Ferme la fenêtre iTerm2 existante puis relance pour ce groupe.")
                return
            }

            let alreadyUp = await api.ping()
            if alreadyUp {
                log("\(tag) Serveur déjà actif sur \(group.port), réutilisation.")
                log("\(tag) Sessions rattachées au dossier « \(group.projectDir) » via x-opencode-directory.")
                log("\(tag) ⚠ Auto-approbation des permissions : dépend du lancement de CE serveur (OPENCODE_PERMISSION).")
                if group.startServer {
                    log("\(tag) startServer=true mais un serveur existe déjà — aucun nouveau serveur lancé.")
                }
                if group.autopilot {
                    log("\(tag) ⚠ Autopilot demandé, mais ce serveur existant n'a pas reçu les variables de scopage.")
                    log("\(tag) Arrête ce serveur (bouton Arrêter) puis relance pour activer l'autopilot sur ce groupe.")
                }
            } else if group.startServer {
                log("\(tag) Démarrage du serveur opencode sur le port \(group.port)…")
                try startServerProcess(opencode: opencode, port: group.port, group: group)
            } else {
                log("\(tag) Mode serveur existant : connexion au port \(group.port)…")
            }

            log("\(tag) Attente de la disponibilité du serveur…")
            guard await waitForServer(api: api, timeout: 30) else {
                throw OpenCodeAPI.APIError(message: "serveur injoignable sur \(baseURL.absoluteString)")
            }
            log("\(tag) Serveur prêt.")

            let orchestratorID = try await api.createSession(title: group.paneMarker)
            log("\(tag) Session orchestrateur créée : \(orchestratorID)")

            var workers: [(title: String, id: String)] = []
            for index in 1...max(1, group.workerCount) {
                let title = "Worker \(index) · \(group.name)"
                let id = try await api.createSession(title: title)
                workers.append((title, id))
                log("\(tag) Session \(title) créée : \(id)")
            }

            var panels: [TerminalPanel] = []
            panels.append(TerminalPanel(
                title: group.paneMarker,
                command: attachCommand(group: group, sessionID: orchestratorID, opencode: opencode, title: group.paneMarker)
            ))
            for worker in workers {
                panels.append(TerminalPanel(
                    title: worker.title,
                    command: attachCommand(group: group, sessionID: worker.id, opencode: opencode, title: worker.title)
                ))
            }

            if group.terminalKind == .iterm2 {
                log("\(tag) Ouverture d'une fenêtre iTerm2 avec \(panels.count) panneaux (grille)…")
            } else {
                log("\(tag) Ouverture de \(panels.count) fenêtre(s) dans \(group.terminalKind.label)…")
            }

            do {
                try TerminalLauncher.openGrid(panels: panels, kind: group.terminalKind)
                if group.terminalKind == .iterm2 {
                    log("\(tag) Fenêtre iTerm2 ouverte : orchestrateur + \(workers.count) worker(s).")
                } else {
                    log("\(tag) Fenêtres \(group.terminalKind.label) ouvertes.")
                }
            } catch {
                log("\(tag) Erreur d'ouverture du terminal : \(error.localizedDescription)")
                log("\(tag) Repli possible : sélectionne « Ghostty » ou « Terminal », puis relance.")
                throw error
            }

            if group.seedWorkers {
                for worker in workers {
                    do {
                        try await api.promptAsync(
                            sessionID: worker.id,
                            text: "Prêt. En attente des tâches envoyées par l'orchestrateur.",
                            agent: group.workerAgent,
                            noReply: true
                        )
                        log("\(tag) \(worker.title) initialisé.")
                    } catch {
                        log("\(tag) Initialisation \(worker.title) ignorée : \(error.localizedDescription)")
                    }
                }
            }

            let prompt = group.autopilot
                ? supervisorPrompt(group: group, workers: workers)
                : orchestratorPrompt(group: group, workers: workers)
            let promptAgent = group.autopilot ? "super-orchestrator" : group.orchestratorAgent
            do {
                try await api.promptAsync(sessionID: orchestratorID, text: prompt, agent: promptAgent)
                if group.autopilot {
                    log("\(tag) Autopilot activé : superviseur lancé dans le panneau orchestrateur.")
                } else {
                    log("\(tag) Prompt initial envoyé à l'orchestrateur.")
                }
            } catch {
                log("\(tag) Échec du prompt initial : \(error.localizedDescription)")
                log("\(tag) Astuce : vérifie que tu as un provider authentifié (opencode auth login).")
            }

            runningGroupIDs.insert(group.id)
            log("\(tag) Terminé : 1 orchestrateur + \(workers.count) worker(s).")
        } catch {
            log("\(tag) Erreur : \(error.localizedDescription)")
        }
    }

    // MARK: - Arrêt

    func stop(groupID: UUID) {
        guard let group = groups.first(where: { $0.id == groupID }) else { return }
        terminateServer(port: group.port)
        runningGroupIDs.remove(group.id)
        log("[\(group.name)] Serveur arrêté.")
    }

    func stopAll() {
        for group in groups {
            stop(groupID: group.id)
        }
    }

    /// Arrête le serveur géré par l'app ou, à défaut, tout `opencode serve`
    /// à l'écoute sur le port donné (même lancé hors de l'app).
    private func terminateServer(port: Int) {
        if let process = servers[port], process.isRunning {
            process.terminate()
        }
        servers[port] = nil
        let killer = Process()
        killer.executableURL = URL(fileURLWithPath: "/usr/bin/pkill")
        killer.arguments = ["-f", "opencode serve --port \(port)"]
        try? killer.run()
        killer.waitUntilExit()
    }

    private func startServerProcess(opencode: String, port: Int, group: LaunchGroup) throws {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/bin/zsh")
        process.arguments = ["-lc", "cd \(shellQuote(group.projectDir)) && exec \(shellQuote(opencode)) serve --port \(port)"]
        // Équivalent de `--auto` pour un serveur partagé : auto-approbation des
        // permissions (les clients `attach` n'ont pas l'option `--auto`).
        var environment = ProcessInfo.processInfo.environment
        environment["OPENCODE_PERMISSION"] = #"{"*":"allow"}"#
        // Scopage de l'autopilot par groupe : le plugin autopilot lit ces variables
        // au chargement du serveur. Chaque groupe obtient son propre namespace
        // d'état, sa racine, son titre de superviseur et son filtre de workers.
        environment["OPENCODE_AUTOPILOT"] = group.autopilot ? "1" : "0"
        if group.autopilot {
            environment["OPENCODE_AUTOPILOT_ROOT"] = group.projectDir
            environment["OPENCODE_AUTOPILOT_NAMESPACE"] = "\(group.name)-\(group.port)"
            environment["OPENCODE_AUTOPILOT_SUPERVISOR_TITLE"] = group.paneMarker
            environment["OPENCODE_AUTOPILOT_WORKER_FILTER"] = group.name
            environment["OPENCODE_AUTOPILOT_AGENT"] = "super-orchestrator"
            environment["OPENCODE_AUTOPILOT_INTERVAL_MIN"] = String(max(1, group.autopilotIntervalMin))
        }
        // Pont Telegram : route les messages de l'utilisateur vers le superviseur
        // de CE groupe. Le token/chatId viennent de ~/.config/opencode/telegram/config.json.
        environment["TELEGRAM_ENABLED"] = "1"
        environment["TELEGRAM_SUPERVISOR_TITLE"] = group.paneMarker
        environment["TELEGRAM_SUPERVISOR_AGENT"] = group.autopilot ? "super-orchestrator" : group.orchestratorAgent
        process.environment = environment
        process.standardOutput = FileHandle.nullDevice
        process.standardError = FileHandle.nullDevice
        try process.run()
        servers[port] = process
    }

    private func resolveOpencodePath() -> String {
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
            let path = String(data: data, encoding: .utf8)?.trimmingCharacters(in: .whitespacesAndNewlines)
            if let path, !path.isEmpty { return path }
        } catch {}
        return "opencode"
    }

    private func waitForServer(api: OpenCodeAPI, timeout: TimeInterval) async -> Bool {
        let deadline = Date().addingTimeInterval(timeout)
        while Date() < deadline {
            if await api.ping() { return true }
            try? await Task.sleep(nanoseconds: 500_000_000)
        }
        return await api.ping()
    }

    private func attachCommand(group: LaunchGroup, sessionID: String, opencode: String, title: String) -> String {
        // `--dir` est indispensable : le serveur opencode scope les sessions par
        // dossier, et sans lui la TUI retombe sur le cwd du serveur.
        let base = "cd \(shellQuote(group.projectDir)) && \(shellQuote(opencode)) attach http://127.0.0.1:\(group.port) --dir \(shellQuote(group.projectDir)) -s \(sessionID)"
        let command = "\(base) || { echo; echo '⚠ Échec de opencode attach (serveur indisponible ou session invalide).'; echo 'Appuie sur Entrée pour fermer cette fenêtre.'; read -k1; }"
        // Fixe un titre de panneau distinct et persistant via OSC 0 (iTerm2 le conserve).
        let escapedTitle = title.replacingOccurrences(of: "'", with: "'\\''")
        return "printf '\\033]0;\(escapedTitle)\\007'; \(command)"
    }

    private func orchestratorPrompt(group: LaunchGroup, workers: [(title: String, id: String)]) -> String {
        let list = workers.map { "- \($0.title) : \($0.id)" }.joined(separator: "\n")
        return """
        Tu es l'orchestrateur du groupe « \(group.name) » (projet : \(group.projectDir)). Voici les fenêtres worker disponibles (titre : sessionID) :

        \(list)

        Consignes :
        - Pour déléguer une sous-tâche, utilise bridge_session_send avec le sessionID du worker ciblé et agent="\(group.workerAgent)".
        - Tu peux paralléliser avec bridge_broadcast (wait=false) puis récolter via bridge_gather.
        - Ne fais pas le travail toi-même, sauf pour inspecter ou débloquer.
        - Après récolte, fournis une synthèse : fait / restant / blocages.

        Note pratique : l'agent affiché dans cette fenêtre peut rester 'build' — ce n'est qu'un indicateur pour les messages tapés manuellement. Pour les prochains messages que l'utilisateur tapera ici, il peut appuyer sur Tab pour sélectionner 'orchestrator'. Tes actions via bridge_* fonctionnent déjà correctement.

        Réponds simplement que tu es prêt(e) et attends la première mission.
        """
    }

    private func supervisorPrompt(group: LaunchGroup, workers: [(title: String, id: String)]) -> String {
        let list = workers.map { "- \($0.title) : \($0.id)" }.joined(separator: "\n")
        return """
        Tu es le SUPERVISEUR AUTOPILOT du groupe « \(group.name) » (projet cible : \(group.projectDir)).

        Workers disponibles (titre : sessionID) :
        \(list)

        La boucle de fond de l'autopilot t'enverra régulièrement des messages « AUTOPILOT CYCLE ». Commence dès maintenant par UN cycle complet :
        1) auto_status ; 2) si le registre est vide → auto_projects_scan (racine = \(group.projectDir)) ; 3) auto_project_brief sur le projet ; 4) ajoute au backlog des tâches « sûres » (kinds autorisés) ; 5) auto_backlog_claim ; 6) dispatche chaque tâche à un worker idle via bridge_session_send (agent="\(group.workerAgent)", wait=true) ; 7) vérifie le rapport puis auto_task_complete ; 8) termine par un rapport de cycle concis.

        Règles : ne travaille QUE sur \(group.projectDir) ; jamais de git push ; ne fais pas le travail toi-même (délègue aux workers).
        """
    }

    private func shellQuote(_ value: String) -> String {
        "'" + value.replacingOccurrences(of: "'", with: "'\\''") + "'"
    }

    // MARK: - Persistance

    private func persist() {
        if let data = try? JSONEncoder().encode(groups) {
            UserDefaults.standard.set(data, forKey: Self.storageKey)
        }
    }
}
