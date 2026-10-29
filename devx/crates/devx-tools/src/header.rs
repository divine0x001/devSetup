//! Commande `header` : insère ou met à jour l'en-tête standard de l'École 42.
//!
//! L'en-tête fait 11 lignes de 80 colonnes (marge de 5), avec le logo ASCII 42
//! aligné à droite et les métadonnées à gauche (fichier, auteur, dates). Le
//! format reproduit la spec `42header` ; norminette le vérifie au caractère
//! près, d'où les tests octet-exacts de ce module.
//!
//! Le login et le mail proviennent des variables d'environnement `USER42` /
//! `MAIL42` (comme l'outil officiel), avec repli sur `USER` / `MAIL`. Aucune
//! modification du contrat gelé `devx-core` n'est donc nécessaire.
//!
//! Les horodatages sont en UTC (format `AAAA/MM/JJ HH:MM:SS`) : norminette
//! vérifie le format, pas la valeur. Un `Updated` est rafraîchi à chaque appel ;
//! le `Created` d'un en-tête existant est préservé.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use devx_core::{Context, DevxError, Output};
use regex::Regex;

/// Largeur totale d'une ligne d'en-tête.
const WIDTH: usize = 80;
/// Marge : colonne de départ du contenu (et symétrique à droite).
const MARGIN: usize = 5;

/// Logo ASCII 42, aligné à droite sur les lignes 3 à 9 de l'en-tête.
const ASCII: [&str; 7] = [
    "        :::      ::::::::",
    "      :+:      :+:    :+:",
    "    +:+ +:+         +:+  ",
    "  +#+  +:+       +#+     ",
    "+#+#+#+#+#+   +#+        ",
    "     #+#    #+#          ",
    "    ###   ########.fr    ",
];

/// Délimiteurs de commentaire selon le type de fichier.
struct Delim {
    start: &'static str,
    end: &'static str,
}

/// Choisit les délimiteurs : `#` pour Makefile/shell/scripts, `/* */` sinon.
fn delim_for(path: &Path) -> Delim {
    let name = path.file_name().and_then(OsStr::to_str).unwrap_or("");
    let ext = path.extension().and_then(OsStr::to_str).unwrap_or("");
    let hash = matches!(name, "Makefile" | "makefile" | "GNUmakefile")
        || matches!(
            ext,
            "mk" | "sh" | "bash" | "zsh" | "fish" | "py" | "pl" | "rb"
        );
    if hash {
        Delim {
            start: "#",
            end: "#",
        }
    } else {
        Delim {
            start: "/*",
            end: "*/",
        }
    }
}

/// Lance `header <fichier>` : lit le fichier, applique l'en-tête et réécrit.
pub fn execute(ctx: &Context, args: &[String]) -> Result<Output, DevxError> {
    let rel = match args {
        [file] => file.clone(),
        _ => return Err(DevxError::invalid_args("usage: header <fichier>")),
    };

    let raw = Path::new(&rel);
    let path: PathBuf = if raw.is_absolute() {
        raw.to_path_buf()
    } else {
        ctx.workspace.root.join(raw)
    };
    if !path.is_file() {
        return Err(DevxError::message(format!("fichier introuvable: {rel}")));
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| DevxError::io(format!("{}: {e}", path.display())))?;
    let (login, email) = identity();
    let (new_content, updated) = apply(
        &content,
        &path,
        &delim_for(&path),
        &login,
        &email,
        now_unix(),
    );
    std::fs::write(&path, new_content)
        .map_err(|e| DevxError::io(format!("{}: {e}", path.display())))?;

    let verb = if updated {
        "header mis à jour"
    } else {
        "header ajouté"
    };
    Ok(Output::new(vec![format!("{verb}: {rel}")]))
}

/// Login et mail : `USER42`/`MAIL42`, repli sur `USER`/`MAIL`, puis `marvin`.
fn identity() -> (String, String) {
    let login = std::env::var("USER42")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "marvin".to_string());
    let email = std::env::var("MAIL42")
        .or_else(|_| std::env::var("MAIL"))
        .unwrap_or_else(|_| format!("{login}@student.42.fr"));
    (login, email)
}

