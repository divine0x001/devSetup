import SwiftUI

struct ContentView: View {
    @ObservedObject var model: LauncherModel
    @State private var pendingDelete: SessionInfo?
    @State private var pendingGlobalDelete: DBSession?

    var body: some View {
        HSplitView {
            sidebar
                .frame(minWidth: 260, idealWidth: 320, maxWidth: 420)
            detail
                .frame(minWidth: 520, maxWidth: .infinity)
        }
        .frame(minWidth: 1000, minHeight: 660)
        .onChange(of: model.selectedGroupID) { _ in
            Task { await model.refreshSessions(silent: true) }
        }
        .onChange(of: model.sessionSource) { source in
            Task {
                if source == .all { await model.refreshAllSessions(silent: true) }
                else { await model.refreshSessions(silent: true) }
            }
        }
        .task {
            await model.refreshAllSessions(silent: true)
            if model.sessionSource == .group { await model.refreshSessions(silent: true) }
        }
        .confirmationDialog(
            "Supprimer la session « \(pendingDelete?.displayTitle ?? "") » ?",
            isPresented: Binding(
                get: { pendingDelete != nil },
                set: { if !$0 { pendingDelete = nil } }
            )
        ) {
            Button("Supprimer", role: .destructive) {
                if let session = pendingDelete {
                    Task { await model.deleteSession(session) }
                }
                pendingDelete = nil
            }
            Button("Annuler", role: .cancel) { pendingDelete = nil }
        }
        .confirmationDialog(
            "Supprimer la session « \(pendingGlobalDelete?.displayTitle ?? "") » ?",
            isPresented: Binding(
                get: { pendingGlobalDelete != nil },
                set: { if !$0 { pendingGlobalDelete = nil } }
            )
        ) {
            Button("Supprimer", role: .destructive) {
                if let session = pendingGlobalDelete {
                    Task { await model.deleteGlobalSession(session) }
                }
                pendingGlobalDelete = nil
            }
            Button("Annuler", role: .cancel) { pendingGlobalDelete = nil }
        }
    }

    // MARK: - Barre latérale

    private var sidebar: some View {
        VStack(spacing: 0) {
            groupsSection
            Divider()
            sessionsSection
            Divider()
            footer
        }
    }

    private var groupsSection: some View {
        VStack(spacing: 0) {
            HStack {
                Text("Groupes").font(.headline)
                Spacer()
                Button { model.addGroup() } label: { Image(systemName: "plus") }
                    .help("Ajouter un groupe")
                Button { model.removeSelectedGroup() } label: { Image(systemName: "minus") }
                    .help("Supprimer le groupe sélectionné")
                    .disabled(model.selectedGroupID == nil)
            }
            .buttonStyle(.borderless)
            .padding(.horizontal, 10)
            .padding(.vertical, 8)

            List(selection: $model.selectedGroupID) {
                ForEach(model.groups) { group in
                    row(for: group).tag(group.id)
                }
            }
            .listStyle(.sidebar)
            .frame(height: 160)
        }
    }

    private var sessionsSection: some View {
        VStack(spacing: 0) {
            HStack {
                Text("Sessions").font(.headline)
                Picker("", selection: $model.sessionSource) {
                    ForEach(SessionSource.allCases) { source in
                        Text(source.rawValue).tag(source)
                    }
                }
                .labelsHidden()
                .pickerStyle(.segmented)
                .frame(width: 150)
                Spacer()
                if isLoadingSessions {
                    ProgressView().controlSize(.small)
                }
                Button {
                    Task {
                        if model.sessionSource == .all { await model.refreshAllSessions() }
                        else { await model.refreshSessions() }
                    }
                } label: {
                    Image(systemName: "arrow.clockwise")
                }
                .help("Rafraîchir")
            }
            .buttonStyle(.borderless)
            .padding(.horizontal, 10)
            .padding(.top, 8)
            .padding(.bottom, 4)

            if model.sessionSource == .all {
                TextField("Filtrer (titre, projet, agent…)", text: $model.sessionFilter)
                    .textFieldStyle(.roundedBorder)
                    .padding(.horizontal, 10)
                    .padding(.bottom, 6)
                allSessionsList
            } else {
                groupSessionsList
            }
        }
    }

