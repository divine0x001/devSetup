//! Scoring flou par sous-séquence (style skim/fzf).
//!
//! Le score récompense les correspondances consécutives et les correspondances
//! en début de mot (après un séparateur), et pénalise les écarts ainsi que la
//! longueur du candidat. Un candidat qui ne contient pas la requête comme
//! sous-séquence est rejeté.

/// Caractères considérés comme séparateurs de mots.
fn is_separator(c: char) -> bool {
    matches!(c, '/' | '\\' | '_' | '-' | '.' | ' ' | ':')
}

/// Score un candidat contre une requête. Retourne `None` si la requête n'est
/// pas une sous-séquence du candidat.
///
/// La comparaison est insensible à la casse ASCII.
pub fn score(query: &str, candidate: &str) -> Option<i64> {
    let query: Vec<char> = query.chars().map(|c| c.to_ascii_lowercase()).collect();
    if query.is_empty() {
        return Some(0);
    }

    let chars: Vec<char> = candidate.chars().map(|c| c.to_ascii_lowercase()).collect();
    let original: Vec<char> = candidate.chars().collect();

    let mut total: i64 = 0;
    let mut search_from = 0usize;
    let mut previous: Option<usize> = None;

    for &qc in &query {
        let idx = (search_from..chars.len()).find(|&i| chars[i] == qc)?;

        total += 10;

        if idx == 0 {
            total += 12;
        } else if is_separator(original[idx - 1]) {
            total += 18;
        }

        if let Some(prev) = previous {
            if idx == prev + 1 {
                total += 15;
            } else {
                total -= (idx - prev - 1) as i64;
            }
        }

        previous = Some(idx);
        search_from = idx + 1;
    }

    total -= (chars.len() as i64) / 10;
    Some(total)
}

#[cfg(test)]
mod tests {
    use super::score;

    #[test]
    fn rejects_non_subsequence() {
        assert_eq!(score("xyz", "src/main.rs"), None);
    }

    #[test]
    fn empty_query_matches_everything() {
        assert_eq!(score("", "anything"), Some(0));
    }

    #[test]
    fn exact_match_scores_higher_than_scattered() {
        let exact = score("main", "main.rs").unwrap();
        let scattered = score("main", "mxaxixnfoo").unwrap();
        assert!(exact > scattered, "{exact} <= {scattered}");
    }

    #[test]
    fn case_insensitive() {
        assert!(score("MAIN", "src/main.rs").is_some());
    }
}
