import Foundation
import AppKit

enum TerminalKind: String, CaseIterable, Identifiable, Codable {
    case iterm2 = "iTerm2"
    case ghostty = "Ghostty"
    case terminal = "Terminal"

    var id: String { rawValue }
    var label: String { rawValue }

    static func available() -> [TerminalKind] {
        var kinds: [TerminalKind] = []
        if FileManager.default.fileExists(atPath: "/Applications/iTerm.app") {
            kinds.append(.iterm2)
        }
        if FileManager.default.fileExists(atPath: "/Applications/Ghostty.app") {
            kinds.append(.ghostty)
        }
        kinds.append(.terminal)
        return kinds
    }
}

struct TerminalPanel {
    let title: String
    let command: String
}

enum TerminalLauncherError: Error, LocalizedError {
    case iterm2Missing
    case applescriptFailed(String)

    var errorDescription: String? {
        switch self {
        case .iterm2Missing:
            return "iTerm2 est introuvable (/Applications/iTerm.app). Installe-le via 'brew install --cask iterm2'."
        case .applescriptFailed(let message):
            return "Échec du pilotage d'iTerm2 : \(message)"
        }
    }
}

enum TerminalLauncher {
    /// Ouvre un terminal mono-panneau (comportement historique, conservé tel quel).
    static func open(command: String, kind: TerminalKind) {
        switch kind {
        case .iterm2:
            try? openITerm2(panels: [TerminalPanel(title: "OpenCode", command: command)])
        case .ghostty:
            openGhostty(command: command)
        case .terminal:
            openTerminal(command: command)
        }
    }

    /// Ouvre tous les agents dans une seule fenêtre disposée en grille.
    /// - iTerm2 : une fenêtre, N panneaux (grille 2×2 pour 4 agents).
    /// - Ghostty / Terminal : repli sur une fenêtre par agent (comportement d'origine).
    static func openGrid(panels: [TerminalPanel], kind: TerminalKind) throws {
        guard !panels.isEmpty else { return }
        switch kind {
        case .iterm2:
            try openITerm2(panels: panels)
        case .ghostty:
            for (index, panel) in panels.enumerated() {
                if index > 0 { Thread.sleep(forTimeInterval: 0.3) }
                openGhostty(command: panel.command)
            }
        case .terminal:
            for (index, panel) in panels.enumerated() {
                if index > 0 { Thread.sleep(forTimeInterval: 0.3) }
                openTerminal(command: panel.command)
            }
        }
    }

    // MARK: - iTerm2

    /// Vrai si une fenêtre iTerm2 contient déjà un panneau dont le titre
    /// commence par `marker` (grille déjà lancée pour ce groupe) — évite les
    /// doublons, groupe par groupe.
    static func iterm2GridAlreadyOpen(marker: String) -> Bool {
        guard FileManager.default.fileExists(atPath: "/Applications/iTerm.app"),
              NSWorkspace.shared.runningApplications.contains(where: {
                  $0.bundleIdentifier == "com.googlecode.iterm2" || $0.localizedName == "iTerm2"
              }) else {
            return false
        }
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/osascript")
        process.arguments = ["-e", iterm2AlreadyOpenScript, marker]
        let outPipe = Pipe()
        let errPipe = Pipe()
        process.standardOutput = outPipe
        process.standardError = errPipe
        do {
            try process.run()
            process.waitUntilExit()
        } catch {
            return false
        }
        let data = outPipe.fileHandleForReading.readDataToEndOfFile()
        return String(data: data, encoding: .utf8)?
            .trimmingCharacters(in: .whitespacesAndNewlines) == "true"
    }

    private static let iterm2AlreadyOpenScript = """
    on run argv
        set marker to (item 1 of argv)
        tell application "iTerm2"
            repeat with w in windows
                repeat with t in tabs of w
                    repeat with s in sessions of t
                        if (name of s) begins with marker then
                            return "true"
                        end if
                    end repeat
                end repeat
            end repeat
            return "false"
        end tell
    end run
    """

    private static func openITerm2(panels: [TerminalPanel]) throws {
        guard FileManager.default.fileExists(atPath: "/Applications/iTerm.app") else {
            throw TerminalLauncherError.iterm2Missing
        }

        let count = panels.count
        let cols = Int(ceil(Double(count).squareRoot()))
        let rows = Int(ceil(Double(count) / Double(cols)))

        var arguments: [String] = [String(cols), String(rows), String(count)]
        arguments.append(contentsOf: panels.map(\.command))
        arguments.append(contentsOf: panels.map(\.title))

        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/osascript")
        process.arguments = ["-e", iterm2GridScript] + arguments

        let stderrPipe = Pipe()
        process.standardError = stderrPipe
        process.standardOutput = FileHandle.nullDevice

        try process.run()
        process.waitUntilExit()

        if process.terminationStatus != 0 {
            let data = stderrPipe.fileHandleForReading.readDataToEndOfFile()
            let message = String(data: data, encoding: .utf8)?
                .trimmingCharacters(in: .whitespacesAndNewlines) ?? "code \(process.terminationStatus)"
            throw TerminalLauncherError.applescriptFailed(message)
        }
    }

    /// Génère dynamiquement une grille `cols × rows` :
    /// 1. la première ligne est construite par divisions verticales successives,
    /// 2. chaque colonne est ensuite subdivisée horizontalement pour créer les lignes.
    /// Les commandes et titres sont transmis via `argv` (aucun échappement manuel fragile).
    private static let iterm2GridScript = """
    on run argv
        set nCols to (item 1 of argv) as integer
        set nRows to (item 2 of argv) as integer
        set n to (item 3 of argv) as integer
        set cells to {}
        tell application "iTerm2"
            activate
            set newWin to (create window with default profile)
            set baseSess to (current session of newWin)
            set end of cells to baseSess
            repeat with j from 1 to (nCols - 1)
                set prevCell to item -1 of cells
                tell prevCell
                    set newCell to (split vertically with default profile)
                end tell
                set end of cells to newCell
            end repeat
            repeat with i from 1 to (nRows - 1)
                repeat with j from 0 to (nCols - 1)
                    set idx to i * nCols + j + 1
                    if idx is less than or equal to n then
                        set parentIdx to (i - 1) * nCols + j + 1
                        set parentCell to item parentIdx of cells
                        tell parentCell
                            set newCell to (split horizontally with default profile)
                        end tell
                        set end of cells to newCell
                    end if
                end repeat
            end repeat
            repeat with k from 1 to n
                set theCell to item k of cells
                set theCmd to item (k + 3) of argv
                set theTitle to item (k + 3 + n) of argv
                tell theCell
                    set name to theTitle
                    write text theCmd
                end tell
            end repeat
            select baseSess
        end tell
    end run
    """

    // MARK: - Ghostty / Terminal (comportement historique préservé)

    private static func openGhostty(command: String) {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/open")
        process.arguments = ["-na", "Ghostty.app", "--args", "-e", "/bin/zsh", "-lc", command]
        try? process.run()
    }

    private static func openTerminal(command: String) {
        let escaped = command
            .replacingOccurrences(of: "\\", with: "\\\\")
            .replacingOccurrences(of: "\"", with: "\\\"")
        let script = "tell application \"Terminal\"\nactivate\ndo script \"\(escaped)\"\nend tell"
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/osascript")
        process.arguments = ["-e", script]
        try? process.run()
    }
}
