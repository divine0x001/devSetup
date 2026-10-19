//! Éditeur de texte : tampon + curseur, colonnes en `char` (UTF-8 sûr).

use std::fs;
use std::path::{Path, PathBuf};

use devx_core::DevxError;

/// Profondeur maximale de l'historique d'annulation (borne la mémoire).
const UNDO_DEPTH: usize = 200;

/// Type de la dernière édition, pour regrouper (« coalescer ») des frappes
/// successives de même nature en une seule étape d'annulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditKind {
    /// Insertion de texte au fil de la frappe.
    Insert,
    /// Suppression de caractères (Backspace/Delete).
    Delete,
    /// Édition structurelle : rompt toujours le regroupement.
    Other,
}

/// Instantané de l'état éditable, empilé pour annuler/rétablir.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Snapshot {
    lines: Vec<String>,
    cursor_row: usize,
    cursor_col: usize,
}

/// Tampon éditable.
///
/// Les colonnes du curseur sont des indices **de caractères** (`char`), jamais
/// des indices d'octets : l'édition est sûre vis-à-vis de l'UTF-8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Buffer {
    /// Contenu découpé en lignes (jamais vide : au moins une chaîne).
    pub lines: Vec<String>,
    /// Ligne du curseur.
    pub cursor_row: usize,
    /// Colonne du curseur (indice de `char` dans la ligne).
    pub cursor_col: usize,
    /// Fichier associé, le cas échéant.
    pub path: Option<PathBuf>,
    /// Vrai si le tampon diffère de l'état sur disque.
    pub dirty: bool,
    /// Pile d'annulation (états antérieurs).
    undo: Vec<Snapshot>,
    /// Pile de rétablissement (états annulés).
    redo: Vec<Snapshot>,
    /// Nature de la dernière édition, pour le regroupement.
    last_edit: EditKind,
    /// Contenu de référence sur disque (dernier chargement/sauvegarde) : sert
    /// à calculer `dirty` exactement, y compris après annulation.
    clean: Vec<String>,
    /// Ancre de sélection (position fixe) ; la sélection court jusqu'au
    /// curseur. `None` : aucune sélection active.
    selection_anchor: Option<(usize, usize)>,
}

impl Buffer {
    /// Construit un tampon neuf (historique vide, curseur en tête).
    fn fresh(lines: Vec<String>, path: Option<PathBuf>) -> Self {
        Self {
            clean: lines.clone(),
            lines,
            cursor_row: 0,
            cursor_col: 0,
            path,
            dirty: false,
            undo: Vec::new(),
            redo: Vec::new(),
            last_edit: EditKind::Other,
            selection_anchor: None,
        }
    }

    /// Recalcule `dirty` : vrai si le contenu diffère de la référence disque.
    fn mark_dirty(&mut self) {
        self.dirty = self.lines != self.clean;
    }

    /// Démarre une sélection à la position du curseur si aucune n'est active
    /// (appelé avant un déplacement avec Maj).
    pub fn anchor_selection(&mut self) {
        if self.selection_anchor.is_none() {
            self.selection_anchor = Some((self.cursor_row, self.cursor_col));
        }
    }

    /// Annule la sélection courante.
    pub fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    /// Sélectionne tout le tampon (ancre au début, curseur à la fin).
    pub fn select_all(&mut self) {
        self.selection_anchor = Some((0, 0));
        self.cursor_row = self.lines.len() - 1;
        self.cursor_col = self.line_len(self.cursor_row);
    }

    /// Bornes normalisées `(début, fin)` de la sélection, ou `None` si vide.
    pub fn selection_range(&self) -> Option<((usize, usize), (usize, usize))> {
        let anchor = self.selection_anchor?;
        let cursor = (self.cursor_row, self.cursor_col);
        if anchor == cursor {
            return None;
        }
        Some(if anchor <= cursor {
            (anchor, cursor)
        } else {
            (cursor, anchor)
        })
    }

    /// Texte sélectionné (lignes jointes par `\n`), ou `None` si vide.
    pub fn selected_text(&self) -> Option<String> {
        let ((sr, sc), (er, ec)) = self.selection_range()?;
        if sr == er {
            let chars: Vec<char> = self.lines[sr].chars().collect();
            return Some(chars[sc..ec].iter().collect());
        }
        let mut out = String::new();
        let first: Vec<char> = self.lines[sr].chars().collect();
        out.extend(&first[sc..]);
        for row in &self.lines[sr + 1..er] {
            out.push('\n');
            out.push_str(row);
        }
        out.push('\n');
        let last: Vec<char> = self.lines[er].chars().collect();
        out.extend(&last[..ec]);
        Some(out)
    }

    /// Supprime la sélection si elle existe ; place le curseur à son début.
    /// Renvoie `true` si une sélection a été supprimée.
    pub fn delete_selection(&mut self) -> bool {
        let Some(((sr, sc), (er, ec))) = self.selection_range() else {
            return false;
        };
        self.record(EditKind::Other);
        let left: String = self.lines[sr].chars().take(sc).collect();
        let right: String = self.lines[er].chars().skip(ec).collect();
        let merged = format!("{left}{right}");
        self.lines.splice(sr..=er, std::iter::once(merged));
        self.cursor_row = sr;
        self.cursor_col = sc;
        self.selection_anchor = None;
        self.mark_dirty();
        true
    }