/// Secondes Unix courantes (0 si l'horloge est antérieure à l'epoch).
fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Construit une ligne de contenu : `start` + marge + gauche + remplissage +
/// droite + marge + `end`, le tout calé à exactement [`WIDTH`] colonnes. La
/// partie gauche est tronquée si elle ne tient pas.
fn line(delim: &Delim, left: &str, right: &str) -> String {
    let inner = WIDTH - 2 * MARGIN;
    let rlen = right.chars().count();
    let max_left = inner.saturating_sub(rlen);
    let left: String = left.chars().take(max_left).collect();
    let fill = inner - left.chars().count() - rlen;
    format!(
        "{}{}{}{}{}{}{}",
        delim.start,
        " ".repeat(MARGIN - delim.start.len()),
        left,
        " ".repeat(fill),
        right,
        " ".repeat(MARGIN - delim.end.len()),
        delim.end
    )
}

/// Ligne de bordure (haut/bas) : `start` + espace + `*…*` + espace + `end`.
fn border(delim: &Delim) -> String {
    let stars = WIDTH - delim.start.len() - delim.end.len() - 2;
    format!("{} {} {}", delim.start, "*".repeat(stars), delim.end)
}

/// Rend les 11 lignes de l'en-tête.
fn render(
    filename: &str,
    delim: &Delim,
    login: &str,
    email: &str,
    created: &str,
    updated: &str,
) -> Vec<String> {
    vec![
        border(delim),
        line(delim, "", ""),
        line(delim, "", ASCII[0]),
        line(delim, filename, ASCII[1]),
        line(delim, "", ASCII[2]),
        line(delim, &format!("By: {login} <{email}>"), ASCII[3]),
        line(delim, "", ASCII[4]),
        line(delim, &format!("Created: {created} by {login}"), ASCII[5]),
        line(delim, &format!("Updated: {updated} by {login}"), ASCII[6]),
        line(delim, "", ""),
        border(delim),
    ]
}

/// Vrai si `line` ressemble à une bordure d'en-tête 42.
fn is_border(delim: &Delim, line: &str) -> bool {
    let t = line.trim();
    t.starts_with(delim.start) && t.ends_with(delim.end) && t.matches('*').count() >= 20
}

/// Applique l'en-tête à `content`. Si un en-tête existe déjà (bordures + ligne
/// `Created:`), il est remplacé en préservant la date de création ; sinon un
/// nouvel en-tête est inséré, suivi d'une ligne vide. Renvoie `(contenu, mis_à_jour)`.
fn apply(
    content: &str,
    path: &Path,
    delim: &Delim,
    login: &str,
    email: &str,
    now: u64,
) -> (String, bool) {
    let filename = path.file_name().and_then(OsStr::to_str).unwrap_or("file");
    let orig: Vec<String> = content.lines().map(str::to_string).collect();
    let now_str = format_timestamp(now);

    let has_header = orig.len() >= 11
        && is_border(delim, &orig[0])
        && is_border(delim, &orig[10])
        && orig[..11].iter().any(|l| l.contains("Created:"));

    let created_re = Regex::new(r"Created:\s*(\d{4}/\d{2}/\d{2} \d{2}:\d{2}:\d{2})")
        .expect("regex Created valide");
    let created = if has_header {
        orig[..11]
            .iter()
            .find_map(|l| created_re.captures(l).map(|c| c[1].to_string()))
            .unwrap_or_else(|| now_str.clone())
    } else {
        now_str.clone()
    };

    let mut out = render(filename, delim, login, email, &created, &now_str);
    if has_header {
        out.extend(orig[11..].iter().cloned());
    } else {
        out.push(String::new());
        out.extend(orig);
    }
    (format!("{}\n", out.join("\n")), has_header)
}

/// Formate des secondes Unix (UTC) en `AAAA/MM/JJ HH:MM:SS`.
fn format_timestamp(unix: u64) -> String {
    let days = (unix / 86_400) as i64;
    let rem = unix % 86_400;
    let (h, mi, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}/{m:02}/{d:02} {h:02}:{mi:02}:{s:02}")
}

