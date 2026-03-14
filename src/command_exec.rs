use crate::config::Config;
use anyhow::Result;
use regex::Regex;
use std::process::*;

/// Exécute l'ensemble des commandes définies dans la configuration `Config`
/// depuis un répertoire racine donné, puis extrait les informations pertinentes
/// de leur sortie via une expression régulière.
///
/// Chaque commande définie dans `cfg.commands` possède :
///
/// - `name`   : un nom logique utilisé comme en-tête dans le résultat
/// - `cmd`    : la ligne de commande à exécuter (analysée avec `shell_words`)
/// - `regex`  : l'expression régulière permettant d'extraire des éléments importants
///
/// La fonction exécute chaque commande dans `root_path`, récupère:
///
/// - la sortie standard (`stdout`)
/// - la sortie d'erreur (`stderr`)
/// - le statut de sortie (succès ou échec)
///
/// puis applique la regex pour récupérer tous les éléments correspondants, qui sont
/// concaténés dans une chaîne finale, marqués par leur statut, et ajoutés au vecteur
/// de résultats.
///
/// # Paramètres
///
/// * `cfg` — La configuration contenant la liste des commandes à exécuter.
/// * `root_path` — Le chemin racine depuis lequel les commandes doivent être exécutées.
///
/// # Retour
///
/// Retourne un `Vec<String>` contenant une entrée par commande.
/// Chaque chaîne est formée comme suit :
///
/// ```text
/// <nom_commande>|<element trouvé> -> <SUCCESS/FAIL>*&&&*<element suivant> -> <SUCCESS/FAIL>*&&&*...
/// ```
///
/// # Erreurs
///
/// Retourne `Err` si :
///
/// - une commande ne peut pas être analysée par `shell_words`
/// - une commande échoue à être exécutée (panic interne `expect`)
///
/// # Exemple
///
/// ```no_run
/// let cfg = load_config();
/// let results = run_all(&cfg, "/tmp/project")?;
/// ```
pub fn run_all(cfg: &Config, root_path: &str) -> Result<Vec<String>> {
    //init the result vect
    let mut result = Vec::new();

    // get Path from root_path
    let rootp = std::path::Path::new(root_path);

    for c in &cfg.commands {
        // Init the string of result
        let mut s = String::new();

        //add header to s
        s.push_str(&c.name);
        s.push('|');

        //Must convert the string to a Vector of 'words' from command
        let tmp_c: Vec<String> = shell_words::split(&c.cmd).unwrap();

        //execute the commands
        let res = if cfg!(target_os = "windows") {
            Command::new(&tmp_c[0])
                .current_dir(rootp) // Run Command at the root path
                .args(&tmp_c[1..])
                .output()
                .expect("Command Failed")
        } else {
            Command::new(&tmp_c[0])
                .current_dir(rootp) // Run Command at the root path
                .args(&tmp_c[1..])
                .output()
                .expect("Command failed")
        };
        // Recover both error and standard output
        let combine_errstd = [res.stdout, res.stderr].concat();

        // Recover command status
        let status = if res.status.success() {
            "SUCCESS"
        } else {
            "FAIL"
        };

        //recover the string from output, apply regex and push the string
        let re = Regex::new(&c.regex).unwrap();
        re.find_iter(String::from_utf8_lossy(&combine_errstd).trim_ascii())
            .map(|m| m.as_str())
            .for_each(|element| {
                s.push_str(element);
                s.push_str(" -> ");
                s.push_str(status);
                s.push_str("*&&&*");
            });
        //recover the string from output and add the output to the string
        result.push(s);
    }
    Ok(result)
}

/// Affiche proprement les résultats générés par [`run_all`].
///
/// La fonction reçoit un vecteur de chaînes contenant, pour chaque commande :
///
/// ```text
/// <nom_commande>|<element> -> <SUCCESS/FAIL>*&&&*<element2> -> <SUCCESS/FAIL>*&&&*...
/// ```
///
/// Le format est alors transformé pour un affichage lisible :
///
/// ```text
/// <nom_commande> :
///     <element> -> <SUCCESS/FAIL>
///     <element2> -> <SUCCESS/FAIL>
///     ...
/// ```
///
/// Les marqueurs internes `|` et `*&&&*` servent uniquement de séparateurs.
///
/// # Paramètres
///
/// * `res_strings` — Le vecteur de chaînes renvoyé par [`run_all`].
///
/// # Retour
///
/// Retourne `Ok(())` après affichage sur la sortie standard.
///
/// # Exemple
///
/// ```no_run
/// let results = run_all(&cfg, "/tmp/project")?;
/// disp(results)?;
/// ```
///
/// # Notes
///
/// Cette fonction ne modifie pas les résultats, elle se contente de les afficher.
pub fn disp(res_strings: Vec<String>) -> Result<()> {
    // displays the result given from run_all
    for string in res_strings {
        let split: Vec<&str> = string.split("|").collect(); // split to recover header
        let header = split[0];
        println!("{} :", header);
        for result in split[1].split("*&&&*") {
            // split to recover body
            println!("\t{}", result);
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_run_all_basic() {
        // Create a temporary directory for safety
        let tmp_dir = tempdir().unwrap();
        let root_path = tmp_dir.path().to_str().unwrap();

        // Write a test file
        let test_file = tmp_dir.path().join("test.txt");
        fs::write(&test_file, "Hello World!\n12345\n").unwrap();

        // Setup Config with commands
        let cfg = crate::config::Config {
            commands: vec![crate::config::CommandConfig {
                name: "cargo_version".to_string(),
                cmd: "cargo --version".to_string(),
                regex: r"\d+\.\d+\.\d+".to_string(), // match version like 1.70.0
            }],
        };

        let res = run_all(&cfg, root_path).unwrap();
        // There should be one result
        assert_eq!(res.len(), 1);

        // The header should be correct
        assert!(res[0].starts_with("cargo_version|"));

        // The body should contain something that looks like a version
        assert!(res[0].contains("*&&&*")); // separator is present
        assert!(res[0].chars().any(|c| c.is_digit(10))); // at least one digit
    }

    #[test]
    fn test_disp_prints() {
        let res_strings = vec![
            "header1|item1*&&&*item2*&&&*".to_string(),
            "header2|foo*&&&*bar*&&&*".to_string(),
        ];

        // Just call disp and ensure it runs without panic
        let result = disp(res_strings);
        assert!(result.is_ok());
    }
}