    /// Insère `text` (éventuellement multi-ligne) à la position du curseur, qui
    /// se place après le texte inséré. Annule d'abord toute sélection active.
    pub fn insert_str(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        self.delete_selection();
        self.record(EditKind::Other);
        let right = self.split_at_cursor();
        let segments: Vec<&str> = text.split('\n').collect();
        // Première ligne : collée à la suite du curseur.
        self.lines[self.cursor_row].push_str(segments[0]);
        self.cursor_col = self.line_len(self.cursor_row);
        // Lignes suivantes insérées en dessous.
        for seg in &segments[1..] {
            self.cursor_row += 1;
            self.lines.insert(self.cursor_row, (*seg).to_string());
            self.cursor_col = self.line_len(self.cursor_row);
        }
        // Ré-accole la partie droite coupée au départ.
        let row = self.cursor_row;
        self.lines[row].push_str(&right);
        self.mark_dirty();
    }

    /// Tampon vide, sans fichier associé.
    pub fn empty() -> Self {
        Self::fresh(vec![String::new()], None)
    }

    /// Tampon vide associé à un chemin (pour créer un nouveau fichier).
    pub fn empty_with_path(path: impl Into<PathBuf>) -> Self {
        Self::fresh(vec![String::new()], Some(path.into()))
    }

