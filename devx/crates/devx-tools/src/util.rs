//! Utilitaires partagés par les commandes du module `tools`.

use devx_core::DevxError;

/// Valide un nom fourni par l'utilisateur avant de l'utiliser comme composant
/// de chemin de fichier.
///
/// Seuls `[A-Za-z0-9._-]` sont acceptés ; les noms vides, `.` et `..` sont
/// rejetés. Cela empêche toute traversée de répertoire (`/`, `\`, `..`).
pub fn validate_name(kind: &str, name: &str) -> Result<(), DevxError> {
    if name.is_empty() {
        return Err(DevxError::invalid_args(format!(
            "le nom de {kind} ne peut pas être vide"
        )));
    }
    if name == "." || name == ".." {
        return Err(DevxError::invalid_args(format!(
            "le nom de {kind} `{name}` est réservé"
        )));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
    {
        return Err(DevxError::invalid_args(format!(
            "le nom de {kind} `{name}` est invalide (autorisé : A-Z a-z 0-9 . _ -)"
        )));
    }
    Ok(())
}

/// Découpe des octets de sortie en lignes UTF-8 (pertes remplacées).
pub fn bytes_to_lines(bytes: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_simple_names() {
        assert!(validate_name("snippet", "my-snip_1").is_ok());
    }

    #[test]
    fn rejects_traversal_and_separators() {
        for bad in ["", ".", "..", "../etc/passwd", "a/b", "a\\b", "a b"] {
            assert!(
                validate_name("session", bad).is_err(),
                "`{bad}` aurait dû être rejeté"
            );
        }
    }

    #[test]
    fn splits_lines_without_trailing_newline() {
        assert_eq!(bytes_to_lines(b"a\nb\n"), vec!["a", "b"]);
        assert_eq!(bytes_to_lines(b""), Vec::<String>::new());
    }
}
