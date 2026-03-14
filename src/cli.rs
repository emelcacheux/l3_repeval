use clap::Parser;
/// Analyse et évalue un ensemble de dépôts Git.
#[derive(Parser, Debug)]
pub struct Args {
    /// Répertoire racine contenant les dépôts
    pub root_dir: String,

    /// Fichier CSV d'entrée (noms, logins, etc.)
    pub input_csv: String,

    /// Fichier CSV de sortie
    pub output_csv: String,

    /// Fichier de configuration (commandes à exécuter)
    #[arg(short, long, default_value = "config.toml")]
    pub config_path: String,
}

pub fn parse_args() -> Args {
    Args::parse()
}
