//! Arbre de fichiers du workspace : types de nœuds et scan du système de
//! fichiers, indépendants de l'état applicatif.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Répertoires ignorés lors du scan.
const IGNORED_DIRS: [&str; 3] = [".git", "target", "node_modules"];

/// Nœud de l'arbre de fichiers du workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// Nom affiché.
    pub name: String,
    /// Chemin absolu.
    pub path: PathBuf,
    /// Vrai s'il s'agit d'un répertoire.
    pub is_dir: bool,
    /// Répertoire déplié.
    pub expanded: bool,
    /// Enfants (vide pour un fichier).
    pub children: Vec<Node>,
}

/// Ligne aplatie prête au rendu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Visible {
    /// Nom affiché.
    pub name: String,
    /// Chemin associé.
    pub path: PathBuf,
    /// Vrai s'il s'agit d'un répertoire.
    pub is_dir: bool,
    /// Profondeur d'indentation.
    pub depth: usize,
    /// Répertoire déplié.
    pub expanded: bool,
    /// Le répertoire contient au moins un enfant.
    pub has_children: bool,
}

/// Aplatit l'arbre en lignes visibles, en suivant l'état déplié des dossiers.
pub(crate) fn flatten(nodes: &[Node], level: usize, out: &mut Vec<Visible>) {
    for node in nodes {
        out.push(Visible {
            name: node.name.clone(),
            path: node.path.clone(),
            is_dir: node.is_dir,
            depth: level,
            expanded: node.expanded,
            has_children: !node.children.is_empty(),
        });
        if node.is_dir && node.expanded {
            flatten(&node.children, level + 1, out);
        }
    }
}

/// Déplie/replie le nœud au chemin `path`. Renvoie `true` s'il a été trouvé.
pub(crate) fn set_expanded_by_path(nodes: &mut [Node], path: &Path, expand: bool) -> bool {
    for node in nodes.iter_mut() {
        if node.path == path {
            if node.is_dir {
                node.expanded = expand;
            }
            return true;
        }
        if set_expanded_by_path(&mut node.children, path, expand) {
            return true;
        }
    }
    false
}

/// Scanne `dir` récursivement (profondeur `depth`, budget d'entrées `budget`),
/// en ignorant [`IGNORED_DIRS`]. Dossiers d'abord, puis par nom.
pub(crate) fn scan_dir(dir: &Path, depth: usize, budget: &mut usize) -> Vec<Node> {
    if depth == 0 || *budget == 0 {
        return Vec::new();
    }
    let Ok(read) = fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut entries: Vec<(bool, String, PathBuf)> = read
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            let name = entry.file_name().to_string_lossy().into_owned();
            if is_dir && IGNORED_DIRS.contains(&name.as_str()) {
                return None;
            }
            Some((is_dir, name, path))
        })
        .collect();
    entries.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));

    let mut nodes = Vec::new();
    for (is_dir, name, path) in entries {
        if *budget == 0 {
            break;
        }
        *budget -= 1;
        let children = if is_dir {
            scan_dir(&path, depth - 1, budget)
        } else {
            Vec::new()
        };
        nodes.push(Node {
            name,
            path,
            is_dir,
            expanded: false,
            children,
        });
    }
    nodes
}

/// Collecte les chemins des répertoires dépliés (récursif).
pub(crate) fn expanded_paths(node: &Node) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if node.is_dir && node.expanded {
        out.push(node.path.clone());
    }
    for child in &node.children {
        out.extend(expanded_paths(child));
    }
    out
}

/// Ré-applique l'état déplié à partir d'un jeu de chemins.
pub(crate) fn restore_expanded(nodes: &mut [Node], expanded: &HashSet<PathBuf>) {
    for node in nodes {
        if node.is_dir && expanded.contains(&node.path) {
            node.expanded = true;
        }
        restore_expanded(&mut node.children, expanded);
    }
}