    private var isLoadingSessions: Bool {
        model.sessionSource == .all ? model.allSessionsLoading : model.sessionsLoading
    }

    private var allSessionsList: some View {
        Group {
            if model.filteredAllSessions.isEmpty {
                emptyState(
                    title: model.allSessions.isEmpty ? "Aucune session" : "Aucun résultat",
                    subtitle: model.allSessions.isEmpty
                        ? "Base opencode vide ou introuvable."
                        : "Essaie un autre filtre."
                )
            } else {
                List(model.filteredAllSessions) { session in
                    allSessionRow(session)
                }
                .listStyle(.sidebar)
                .frame(maxHeight: .infinity)
            }
        }
    }

    private var groupSessionsList: some View {
        Group {
            if model.sessions.isEmpty {
                emptyState(
                    title: "Aucune session",
                    subtitle: model.selectedGroup
                        .map { "Serveur du groupe « \($0.name) » (:\($0.port))." }
                        ?? "Sélectionne un groupe."
                )
            } else {
                List(model.sessions) { session in
                    groupSessionRow(session)
                }
                .listStyle(.sidebar)
                .frame(maxHeight: .infinity)
            }
        }
    }

    private func emptyState(title: String, subtitle: String) -> some View {
        VStack(spacing: 6) {
            Text(title).foregroundStyle(.secondary)
            Text(subtitle)
                .font(.caption)
                .foregroundStyle(.tertiary)
                .multilineTextAlignment(.center)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .padding(10)
    }

    /// Ligne d'une session globale (base) : rouvrable sans serveur.
    private func allSessionRow(_ session: DBSession) -> some View {
        HStack(spacing: 8) {
            Image(systemName: "terminal")
                .font(.system(size: 10))
                .foregroundStyle(.secondary)
            VStack(alignment: .leading, spacing: 2) {
                Text(session.displayTitle).lineLimit(1)
                Text("\(session.projectName) · \(session.relativeUpdated)\(agentSuffix(session.agent))")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
            Spacer(minLength: 4)
            Button { model.reopen(session: session) } label: {
                Image(systemName: "arrow.up.forward.app")
            }
            .help("Rouvrir cette session dans le terminal")
            Button { pendingGlobalDelete = session } label: {
                Image(systemName: "trash")
            }
            .help("Supprimer la session")
        }
        .buttonStyle(.borderless)
        .padding(.vertical, 2)
    }

    private func agentSuffix(_ agent: String?) -> String {
        guard let agent, !agent.isEmpty else { return "" }
        return " · \(agent)"
    }

    private func groupSessionRow(_ session: SessionInfo) -> some View {
        HStack(spacing: 8) {
            Circle()
                .fill(model.status(for: session) == "busy" ? Color.green : Color.secondary.opacity(0.5))
                .frame(width: 7, height: 7)
            VStack(alignment: .leading, spacing: 2) {
                Text(session.displayTitle).lineLimit(1)
                Text("\(session.projectName) · \(session.relativeUpdated)")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
            Spacer(minLength: 4)
            Button { model.attach(session: session) } label: {
                Image(systemName: "arrow.up.forward.app")
            }
            .help("Rouvrir cette session dans le terminal")
            Button { pendingDelete = session } label: {
                Image(systemName: "trash")
            }
            .help("Supprimer la session")
        }
        .buttonStyle(.borderless)
        .padding(.vertical, 2)
    }

    private var footer: some View {
        HStack(spacing: 8) {
            Button {
                Task { await model.launchAll() }
            } label: {
                Label("Lancer tout", systemImage: "play.fill")
            }
            .disabled(model.busy)

            Button(role: .destructive) {
                model.stopAll()
            } label: {
                Label("Tout arrêter", systemImage: "stop.fill")
            }
            .disabled(model.runningGroupIDs.isEmpty && model.busyGroupIDs.isEmpty)

            if model.busy { ProgressView().controlSize(.small) }
        }
        .padding(10)
    }

    private func row(for group: LaunchGroup) -> some View {
        HStack(spacing: 8) {
            Circle()
                .fill(statusColor(for: group))
                .frame(width: 8, height: 8)
            VStack(alignment: .leading, spacing: 2) {
                Text(group.name)
                Text("\(group.agentCount) agents · :\(group.port)\(group.autopilot ? " · autopilot" : "")")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(.vertical, 2)
    }

    private func statusColor(for group: LaunchGroup) -> Color {
        if model.busyGroupIDs.contains(group.id) { return .orange }
        if model.runningGroupIDs.contains(group.id) { return .green }
        return Color.secondary.opacity(0.5)
    }

    // MARK: - Éditeur du groupe sélectionné

    @ViewBuilder
    private var detail: some View {
        if let index = model.selectedIndex {
            GroupEditor(group: $model.groups[index], model: model)
        } else {
            Text("Aucun groupe. Clique sur + pour en créer un.")
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
    }
}

private struct GroupEditor: View {
    @Binding var group: LaunchGroup
    @ObservedObject var model: LauncherModel

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            TextField("Nom du groupe", text: $group.name)
                .font(.title2).bold()
                .textFieldStyle(.plain)

            Grid(alignment: .leading, horizontalSpacing: 12, verticalSpacing: 10) {
                GridRow {
                    Text("Répertoire")
                    HStack {
                        TextField("Chemin du projet", text: $group.projectDir)
                            .textFieldStyle(.roundedBorder)
                        Button("Choisir…") { model.chooseDirectory(for: group.id) }
                    }
                }
                GridRow {
                    Text("Port")
                    TextField("4096", value: $group.port, format: .number)
                        .textFieldStyle(.roundedBorder)
                        .frame(width: 100)
                }
                GridRow {
                    Text("Workers")
                    Stepper(value: $group.workerCount, in: 1...12) {
                        Text("\(group.workerCount)")
                    }
                }
                GridRow {
                    Text("Terminal")
                    Picker("", selection: $group.terminalKind) {
                        ForEach(TerminalKind.available()) { kind in
                            Text(kind.label).tag(kind)
                        }
                    }
                    .labelsHidden()
                    .frame(width: 160)
                }
                GridRow {
                    Text("Agents")
                    HStack {
                        TextField("orchestrator", text: $group.orchestratorAgent)
                            .textFieldStyle(.roundedBorder).frame(width: 140)
                        Text("chef")
                        TextField("worker", text: $group.workerAgent)
                            .textFieldStyle(.roundedBorder).frame(width: 140)
                        Text("worker")
                    }
                }
                GridRow {
                    Text("Options")
                    HStack {
                        Toggle("Lancer le serveur", isOn: $group.startServer)
                        Toggle("Initialiser les workers", isOn: $group.seedWorkers)
                    }
                }
                GridRow {
                    Text("Autopilot")
                    HStack {
                        Toggle("Superviser automatiquement", isOn: $group.autopilot)
                        if group.autopilot {
                            Text("toutes les")
                            TextField("30", value: $group.autopilotIntervalMin, format: .number)
                                .textFieldStyle(.roundedBorder)
                                .frame(width: 60)
                            Text("min")
                        }
                    }
                }
            }

            HStack(spacing: 12) {
                Button {
                    Task {
                        await model.launch(groupID: group.id)
                        await model.refreshSessions(silent: true)
                        await model.refreshAllSessions(silent: true)
                    }
                } label: {
                    Label("Lancer ce groupe", systemImage: "play.fill")
                }
                .disabled(model.busyGroupIDs.contains(group.id))

                Button(role: .destructive) {
                    model.stop(groupID: group.id)
                } label: {
                    Label("Arrêter", systemImage: "stop.fill")
                }
                .disabled(!model.runningGroupIDs.contains(group.id))

                if model.busyGroupIDs.contains(group.id) {
                    ProgressView().controlSize(.small)
                }
            }

            Divider()

            Text("Journal").font(.headline)

            ScrollView {
                ScrollViewReader { proxy in
                    Text(model.logs.isEmpty ? "Prêt." : model.logs)
                        .font(.system(.body, design: .monospaced))
                        .textSelection(.enabled)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .id("log-end")
                        .onChange(of: model.logs) { _ in
                            proxy.scrollTo("log-end", anchor: .bottom)
                        }
                }
            }
            .frame(minHeight: 200)
            .padding(8)
            .background(Color(nsColor: .textBackgroundColor))
            .clipShape(RoundedRectangle(cornerRadius: 8))
        }
        .padding(20)
    }
}
