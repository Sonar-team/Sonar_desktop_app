//! Contenus des dialogues « À propos » et « Changelog » du menu À propos.

/// Présentation de SONAR, de sa licence, de la matrice SFMS et des versions de
/// la chaîne de build injectées par la CI (`SONAR_*_VERSION`).
pub fn about_message() -> String {
    format!(
        concat!(
            "SONAR {}\n\n",
            "Informations générales\n",
            "{}\n",
            "Analyseur passif de flux réseau.\n",
            "Licence : GNU AGPL v3.0 uniquement ({})\n",
            "Crédits / auteurs : {}\n",
            "Dépôt : {}\n\n",
            "Chaîne de build\n",
            "Rust: {}\n",
            "Node.js: {}\n",
            "Deno: {}\n",
            "Tauri CLI: {}\n",
            "Npcap (Windows): requis séparément\n\n",
            "Matrice SONAR (SFMS)\n",
            "Le SONAR Flow Matrix Standard est un CSV déterministe et réimportable. ",
            "Chaque ligne agrège un flux unidirectionnel identifié par ses extrémités, ",
            "protocoles et ports.\n",
            "count, total_bytes et last_seen résument l’activité ; encap_id relie les ",
            "niveaux d’un tunnel et origin trace les fichiers fusionnés."
        ),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_DESCRIPTION"),
        env!("CARGO_PKG_LICENSE"),
        env!("CARGO_PKG_AUTHORS"),
        env!("CARGO_PKG_REPOSITORY"),
        option_env!("SONAR_RUST_VERSION").unwrap_or("unknown"),
        option_env!("SONAR_NODE_VERSION").unwrap_or("unknown"),
        option_env!("SONAR_DENO_VERSION").unwrap_or("unknown"),
        option_env!("SONAR_TAURI_CLI_VERSION").unwrap_or("unknown"),
    )
}

const CHANGELOG: &str = include_str!("../../../CHANGELOG.md");

/// Notes de la dernière release, extraites du CHANGELOG.md embarqué à la
/// compilation. Chaque release commence par un en-tête `## **[x.y.z] - date**` ;
/// les sous-sections (`## ✨ ...`) ne matchent pas le motif `## **[`.
pub fn changelog_message() -> String {
    const RELEASE_HEADER: &str = "## **[";

    let Some(start) = CHANGELOG.find(RELEASE_HEADER) else {
        return "Changelog indisponible".to_string();
    };

    let latest = &CHANGELOG[start..];
    let end = latest[RELEASE_HEADER.len()..]
        .find(RELEASE_HEADER)
        .map(|i| i + RELEASE_HEADER.len())
        .unwrap_or(latest.len());

    strip_markdown(latest[..end].trim_end())
}

/// Allège le markdown pour un dialogue natif : retire les marqueurs de titre
/// et de gras, garde les puces telles quelles.
fn strip_markdown(text: &str) -> String {
    text.lines()
        .map(|line| line.trim_start_matches('#').trim_start().replace("**", ""))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changelog_message_contains_current_version() {
        // Garde-fou de release : le CHANGELOG.md doit être mis à jour en même
        // temps que la version dans Cargo.toml.
        assert!(
            changelog_message().contains(&format!("[{}]", env!("CARGO_PKG_VERSION"))),
            "la dernière entrée du CHANGELOG.md ne correspond pas à la version {}",
            env!("CARGO_PKG_VERSION")
        );
    }

    #[test]
    fn changelog_message_stops_at_previous_release() {
        // Version de l'avant-dernière release, lue dans le CHANGELOG brut.
        let second_header = CHANGELOG
            .match_indices("## **[")
            .nth(1)
            .map(|(i, _)| &CHANGELOG[i + "## **[".len()..])
            .and_then(|rest| rest.split_once(']'))
            .map(|(version, _)| format!("[{version}]"))
            .expect("le CHANGELOG doit contenir au moins deux releases");

        assert!(
            !changelog_message().contains(&second_header),
            "le message ne doit contenir que la dernière release"
        );
    }

    #[test]
    fn about_message_mentions_separate_npcap_installation() {
        assert!(about_message().contains("Npcap (Windows): requis séparément"));
    }

    #[test]
    fn about_message_contains_current_version_and_package_metadata() {
        let message = about_message();

        assert!(
            message.starts_with(&format!("SONAR {}", env!("CARGO_PKG_VERSION"))),
            "le message doit commencer par la version courante de SONAR"
        );
        assert!(message.contains(env!("CARGO_PKG_DESCRIPTION")));
        assert!(message.contains(env!("CARGO_PKG_AUTHORS")));
        assert!(message.contains(env!("CARGO_PKG_REPOSITORY")));
        assert!(
            message.contains(&format!(
                "Licence : GNU AGPL v3.0 uniquement ({})",
                env!("CARGO_PKG_LICENSE")
            )),
            "le message doit afficher la licence déclarée dans Cargo.toml"
        );
    }

    #[test]
    fn about_message_explains_the_sfms_matrix() {
        let message = about_message();

        assert!(message.contains("SONAR Flow Matrix Standard"));
        assert!(message.contains("flux unidirectionnel"));

        for field in ["count", "total_bytes", "last_seen", "encap_id", "origin"] {
            assert!(
                message.contains(field),
                "le descriptif SFMS doit expliquer le champ {field}"
            );
        }
    }
}