/// Convertit un nombre de jours depuis 1970-01-01 en `(année, mois, jour)`
/// (algorithme de Howard Hinnant, calendrier grégorien proleptique).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c_delim() -> Delim {
        Delim {
            start: "/*",
            end: "*/",
        }
    }

    const EPOCH_2024: u64 = 1_704_067_200; // 2024/01/01 00:00:00 UTC
    const DAY_2024_02: u64 = 1_704_153_600; // 2024/01/02 00:00:00 UTC

    #[test]
    fn timestamp_formats_utc() {
        assert_eq!(format_timestamp(0), "1970/01/01 00:00:00");
        assert_eq!(format_timestamp(EPOCH_2024), "2024/01/01 00:00:00");
        assert_eq!(format_timestamp(EPOCH_2024 + 3661), "2024/01/01 01:01:01");
    }

    #[test]
    fn header_is_eleven_lines_of_eighty_columns() {
        let h = render(
            "main.c",
            &c_delim(),
            "marvin",
            "marvin@student.42.fr",
            "2024/01/01 00:00:00",
            "2024/01/01 00:00:00",
        );
        assert_eq!(h.len(), 11);
        for (i, l) in h.iter().enumerate() {
            assert_eq!(l.chars().count(), 80, "ligne {i} mal dimensionnée: {l:?}");
        }
    }

    #[test]
    fn header_carries_fields_logo_and_borders() {
        let h = render(
            "main.c",
            &c_delim(),
            "marvin",
            "marvin@student.42.fr",
            "2024/01/01 00:00:00",
            "2024/01/02 12:34:56",
        );
        assert_eq!(h[0], format!("/* {} */", "*".repeat(74)));
        assert_eq!(h[0], h[10]);
        assert!(h[3].contains("main.c"));
        assert!(h[5].contains("By: marvin <marvin@student.42.fr>"));
        assert!(h[7].contains("Created: 2024/01/01 00:00:00 by marvin"));
        assert!(h[8].contains("Updated: 2024/01/02 12:34:56 by marvin"));
        // Le logo ASCII complet est présent.
        let joined = h.join("\n");
        for art in [":::", "+:+", "+#+#+#+#+#+", "#+#", "########.fr"] {
            assert!(joined.contains(art), "logo incomplet: {art} manquant");
        }
    }

    #[test]
    fn hash_header_aligns_for_makefiles() {
        let h = render(
            "Makefile",
            &delim_for(Path::new("Makefile")),
            "marvin",
            "marvin@student.42.fr",
            "2024/01/01 00:00:00",
            "2024/01/01 00:00:00",
        );
        for l in &h {
            assert_eq!(l.chars().count(), 80);
            assert!(l.starts_with('#') && l.ends_with('#'));
        }
    }

    #[test]
    fn apply_inserts_header_then_blank_then_code() {
        let code = "int main(void)\n{\n\treturn (0);\n}\n";
        let (out, updated) = apply(
            code,
            Path::new("ft.c"),
            &c_delim(),
            "marvin",
            "marvin@student.42.fr",
            EPOCH_2024,
        );
        assert!(!updated);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[0], format!("/* {} */", "*".repeat(74)));
        assert!(lines[11].is_empty(), "ligne vide attendue après l'en-tête");
        assert_eq!(lines[12], "int main(void)");
        assert!(out.contains("Created: 2024/01/01 00:00:00 by marvin"));
    }

    #[test]
    fn apply_updates_existing_header_and_preserves_created() {
        let code = "int main(void)\n{\n\treturn (0);\n}\n";
        let (first, _) = apply(
            code,
            Path::new("ft.c"),
            &c_delim(),
            "marvin",
            "marvin@student.42.fr",
            EPOCH_2024,
        );
        let (second, updated) = apply(
            &first,
            Path::new("ft.c"),
            &c_delim(),
            "marvin",
            "marvin@student.42.fr",
            DAY_2024_02,
        );
        assert!(updated);
        assert!(second.contains("Created: 2024/01/01 00:00:00 by marvin"));
        assert!(second.contains("Updated: 2024/01/02 00:00:00 by marvin"));
        assert!(second.contains("return (0)"), "le code doit être préservé");
        // Un seul en-tête (deux bordures, pas quatre).
        let borders = second.lines().filter(|&l| is_border(&c_delim(), l)).count();
        assert_eq!(borders, 2);
    }
}