    /// Charge un fichier dans un nouveau tampon.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, DevxError> {
        let path = path.as_ref().to_path_buf();
        let raw = fs::read_to_string(&path)
            .map_err(|err| DevxError::io(format!("{}: {err}", path.display())))?;
        let mut lines: Vec<String> = raw.lines().map(str::to_string).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }
        Ok(Self::fresh(lines, Some(path)))
    }

    /// Sauvegarde le tampon sur disque (échec si aucun fichier associé).
    pub fn save(&mut self) -> Result<(), DevxError> {
        let Some(path) = &self.path else {
            return Err(DevxError::message(
                "aucun fichier associé (:e <fichier> d'abord)",
            ));
        };
        let mut content = self.lines.join("\n");
        content.push('\n');
        fs::write(path, content)
            .map_err(|err| DevxError::io(format!("écriture de {}: {err}", path.display())))?;
        self.clean = self.lines.clone();
        self.dirty = false;
        // La prochaine édition ouvre une nouvelle étape d'annulation.
        self.last_edit = EditKind::Other;
        Ok(())
    }

    /// Enregistre l'état courant avant une édition de type `kind`, et vide la
    /// pile de rétablissement. Les éditions consécutives de même type (hors
    /// [`EditKind::Other`]) sont regroupées en une seule étape d'annulation.
    fn record(&mut self, kind: EditKind) {
        self.redo.clear();
        let coalesce = kind != EditKind::Other && kind == self.last_edit;
        self.last_edit = kind;
        if coalesce {
            return;
        }
        self.undo.push(self.snapshot());
        if self.undo.len() > UNDO_DEPTH {
            self.undo.remove(0);
        }
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            lines: self.lines.clone(),
            cursor_row: self.cursor_row,
            cursor_col: self.cursor_col,
        }
    }

    fn restore(&mut self, snap: Snapshot) {
        self.lines = snap.lines;
        self.cursor_row = snap.cursor_row;
        self.cursor_col = snap.cursor_col;
    }

    /// Annule la dernière édition. Renvoie `false` s'il n'y a rien à annuler.
    pub fn undo(&mut self) -> bool {
        let Some(prev) = self.undo.pop() else {
            return false;
        };
        self.redo.push(self.snapshot());
        self.restore(prev);
        self.last_edit = EditKind::Other;
        self.mark_dirty();
        true
    }

    /// Rétablit la dernière édition annulée. Renvoie `false` s'il n'y a rien.
    pub fn redo(&mut self) -> bool {
        let Some(next) = self.redo.pop() else {
            return false;
        };
        self.undo.push(self.snapshot());
        self.restore(next);
        self.last_edit = EditKind::Other;
        self.mark_dirty();
        true
    }

    /// Position du curseur en coordonnées affichables (ligne:colonne 1-based).
    pub fn cursor_position(&self) -> (usize, usize) {
        (self.cursor_row + 1, self.cursor_col + 1)
    }

    /// Nom court affichable du tampon : nom de fichier, ou « sans-titre » pour
    /// un tampon sans chemin associé.
    pub fn display_name(&self) -> String {
        self.path
            .as_deref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "sans-titre".to_string())
    }

    /// Insère un caractère à la position du curseur.
    pub fn insert_char(&mut self, c: char) {
        self.record(EditKind::Insert);
        let line = &mut self.lines[self.cursor_row];
        let mut chars: Vec<char> = line.chars().collect();
        chars.insert(self.cursor_col, c);
        *line = chars.into_iter().collect();
        self.cursor_col += 1;
        self.mark_dirty();
    }

    /// Coupe la ligne courante à la colonne du curseur et insère une nouvelle
    /// ligne avec la partie droite.
    pub fn insert_newline(&mut self) {
        self.record(EditKind::Other);
        let right = self.split_at_cursor();
        self.lines.insert(self.cursor_row + 1, right);
        self.cursor_row += 1;
        self.cursor_col = 0;
        self.mark_dirty();
    }

    /// Efface le caractère précédant le curseur ; en début de ligne, fusionne
    /// avec la ligne précédente.
    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            self.record(EditKind::Delete);
            let line = &mut self.lines[self.cursor_row];
            let mut chars: Vec<char> = line.chars().collect();
            chars.remove(self.cursor_col - 1);
            *line = chars.into_iter().collect();
            self.cursor_col -= 1;
            self.mark_dirty();
        } else if self.cursor_row > 0 {
            self.record(EditKind::Delete);
            let current = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            let merged = format!("{}{}", self.lines[self.cursor_row], current);
            self.lines[self.cursor_row] = merged;
            self.cursor_col = self.line_len(self.cursor_row);
            self.mark_dirty();
        }
    }

    /// Efface le caractère sous le curseur ; en fin de ligne, fusionne avec la
    /// ligne suivante.
    pub fn delete(&mut self) {
        let len = self.line_len(self.cursor_row);
        if self.cursor_col < len {
            self.record(EditKind::Delete);
            let line = &mut self.lines[self.cursor_row];
            let mut chars: Vec<char> = line.chars().collect();
            chars.remove(self.cursor_col);
            *line = chars.into_iter().collect();
            self.mark_dirty();
        } else if self.cursor_row + 1 < self.lines.len() {
            self.record(EditKind::Delete);
            let next = self.lines.remove(self.cursor_row + 1);
            self.lines[self.cursor_row].push_str(&next);
            self.mark_dirty();
        }
    }

    /// Déplace le curseur d'une ligne vers le haut.
    pub fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.clamp_col();
        }
    }

    /// Déplace le curseur d'une ligne vers le bas.
    pub fn move_down(&mut self) {
        let last = self.lines.len() - 1;
        if self.cursor_row < last {
            self.cursor_row += 1;
            self.clamp_col();
        }
    }

    /// Déplace le curseur d'un caractère vers la gauche.
    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.line_len(self.cursor_row);
        }
    }

    /// Déplace le curseur d'un caractère vers la droite.
    pub fn move_right(&mut self) {
        let last = self.lines.len() - 1;
        if self.cursor_col < self.line_len(self.cursor_row) {
            self.cursor_col += 1;
        } else if self.cursor_row < last {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    /// Home « intelligent » : place le curseur au premier caractère non blanc
    /// de la ligne ; s'il y est déjà, revient en colonne 0.
    pub fn move_home(&mut self) {
        let first_non_ws = self.lines[self.cursor_row]
            .chars()
            .position(|c| !c.is_whitespace())
            .unwrap_or(0);
        self.cursor_col = if self.cursor_col == first_non_ws {
            0
        } else {
            first_non_ws
        };
    }

    /// Remonte le curseur de `n` lignes (au moins une), borné au début.
    pub fn page_up(&mut self, n: usize) {
        self.cursor_row = self.cursor_row.saturating_sub(n.max(1));
        self.clamp_col();
    }

    /// Descend le curseur de `n` lignes (au moins une), borné à la fin.
    pub fn page_down(&mut self, n: usize) {
        let last = self.lines.len() - 1;
        self.cursor_row = (self.cursor_row + n.max(1)).min(last);
        self.clamp_col();
    }

    /// Place le curseur au tout début du document.
    pub fn move_doc_start(&mut self) {
        self.cursor_row = 0;
        self.cursor_col = 0;
    }

    /// Place le curseur à la toute fin du document.
    pub fn move_doc_end(&mut self) {
        self.cursor_row = self.lines.len() - 1;
        self.cursor_col = self.line_len(self.cursor_row);
    }

    /// Échange la ligne courante avec celle du dessus ; le curseur suit.
    pub fn move_line_up(&mut self) {
        if self.cursor_row == 0 {
            return;
        }
        self.record(EditKind::Other);
        self.lines.swap(self.cursor_row, self.cursor_row - 1);
        self.cursor_row -= 1;
        self.clamp_col();
        self.mark_dirty();
    }

    /// Échange la ligne courante avec celle du dessous ; le curseur suit.
    pub fn move_line_down(&mut self) {
        if self.cursor_row + 1 >= self.lines.len() {
            return;
        }
        self.record(EditKind::Other);
        self.lines.swap(self.cursor_row, self.cursor_row + 1);
        self.cursor_row += 1;
        self.clamp_col();
        self.mark_dirty();
    }

    /// Duplique la ligne courante juste en dessous ; le curseur descend sur la
    /// copie (une seule étape d'annulation).
    pub fn duplicate_line(&mut self) {
        self.record(EditKind::Other);
        let copy = self.lines[self.cursor_row].clone();
        self.lines.insert(self.cursor_row + 1, copy);
        self.cursor_row += 1;
        self.mark_dirty();
    }

    /// Supprime la ligne courante. Un tampon à une seule ligne devient une
    /// ligne vide (jamais zéro ligne).
    pub fn delete_line(&mut self) {
        self.record(EditKind::Other);
        if self.lines.len() == 1 {
            self.lines[0].clear();
            self.cursor_col = 0;
        } else {
            self.lines.remove(self.cursor_row);
            if self.cursor_row >= self.lines.len() {
                self.cursor_row = self.lines.len() - 1;
            }
            self.clamp_col();
        }
        self.mark_dirty();
    }

    /// Recule le curseur au début du mot précédent (saute les espaces puis le
    /// mot). En début de ligne, passe à la fin de la ligne précédente.
    pub fn move_word_left(&mut self) {
        if self.cursor_col == 0 {
            self.move_left();
            return;
        }
        let chars: Vec<char> = self.lines[self.cursor_row].chars().collect();
        self.cursor_col = word_start_before(&chars, self.cursor_col);
    }

    /// Avance le curseur au début du mot suivant (saute le mot courant puis les
    /// espaces). En fin de ligne, passe au début de la ligne suivante.
    pub fn move_word_right(&mut self) {
        let chars: Vec<char> = self.lines[self.cursor_row].chars().collect();
        if self.cursor_col >= chars.len() {
            self.move_right();
            return;
        }
        let mut i = self.cursor_col;
        while i < chars.len() && !chars[i].is_whitespace() {
            i += 1;
        }
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        self.cursor_col = i;
    }

    /// Supprime le mot précédant le curseur (Ctrl-W). En début de ligne, se
    /// comporte comme un [`Buffer::backspace`] (fusion avec la ligne du dessus).
    pub fn delete_word_left(&mut self) {
        if self.cursor_col == 0 {
            self.backspace();
            return;
        }
        let chars: Vec<char> = self.lines[self.cursor_row].chars().collect();
        let start = word_start_before(&chars, self.cursor_col);
        if start == self.cursor_col {
            return;
        }
        // Chaque Ctrl-W est une étape d'annulation distincte.
        self.record(EditKind::Other);
        let mut chars = chars;
        chars.drain(start..self.cursor_col);
        self.lines[self.cursor_row] = chars.into_iter().collect();
        self.cursor_col = start;
        self.mark_dirty();
    }

    /// Cherche la prochaine occurrence de `query` à partir de
    /// `(from_row, from_col)` (incluse), en bouclant sur le document. Renvoie
    /// la position `(row, col)` en indices de `char`, ou `None` si `query` est
    /// vide ou introuvable. La recherche est sensible à la casse.
    pub fn find_from(
        &self,
        query: &str,
        from_row: usize,
        from_col: usize,
    ) -> Option<(usize, usize)> {
        if query.is_empty() {
            return None;
        }
        let n = self.lines.len();
        for i in 0..=n {
            let row = (from_row + i) % n;
            let start = if i == 0 { from_col } else { 0 };
            let hay: Vec<char> = self.lines[row].chars().collect();
            if let Some(col) = find_in_chars(&hay, query, start) {
                return Some((row, col));
            }
        }
        None
    }

    /// Indices (en `char`) de départ de chaque occurrence non chevauchante de
    /// `query` sur la ligne `row`. Vide si `query` est vide ou `row` hors borne.
    pub fn line_match_starts(&self, row: usize, query: &str) -> Vec<usize> {
        if query.is_empty() || row >= self.lines.len() {
            return Vec::new();
        }
        let hay: Vec<char> = self.lines[row].chars().collect();
        let width = query.chars().count();
        let mut starts = Vec::new();
        let mut from = 0;
        while let Some(i) = find_in_chars(&hay, query, from) {
            starts.push(i);
            from = i + width;
        }
        starts
    }

    /// Insère `width` espaces à la position du curseur (touche Tab).
    pub fn insert_tab(&mut self, width: usize) {
        if width == 0 {
            return;
        }
        self.record(EditKind::Insert);
        let line = &mut self.lines[self.cursor_row];
        let mut chars: Vec<char> = line.chars().collect();
        for _ in 0..width {
            chars.insert(self.cursor_col, ' ');
        }
        *line = chars.into_iter().collect();
        self.cursor_col += width;
        self.mark_dirty();
    }

    /// Supprime jusqu'à `width` espaces immédiatement avant le curseur
    /// (Shift-Tab : désindenter). Sans effet en début de ligne ou si le
    /// caractère précédent n'est pas un espace.
    pub fn remove_tab(&mut self, width: usize) {
        if width == 0 || self.cursor_col == 0 {
            return;
        }
        let before: Vec<char> = self.lines[self.cursor_row]
            .chars()
            .take(self.cursor_col)
            .collect();
        let run = before.iter().rev().take_while(|c| **c == ' ').count();
        let n = run.min(width);
        if n == 0 {
            return;
        }
        self.record(EditKind::Other);
        let line = &mut self.lines[self.cursor_row];
        let mut chars: Vec<char> = line.chars().collect();
        chars.drain(self.cursor_col - n..self.cursor_col);
        *line = chars.into_iter().collect();
        self.cursor_col -= n;
        self.mark_dirty();
    }

    /// Place le curseur en fin de ligne.
    pub fn move_end(&mut self) {
        self.cursor_col = self.line_len(self.cursor_row);
    }

    /// Coupe la ligne courante à la colonne du curseur ; le curseur garde la
    /// partie gauche, on renvoie la partie droite.
    fn split_at_cursor(&mut self) -> String {
        let chars: Vec<char> = self.lines[self.cursor_row].chars().collect();
        let left: String = chars[..self.cursor_col].iter().collect();
        let right: String = chars[self.cursor_col..].iter().collect();
        self.lines[self.cursor_row] = left;
        right
    }

    fn line_len(&self, row: usize) -> usize {
        self.lines[row].chars().count()
    }

    fn clamp_col(&mut self) {
        self.cursor_col = self.cursor_col.min(self.line_len(self.cursor_row));
    }
}

/// Début du mot situé juste avant `col` dans `chars` : on recule d'abord sur
/// les espaces, puis sur les caractères non blancs. Renvoie l'indice obtenu.
fn word_start_before(chars: &[char], col: usize) -> usize {
    let mut i = col.min(chars.len());
    while i > 0 && chars[i - 1].is_whitespace() {
        i -= 1;
    }
    while i > 0 && !chars[i - 1].is_whitespace() {
        i -= 1;
    }
    i
}

/// Indice du premier `needle` dans `hay` (slices de `char`) à partir de
/// `start`, ou `None`. `needle` vide ou plus long que `hay` renvoie `None`.
fn find_in_chars(hay: &[char], needle: &str, start: usize) -> Option<usize> {
    let needle: Vec<char> = needle.chars().collect();
    if needle.is_empty() || needle.len() > hay.len() {
        return None;
    }
    let last = hay.len() - needle.len();
    (start..=last).find(|&i| hay[i..i + needle.len()] == needle[..])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn empty_buffer_has_one_line() {
        let buffer = Buffer::empty();
        assert_eq!(buffer.lines, vec![String::new()]);
        assert_eq!(buffer.cursor_row, 0);
        assert_eq!(buffer.cursor_col, 0);
        assert!(!buffer.dirty);
        assert!(buffer.path.is_none());
    }

    #[test]
    fn insert_char_at_cursor() {
        let mut b = Buffer::empty();
        b.insert_char('a');
        b.insert_char('b');
        b.move_left();
        b.insert_char('X');
        assert_eq!(b.lines[0], "aXb");
        assert_eq!(b.cursor_col, 2);
        assert!(b.dirty);
    }

    #[test]
    fn insert_newline_splits_line() {
        let mut b = Buffer::empty();
        for c in "ab".chars() {
            b.insert_char(c);
        }
        b.move_left();
        b.insert_newline();
        assert_eq!(b.lines, vec!["a", "b"]);
        assert_eq!((b.cursor_row, b.cursor_col), (1, 0));
    }

    #[test]
    fn backspace_mid_line_and_merge() {
        let mut b = Buffer::empty();
        for c in "abcd".chars() {
            b.insert_char(c);
        }
        b.move_left();
        b.backspace();
        assert_eq!(b.lines[0], "abd");

        // En début de ligne : fusion avec la précédente.
        let mut b = Buffer::empty();
        b.insert_char('x');
        b.insert_newline();
        b.insert_char('y');
        b.move_home();
        b.backspace();
        assert_eq!(b.lines, vec!["xy"]);
        assert_eq!(b.cursor_row, 0);
    }

    #[test]
    fn delete_char_and_merge_next() {
        let mut b = Buffer::empty();
        for c in "abcd".chars() {
            b.insert_char(c);
        }
        b.move_home();
        b.delete();
        assert_eq!(b.lines[0], "bcd");

        // En fin de ligne : fusion avec la suivante.
        let mut b = Buffer::empty();
        b.insert_char('a');
        b.insert_newline();
        b.insert_char('b');
        b.move_up();
        b.move_end();
        b.delete();
        assert_eq!(b.lines, vec!["ab"]);
    }

    #[test]
    fn cursor_movements_respect_bounds() {
        let mut b = Buffer::empty();
        for c in "hello".chars() {
            b.insert_char(c);
        }
        b.insert_newline();
        b.insert_char('w');
        b.move_end();
        assert_eq!((b.cursor_row, b.cursor_col), (1, 1));

        // En bas : déjà sur la dernière ligne.
        b.move_down();
        assert_eq!((b.cursor_row, b.cursor_col), (1, 1));

        // En haut : colonne bornée à la longueur de la ligne.
        b.move_up();
        b.move_up();
        assert_eq!(b.cursor_row, 0);
        assert_eq!(b.cursor_col, 1);

        // Droite au-delà de la fin de ligne : passe à la ligne suivante.
        b.move_end();
        b.move_right();
        assert_eq!((b.cursor_row, b.cursor_col), (1, 0));
        b.move_right();
        assert_eq!((b.cursor_row, b.cursor_col), (1, 1));
        b.move_right();
        assert_eq!((b.cursor_row, b.cursor_col), (1, 1));
    }

    #[test]
    fn moving_down_clamps_column_to_line_length() {
        let mut b = Buffer::empty();
        for c in "very long line".chars() {
            b.insert_char(c);
        }
        b.insert_newline();
        b.insert_char('s');
        b.move_up();
        b.move_end();
        b.move_down();
        assert_eq!(b.cursor_row, 1);
        assert_eq!(b.cursor_col, 1);
    }

    #[test]
    fn home_and_end() {
        let mut b = Buffer::empty();
        for c in "abc".chars() {
            b.insert_char(c);
        }
        b.move_home();
        assert_eq!(b.cursor_col, 0);
        b.move_end();
        assert_eq!(b.cursor_col, 3);
    }

    #[test]
    fn smart_home_toggles_between_indent_and_column_zero() {
        let mut b = Buffer::empty();
        for c in "    code".chars() {
            b.insert_char(c);
        }
        b.move_end(); // col 8
        b.move_home(); // → premier non-blanc (col 4)
        assert_eq!(b.cursor_col, 4);
        b.move_home(); // déjà au non-blanc → col 0
        assert_eq!(b.cursor_col, 0);
        b.move_home(); // col 0 != premier non-blanc → revient au non-blanc
        assert_eq!(b.cursor_col, 4);
    }

    #[test]
    fn utf8_columns_are_char_based() {
        let mut b = Buffer::empty();
        for c in "héllo 🙂".chars() {
            b.insert_char(c);
        }
        b.move_home();
        b.insert_char('X');
        assert_eq!(b.lines[0], "Xhéllo 🙂");
        // Colonne en caractères (pas d'octets).
        b.move_end();
        assert_eq!(b.cursor_col, "Xhéllo 🙂".chars().count());
        b.backspace();
        b.move_end();
        assert_eq!(b.cursor_col, "Xhéllo ".chars().count());
    }

    #[test]
    fn load_and_save_round_trip() {
        let dir = env::temp_dir();
        let path = dir.join(format!("devx-tui-test-{}.txt", std::process::id()));
        fs::write(&path, "ligne une\nligne deux\n").unwrap();

        let mut b = Buffer::load(&path).unwrap();
        assert_eq!(b.lines, vec!["ligne une", "ligne deux"]);
        assert!(!b.dirty);

        b.lines[1].push('!');
        b.dirty = true;
        b.save().unwrap();
        assert!(!b.dirty);

        let reloaded = fs::read_to_string(&path).unwrap();
        assert_eq!(reloaded, "ligne une\nligne deux!\n");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn load_missing_file_errors() {
        let missing = env::temp_dir().join("devx-tui-definitely-missing.txt");
        assert!(Buffer::load(missing).is_err());
    }

    #[test]
    fn save_without_path_errors() {
        let mut b = Buffer::empty();
        b.dirty = true;
        assert!(b.save().is_err());
    }

    #[test]
    fn insert_tab_writes_spaces_and_really_moves_cursor() {
        let mut b = Buffer::empty();
        b.insert_char('a');
        b.insert_tab(4);
        assert_eq!(b.lines[0], "a    ");
        assert_eq!(b.cursor_col, 5);
        assert!(b.dirty);
    }

    #[test]
    fn remove_tab_deletes_consecutive_spaces() {
        let mut b = Buffer::empty();
        for c in "  ab".chars() {
            b.insert_char(c);
        }
        // Curseur posé après les 2 espaces (sur 'a') : Shift-Tab les retire.
        b.cursor_col = 2;
        b.remove_tab(4);
        assert_eq!(b.lines[0], "ab");
        assert_eq!(b.cursor_col, 0);

        // Début de ligne : sans effet.
        b.remove_tab(4);
        assert_eq!(b.lines[0], "ab");
        assert_eq!(b.cursor_col, 0);
        assert!(b.dirty);
    }

    #[test]
    fn remove_tab_stops_at_non_space() {
        let mut b = Buffer::empty();
        for c in " ab".chars() {
            b.insert_char(c);
        }
        b.move_end();
        b.remove_tab(4);
        assert_eq!(b.lines[0], " ab");
        assert_eq!(b.cursor_col, 3);
    }

    // --- annuler / rétablir ---

    #[test]
    fn undo_coalesces_a_run_of_inserts() {
        let mut b = Buffer::empty();
        for c in "abc".chars() {
            b.insert_char(c);
        }
        assert_eq!(b.lines[0], "abc");
        // Un seul undo annule toute la saisie contiguë.
        assert!(b.undo());
        assert_eq!(b.lines[0], "");
        assert_eq!(b.cursor_col, 0);
        // Plus rien à annuler.
        assert!(!b.undo());
    }

    #[test]
    fn newline_breaks_the_undo_run() {
        let mut b = Buffer::empty();
        for c in "ab".chars() {
            b.insert_char(c);
        }
        b.insert_newline();
        for c in "cd".chars() {
            b.insert_char(c);
        }
        assert_eq!(b.lines, vec!["ab", "cd"]);
        // 1er undo : retire "cd".
        b.undo();
        assert_eq!(b.lines, vec!["ab", ""]);
        // 2e undo : retire le saut de ligne.
        b.undo();
        assert_eq!(b.lines, vec!["ab"]);
        // 3e undo : retire "ab".
        b.undo();
        assert_eq!(b.lines, vec![""]);
    }

    #[test]
    fn redo_reapplies_undone_edits() {
        let mut b = Buffer::empty();
        for c in "hi".chars() {
            b.insert_char(c);
        }
        b.undo();
        assert_eq!(b.lines[0], "");
        assert!(b.redo());
        assert_eq!(b.lines[0], "hi");
        assert_eq!(b.cursor_col, 2);
        assert!(!b.redo());
    }

    #[test]
    fn a_fresh_edit_clears_the_redo_stack() {
        let mut b = Buffer::empty();
        b.insert_char('a');
        b.undo();
        // Une nouvelle édition après un undo invalide le rétablissement.
        b.insert_char('z');
        assert!(!b.redo());
        assert_eq!(b.lines[0], "z");
    }

    #[test]
    fn undo_separates_insert_from_delete() {
        let mut b = Buffer::empty();
        for c in "abc".chars() {
            b.insert_char(c);
        }
        b.backspace(); // supprime 'c'
        assert_eq!(b.lines[0], "ab");
        // 1er undo : rétablit la suppression.
        b.undo();
        assert_eq!(b.lines[0], "abc");
        // 2e undo : annule l'insertion.
        b.undo();
        assert_eq!(b.lines[0], "");
    }

    #[test]
    fn undo_on_empty_history_is_a_noop() {
        let mut b = Buffer::empty();
        assert!(!b.undo());
        assert!(!b.redo());
        assert_eq!(b.lines, vec![String::new()]);
    }

    #[test]
    fn undoing_to_saved_state_clears_dirty() {
        let mut b = Buffer::empty();
        for c in "abc".chars() {
            b.insert_char(c);
        }
        assert!(b.dirty);
        b.undo(); // revient au tampon vide (== état disque initial)
        assert!(
            !b.dirty,
            "revenir au contenu de référence doit remettre dirty à faux"
        );
        // Rétablir re-salit.
        b.redo();
        assert!(b.dirty);
    }

    #[test]
    fn noop_backspace_does_not_dirty_or_record() {
        let mut b = Buffer::empty();
        b.backspace(); // début de tampon : aucun effet
        assert!(!b.dirty);
        assert!(!b.undo());
    }

    // --- navigation par page / document ---

    fn buffer_of(n: usize) -> Buffer {
        let mut b = Buffer::empty();
        b.lines = (0..n).map(|i| format!("ligne {i}")).collect();
        b
    }

    #[test]
    fn page_down_and_up_clamp_to_bounds() {
        let mut b = buffer_of(10);
        b.page_down(4);
        assert_eq!(b.cursor_row, 4);
        b.page_down(100);
        assert_eq!(b.cursor_row, 9); // borné à la dernière ligne
        b.page_up(3);
        assert_eq!(b.cursor_row, 6);
        b.page_up(100);
        assert_eq!(b.cursor_row, 0);
    }

    #[test]
    fn page_moves_clamp_the_column() {
        let mut b = buffer_of(3);
        b.lines[0] = "court".to_string();
        b.lines[2] = "x".to_string();
        b.cursor_row = 0;
        b.move_end(); // colonne 5
        b.page_down(2);
        assert_eq!(b.cursor_row, 2);
        assert_eq!(b.cursor_col, 1); // bornée à la longueur de "x"
    }

    #[test]
    fn move_line_up_and_down_swap_and_carry_cursor() {
        let mut b = buf(&["un", "deux", "trois"]);
        b.cursor_row = 1; // "deux"
        b.move_line_up();
        assert_eq!(b.lines, vec!["deux", "un", "trois"]);
        assert_eq!(b.cursor_row, 0);
        b.move_line_down();
        assert_eq!(b.lines, vec!["un", "deux", "trois"]);
        assert_eq!(b.cursor_row, 1);
    }

    #[test]
    fn move_line_is_noop_at_edges_and_undoable() {
        let mut b = buf(&["un", "deux"]);
        // Haut depuis la 1re ligne : sans effet.
        b.cursor_row = 0;
        b.move_line_up();
        assert_eq!(b.lines, vec!["un", "deux"]);
        // Bas depuis la dernière : sans effet.
        b.cursor_row = 1;
        b.move_line_down();
        assert_eq!(b.lines, vec!["un", "deux"]);
        // Une opération réelle est annulable.
        b.move_line_up();
        assert_eq!(b.lines, vec!["deux", "un"]);
        b.undo();
        assert_eq!(b.lines, vec!["un", "deux"]);
    }

    #[test]
    fn duplicate_line_copies_below_and_follows_cursor() {
        let mut b = buf(&["un", "deux"]);
        b.cursor_row = 0;
        b.duplicate_line();
        assert_eq!(b.lines, vec!["un", "un", "deux"]);
        assert_eq!(b.cursor_row, 1);
        b.undo();
        assert_eq!(b.lines, vec!["un", "deux"]);
    }

    #[test]
    fn delete_line_removes_and_keeps_at_least_one_line() {
        let mut b = buf(&["un", "deux", "trois"]);
        b.cursor_row = 1;
        b.delete_line();
        assert_eq!(b.lines, vec!["un", "trois"]);
        assert_eq!(b.cursor_row, 1); // reste sur l'ancienne position (= "trois")
                                     // Supprimer jusqu'à la dernière laisse une ligne vide.
        b.delete_line();
        b.delete_line();
        assert_eq!(b.lines, vec![""]);
        assert_eq!(b.cursor_row, 0);
    }

    #[test]
    fn word_left_and_right_hop_over_words() {
        let mut b = Buffer::empty();
        for c in "foo  bar baz".chars() {
            b.insert_char(c);
        }
        // Curseur en fin ("baz" à l'index 9..12), col = 12.
        b.move_word_left();
        assert_eq!(b.cursor_col, 9); // début de "baz"
        b.move_word_left();
        assert_eq!(b.cursor_col, 5); // début de "bar"
        b.move_word_left();
        assert_eq!(b.cursor_col, 0); // début de "foo"
        b.move_word_right();
        assert_eq!(b.cursor_col, 5); // début de "bar"
        b.move_word_right();
        assert_eq!(b.cursor_col, 9); // début de "baz"
    }

    #[test]
    fn word_moves_cross_line_boundaries() {
        let mut b = Buffer::empty();
        for c in "ab".chars() {
            b.insert_char(c);
        }
        b.insert_newline();
        for c in "cd".chars() {
            b.insert_char(c);
        }
        b.move_home(); // (1, 0)
        b.move_word_left(); // début de ligne → fin ligne précédente
        assert_eq!((b.cursor_row, b.cursor_col), (0, 2));
    }

    #[test]
    fn delete_word_removes_previous_word_and_spaces() {
        let mut b = Buffer::empty();
        for c in "foo bar".chars() {
            b.insert_char(c);
        }
        b.delete_word_left(); // retire "bar"
        assert_eq!(b.lines[0], "foo ");
        assert_eq!(b.cursor_col, 4);
        b.delete_word_left(); // retire " foo" (espaces + mot)
        assert_eq!(b.lines[0], "");
        assert_eq!(b.cursor_col, 0);
        // Contenu revenu au tampon vide initial : dirty exact → faux.
        assert!(!b.dirty);
        // Chaque Ctrl-W est une étape d'annulation distincte.
        b.undo();
        assert_eq!(b.lines[0], "foo ");
        b.undo();
        assert_eq!(b.lines[0], "foo bar");
    }

    #[test]
    fn doc_start_and_end_jump_to_extremes() {
        let mut b = buffer_of(5);
        b.lines[4] = "fin".to_string();
        b.cursor_row = 2;
        b.move_doc_end();
        assert_eq!((b.cursor_row, b.cursor_col), (4, 3));
        b.move_doc_start();
        assert_eq!((b.cursor_row, b.cursor_col), (0, 0));
    }

    // --- sélection ---

    #[test]
    fn selection_range_normalizes_direction() {
        let mut b = buf(&["hello"]);
        b.cursor_col = 1;
        b.anchor_selection(); // ancre en (0,1)
        b.cursor_col = 4; // curseur en (0,4)
        assert_eq!(b.selection_range(), Some(((0, 1), (0, 4))));
        assert_eq!(b.selected_text().as_deref(), Some("ell"));
        // Direction inverse : mêmes bornes normalisées.
        b.cursor_col = 1;
        b.selection_anchor = Some((0, 4));
        assert_eq!(b.selection_range(), Some(((0, 1), (0, 4))));
    }

    #[test]
    fn select_all_covers_whole_buffer() {
        let mut b = buf(&["abc", "de"]);
        b.select_all();
        assert_eq!(b.selection_range(), Some(((0, 0), (1, 2))));
        assert_eq!(b.selected_text().as_deref(), Some("abc\nde"));
    }

    #[test]
    fn selected_text_spans_multiple_lines() {
        let mut b = buf(&["abc", "def", "ghi"]);
        b.cursor_row = 0;
        b.cursor_col = 1;
        b.anchor_selection();
        b.cursor_row = 2;
        b.cursor_col = 2;
        assert_eq!(b.selected_text().as_deref(), Some("bc\ndef\ngh"));
    }

    #[test]
    fn delete_selection_removes_span_and_merges() {
        let mut b = buf(&["abc", "def", "ghi"]);
        b.cursor_row = 0;
        b.cursor_col = 1;
        b.anchor_selection();
        b.cursor_row = 2;
        b.cursor_col = 2;
        assert!(b.delete_selection());
        assert_eq!(b.lines, vec!["ai"]);
        assert_eq!((b.cursor_row, b.cursor_col), (0, 1));
        assert!(b.selection_range().is_none());
        // Annulable.
        b.undo();
        assert_eq!(b.lines, vec!["abc", "def", "ghi"]);
    }

    #[test]
    fn insert_str_pastes_multiline_text() {
        let mut b = buf(&["axz"]);
        b.cursor_col = 1; // entre 'a' et 'x'
        b.insert_str("B\nC");
        assert_eq!(b.lines, vec!["aB", "Cxz"]);
        assert_eq!((b.cursor_row, b.cursor_col), (1, 1));
    }

    #[test]
    fn insert_str_replaces_active_selection() {
        let mut b = buf(&["hello"]);
        b.cursor_col = 1;
        b.anchor_selection();
        b.cursor_col = 4; // "ell" sélectionné
        b.insert_str("X");
        assert_eq!(b.lines, vec!["hXo"]);
        assert!(b.selection_range().is_none());
    }

    // --- recherche ---

    fn buf(lines: &[&str]) -> Buffer {
        let mut b = Buffer::empty();
        b.lines = lines.iter().map(|s| s.to_string()).collect();
        b
    }

    #[test]
    fn find_locates_forward_then_wraps() {
        let b = buf(&["foo bar", "baz foo", "qux"]);
        // Depuis le début : premier "foo" ligne 0.
        assert_eq!(b.find_from("foo", 0, 0), Some((0, 0)));
        // Juste après : prochain "foo" ligne 1.
        assert_eq!(b.find_from("foo", 0, 1), Some((1, 4)));
        // Après le dernier : boucle au premier.
        assert_eq!(b.find_from("foo", 1, 5), Some((0, 0)));
    }

    #[test]
    fn find_is_char_based_for_utf8() {
        let b = buf(&["héllo wörld", "café"]);
        assert_eq!(b.find_from("wörld", 0, 0), Some((0, 6)));
        assert_eq!(b.find_from("café", 0, 0), Some((1, 0)));
    }

    #[test]
    fn find_empty_or_absent_returns_none() {
        let b = buf(&["abc"]);
        assert_eq!(b.find_from("", 0, 0), None);
        assert_eq!(b.find_from("zzz", 0, 0), None);
        // Motif plus long que la ligne.
        assert_eq!(b.find_from("abcdef", 0, 0), None);
    }

    #[test]
    fn line_match_starts_lists_non_overlapping_occurrences() {
        let b = buf(&["aXbXcX", "none"]);
        assert_eq!(b.line_match_starts(0, "X"), vec![1, 3, 5]);
        // Non chevauchant : "aa" dans "aaaa" → positions 0 et 2.
        let b2 = buf(&["aaaa"]);
        assert_eq!(b2.line_match_starts(0, "aa"), vec![0, 2]);
        // Absent / vide / hors borne.
        assert!(b.line_match_starts(1, "X").is_empty());
        assert!(b.line_match_starts(0, "").is_empty());
        assert!(b.line_match_starts(9, "X").is_empty());
    }
}
