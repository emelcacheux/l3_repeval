use crate::git_stats::RepoStats;
use anyhow::Result;
use csv::*;
use time::OffsetDateTime;

#[derive(Debug, serde::Deserialize, PartialEq, Eq)]
struct InpRecord {
    #[serde(rename = "nom")]
    pub name: String,
    #[serde(rename = "prénom")]
    pub surname: String,
    #[serde(rename = "n°étudiant")]
    pub id_stud: u32,
    #[serde(rename = "github")]
    pub git_log: String,
}

impl InpRecord {
    /// Cette fonction charge un fichier csv, le lit et transforme en la structure InpRecord.
    /// Puis met les données, si elles sont valides, dans un vecteur qu'elle retournera.
    ///
    /// # Argument
    ///
    /// Un 'input' un str du fichier csv à charger
    ///
    /// # Retour
    ///
    /// Un vecteur, Vec<InpRecord>, contenant le contenu du fichier analysé
    pub fn load(input: &str) -> Result<Vec<InpRecord>> {
        // Load the input csv file
        let mut r = Vec::new();
        let mut reader = csv::Reader::from_path(input)?;
        for res in reader.deserialize() {
            let rec: InpRecord = res?;
            r.push(rec);
        }
        Ok(r)
    }
    /// Détermine si un enregistrement d’entrée (`InpRecord`) est lié à des
    /// statistiques de dépôt (`RepoStats`).
    ///
    /// La relation est considérée comme valide lorsque le champ `git_log`
    /// de l’enregistrement correspond exactement à l’un des auteurs présents
    /// dans `RepoStats`.
    ///
    /// # Paramètres
    ///
    /// * `rs` — Les statistiques d’un dépôt Git, contenant notamment la liste
    ///          des auteurs (`rs.author`).
    ///
    /// # Retour
    ///
    /// Renvoie :
    ///
    /// - `true` si `self.git_log` correspond à un auteur du dépôt.
    /// - `false` sinon.
    ///
    /// # Exemple
    ///
    /// ```no_run
    /// if record.is_linked(&repo_stats) {
    ///     println!("L'étudiant est associé à ce dépôt.");
    /// }
    /// ```
    pub fn is_linked(&self, rs: &RepoStats) -> bool {
        // Verify if the InpRecord has a link with the Repo Stats
        let mut flag = false;
        for auth in &rs.author {
            if &self.git_log == auth {
                flag = true;
                break;
            }
        }
        flag
    }
    /// Exporte un enregistrement d’entrée (`InpRecord`) enrichi avec les
    /// statistiques du dépôt (`RepoStats`) dans un format compatible CSV.
    ///
    /// Cette fonction génère une ligne (`StringRecord`) combinant :
    ///
    /// - les informations personnelles contenues dans `InpRecord` ;
    /// - les métriques récupérées dans `RepoStats` pour l’auteur correspondant
    ///   à `self.git_log`.
    ///
    /// Les champs exportés sont notamment :
    ///
    /// 1. Nom  
    /// 2. Prénom  
    /// 3. Identifiant étudiant  
    /// 4. Identifiant Git  
    /// 5. Nombre total de commits du dépôt  
    /// 6. Nombre de commits de l’étudiant  
    /// 7. Timestamp du premier commit  
    /// 8. Timestamp du dernier commit  
    /// 9. Minimum entre deux commits  
    /// 10. Moyenne entre deux commits  
    /// 11. Longueur moyenne des messages  
    /// 12. Nombre total de modifications  
    /// 13. Nom du dépôt
    ///
    /// # Paramètres
    ///
    /// * `rs` — Statistiques du dépôt Git correspondant à l’étudiant.
    ///
    /// # Retour
    ///
    /// Retourne un `StringRecord` prêt à être écrit dans un fichier CSV.
    ///
    /// # Erreurs
    ///
    /// Peut retourner une erreur si :
    ///
    /// - les timestamps présents dans `RepoStats` ne peuvent pas être convertis
    ///   avec `OffsetDateTime::from_unix_timestamp`.
    ///
    /// # Exemple
    ///
    /// ```no_run
    /// let record = inp_rec.export(&stats)?;
    /// writer.write_record(&record)?;
    /// ```
    pub fn export(&self, rs: &RepoStats) -> Result<StringRecord> {
        // Recover the string record from combination of repostats and input record
        let borr_g = &self.git_log;
        let bbg = &borr_g;
        Ok(StringRecord::from(vec![
            self.name.clone(),
            self.surname.clone(),
            self.id_stud.to_string(),
            self.git_log.clone(),
            rs.nb_commits.to_string(),
            rs.c_p_author.get(*bbg).unwrap_or(&0usize).to_string(),
            OffsetDateTime::from_unix_timestamp(*rs.time_first.get(*bbg).unwrap())
                .unwrap()
                .to_string(),
            OffsetDateTime::from_unix_timestamp(*rs.time_last.get(*bbg).unwrap())
                .unwrap()
                .to_string(),
            rs.min_bet_com.get(*bbg).unwrap_or(&0i64).to_string(),
            rs.avg_bet_com.get(*bbg).unwrap_or(&0f64).to_string(),
            rs.msg_len.get(*bbg).unwrap_or(&0f64).to_string(),
            rs.nb_mod.get(*bbg).unwrap_or(&0usize).to_string(),
            rs.name.clone(),
        ]))
    }
}
/// Génère le rapport final sous forme de fichier CSV contenant les informations
/// des étudiants enrichies avec les statistiques provenant de leurs dépôts Git.
///
/// La fonction :
///
/// 1. charge le fichier CSV d’entrée contenant la liste des étudiants  
/// 2. ouvre (ou crée) le fichier CSV de sortie  
/// 3. écrit l’en-tête du rapport  
/// 4. pour chaque dépôt (`RepoStats`) et chaque étudiant (`InpRecord`) :  
///    - vérifie si l’étudiant est lié au dépôt via [`InpRecord::is_linked`]  
///    - exporte ses données enrichies via [`InpRecord::export`]  
///    - écrit la ligne dans le CSV final
///
/// # Paramètres
///
/// * `input_csv` — Chemin vers le fichier contenant les informations d’entrée
///                 (étudiants, identifiants Git, etc.).
/// * `output_csv` — Chemin du fichier CSV de sortie généré.
/// * `stats` — Liste des statistiques calculées sur les différents dépôts.
///
/// # Retour
///
/// Retourne `Ok(())` en cas de succès.
///
/// # Erreurs
///
/// Retourne une erreur si :
///
/// - le fichier CSV d’entrée ne peut pas être lu ;  
/// - le fichier CSV de sortie ne peut pas être créé ou écrit ;  
/// - une erreur survient lors de l’export d’un enregistrement.
///
/// # Exemple
///
/// ```no_run
/// let stats = compute_all_stats();
/// generate("etudiants.csv", "rapport.csv", &stats)?;
/// ```
pub fn generate(input_csv: &str, output_csv: &str, stats: &[RepoStats]) -> Result<()> {
    //Generate the final report saved in a csv file
    let inprecs = InpRecord::load(input_csv)?;

    let mut writer = csv::Writer::from_path(output_csv)?;
    let _ = writer.write_record([
        "Name",
        "Surname",
        "ID_Student",
        "GitHub",
        "Total Commit",
        "Nb of Commits",
        "Date First Commit",
        "Date Last Commit",
        "Minimum Time Between Commits",
        "Average Time Between Commits",
        "Average Message Length",
        "Number of Modification",
        "Repository Path (Name)",
    ]); // write headers
    for rs in stats {
        for inp_rec in &inprecs {
            if inp_rec.is_linked(rs) {
                let t = writer.write_record(&inp_rec.export(rs)?);
                if t.is_err() {
                    eprintln!("{:?}", t);
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::git_stats;

    #[test]
    pub fn test_load() {
        let t = InpRecord::load("inp.csv").unwrap();
        assert!(!t.is_empty(), "Record not loaded");
        assert_eq!(
            t[0],
            InpRecord {
                name: "RAZATOVO RANDRIANASOLO".to_string(),
                surname: "Erwan".to_string(),
                id_stud: 22304502,
                git_log: "Mowowglii".to_string()
            }
        )
    }

    #[test]
    pub fn test_link() {
        let current_dir = std::env::current_dir().unwrap();
        let borr_curr = current_dir.as_path();
        let recs = InpRecord::load("inp.csv").unwrap();
        let test = git_stats::analyze_repo(borr_curr).unwrap().unwrap();
        assert!(recs[0].is_linked(&test), "Author should be in Repo")
    }

    #[test]
    pub fn test_export() {
        let current_dir = std::env::current_dir().unwrap();
        let borr_curr = current_dir.as_path();
        let test = git_stats::analyze_repo(borr_curr).unwrap().unwrap();
        let recs = InpRecord::load("inp.csv").unwrap();
        assert!(recs[0].export(&test).is_ok(), "Function Failed")
    }

    #[test]
    pub fn test_generate() {
        let current_dir = std::env::current_dir().unwrap();
        let borr_curr = current_dir.as_path();
        let test = git_stats::analyze_all(borr_curr.to_str().unwrap()).unwrap();
        //let test = git_stats::analyze_all("C:\\Moi\\Univ\\L3-Informatique\\Semestre 5\\LSIN512 - Programmation Système\\TD\\Projet").unwrap();
        let t = generate("inp.csv", "test.csv", &test);
        assert!(t.is_ok(), "Generation failed");
    }
}
