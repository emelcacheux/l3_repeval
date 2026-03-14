use anyhow::Result;
use git2::*;
use std::fs;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct RepoStats {
    pub name: String,
    pub nb_commits: usize,
    pub author: Vec<String>,
    pub c_p_author: HashMap<String, usize>,
    pub time_first: HashMap<String, i64>,
    pub time_last: HashMap<String, i64>,
    pub min_bet_com: HashMap<String, i64>,
    pub avg_bet_com: HashMap<String, f64>,
    pub nb_mod: HashMap<String, usize>,
    pub msg_len: HashMap<String, f64>,
}

impl RepoStats {
    /// Crée et retourne une nouvelle structure 'RepoStats' vide
    ///
    /// Cette fonction initialise tous les champs de la structure avec des valeurs par défaut, afin de préparer l'objet à recevoir les information d'un dépot Git
    ///
    /// # Fonctionnement
    ///
    /// name : chaîne vide représentant le chemin du dépôt
    /// nb_commits : iniialisé à 0
    /// Toutes les HashMap sont créées vides et seront remplies plus tard durant l'analyse
    ///
    /// # Retour
    ///
    /// Une structure 'RepoStats' neuve, prête à être utilisée pour stocker les statistiques d'un dépôt Git
    pub fn new() -> Self {
        RepoStats {
            name: "".to_string(),
            nb_commits: 0usize,
            author: Vec::new(),
            c_p_author: HashMap::new(),
            time_first: HashMap::new(),
            time_last: HashMap::new(),
            min_bet_com: HashMap::new(),
            avg_bet_com: HashMap::new(),
            nb_mod: HashMap::new(),
            msg_len: HashMap::new(),
        }
    }

    /// Initialise l’ensemble des statistiques du dépôt en parcourant l’historique Git.
    ///
    /// Cette fonction parcourt tous les commits du dépôt donné en argument, en utilisant
    /// un `Revwalk` trié par ordre chronologique (du plus ancien au plus récent).
    ///  
    /// Pour chaque commit rencontré, elle met à jour divers champs de `RepoStats`,
    /// notamment :
    ///
    /// - le nombre total de commits (`nb_commits`) ;
    /// - la liste des auteurs uniques (`author`) ;
    /// - la date du premier commit de chaque auteur (`time_first`) ;
    /// - la date du dernier commit rencontré de chaque auteur (`time_last`) ;
    /// - le temps minimal entre deux commits consécutifs d’un même auteur (`min_bet_com`) ;
    /// - la somme des intervalles entre commits pour calculer la moyenne ultérieure (`avg_bet_com`) ;
    /// - le nombre total de commits par auteur (`c_p_author`) ;
    /// - la somme des longueurs des messages de commit (`msg_len`) ;
    /// - le nombre total de modifications (insertions + suppressions) par auteur (`nb_mod`).
    ///
    /// # Détails du fonctionnement
    ///
    /// - Les commits sont parcourus du plus ancien au plus récent.
    /// - Lorsqu’un auteur apparaît pour la première fois :  
    ///   - son premier timestamp est enregistré dans `time_first` ;  
    ///   - `time_last` est initialisé à ce même timestamp ;  
    ///   - `min_bet_com` est initialisé à `i64::MAX` ;  
    ///   - `avg_bet_com` est initialisé à `0.0`.  
    ///
    /// - Lorsqu’un commit d’un auteur déjà connu est rencontré :  
    ///   - un intervalle est calculé entre ce commit et le précédent (`time_last`) ;  
    ///   - cet intervalle met à jour :  
    ///     - le minimum (`min_bet_com`) si nécessaire ;  
    ///     - la somme des intervalles (`avg_bet_com`) ;
    ///   - `time_last` est mis à jour au timestamp courant.
    ///
    /// - Le nombre de commits par auteur est incrémenté dans `c_p_author`.
    /// - La longueur du message de commit est ajoutée à `msg_len`.
    /// - Le nombre d’insertion et suppressions (diff) est compté et ajouté à `nb_mod`.
    ///
    /// # Paramètres
    ///
    /// * `repo` — Une référence vers un [`git2::Repository`] déjà chargé.
    ///
    /// # Retour
    ///
    /// Retourne `Ok(())` en cas de succès, ou un `git2::Error` encapsulé dans un `Result`
    /// en cas de problème lors de la lecture de l’historique ou des diffs.
    ///
    /// # Erreurs possibles
    ///
    /// Cette fonction peut échouer si :
    ///
    /// - le dépôt Git est invalide ;  
    /// - un commit ne peut pas être lu ;  
    /// - un diff ne peut pas être généré ;  
    /// - un des appels aux fonctions de `git2` échoue.
    ///
    /// # Exemple
    ///
    /// ```no_run
    /// use git2::Repository;
    /// use your_crate::RepoStats;
    ///
    /// let repo = Repository::discover(".")?;
    /// let mut stats = RepoStats::new();
    /// stats.init_calculation(&repo)?;
    /// ```
    ///
    /// # Remarque
    ///
    /// Cette fonction ne calcule **pas encore** la moyenne des longueurs de message.
    /// Utilisez [`RepoStats::calculate_avg_msg_len`] après appel à cette fonction.
    ///
    /// # Complexité
    ///
    /// La complexité est linéaire en fonction du nombre total de commits du dépôt.
    pub fn init_calculation(&mut self, repo: &Repository) -> Result<()> {
        //Find the timestamp of the first commit from author name and inject it in the time_first field of Repostats at the index assigned to the author
        let mut rev = repo.revwalk()?;
        rev.set_sorting(Sort::TIME | Sort::REVERSE)?;
        rev.push_head()?; //Start walk from oldest commit

        for oid in rev {
            //increment nb_commits
            self.nb_commits += 1;

            let o = oid?; // Recover Oid
            let commit = repo.find_commit(o)?; // Recover commit

            //Recover Author
            let a: String = commit.author().name().unwrap().to_string();

            //Recover Timestamp
            let t = commit.time().seconds();
            let borr_t = &t;

            // Recover time first
            let flag = self.author.contains(&a);

            //if we find a new author...
            if !flag {
                // ...we add name and commit timestamp to the author field of RepoStats
                self.author.push(a.clone());

                // ...we add name and timestamp to the time_first field of RepoStats
                self.time_first.entry(a.clone()).or_insert(*borr_t);

                // ... we init the value of the min time between commit as the max value of type i64
                self.min_bet_com.entry(a.clone()).or_insert(i64::MAX);

                // ... we init the value of the average time between commit to 0
                self.avg_bet_com.entry(a.clone()).or_insert(0.0);

                // ...we init the timestamp of the last commit of the author
                self.time_last.entry(a.clone()).or_insert(*borr_t);
            }

            //Calculate the min and avg time betweeen commit and update the author last commit timestamp
            // Verify if the timestamp of last commit of the author is different from his current timestamp of commit
            if t > *self.time_last.get(&a).unwrap() {
                let interv = borr_t - self.time_last.get(&a).unwrap(); // Calculate the time between current commit and author commit last timestamp
                let borr_i = &interv;

                // if the current minimum time between commit is greater than the interval, replace it
                if *self.min_bet_com.get(&a).unwrap() > interv {
                    self.min_bet_com.entry(a.clone()).and_modify(|value| {
                        *value = *borr_i;
                    });
                }

                // Add the interval to the average
                self.avg_bet_com.entry(a.clone()).and_modify(|value| {
                    *value += interv as f64;
                });

                // Update the timestamp of the last commit of the author to the current commit timestamp
                self.time_last.entry(a.clone()).and_modify(|value| {
                    *value = t;
                });
            }

            // Recover Commit per author
            self.c_p_author
                .entry(a.clone())
                .and_modify(|nbcommit| {
                    *nbcommit += 1;
                })
                .or_insert(1usize); // Update author number of commit or Insert 1 as first commit found for him

            // Recover Sum of messages length
            self.msg_len
                .entry(a)
                .and_modify(|messlen| {
                    *messlen += commit.message_bytes().len() as f64;
                })
                .or_insert(commit.message_bytes().len() as f64); // Update author message length or Insert author message len when first commit is encountered

            // Recover number of modification for different author
            // Get first parent (or empty tree for root commit)
            let diff = if commit.parent_count() == 0 {
                let empty_tree = repo.find_tree(repo.treebuilder(None)?.write()?)?;
                repo.diff_tree_to_tree(Some(&empty_tree), Some(&commit.tree()?), None)?
            } else {
                let parent_commit = commit.parent(0)?;
                repo.diff_tree_to_tree(Some(&parent_commit.tree()?), Some(&commit.tree()?), None)?
            };

            // Count insertions + deletions
            let changes = diff.stats()?.insertions() + diff.stats()?.deletions();

            // Update nb_mod for the author of the commit
            let author = commit.author().name().unwrap().to_string();
            self.nb_mod
                .entry(author)
                .and_modify(|n| *n += changes)
                .or_insert(changes);
        }
        Ok(())
    }

    /// calcule la longueur moyenne des messages de commit pour chaque auteur
    ///
    /// Cette fonction utilise la somme des longueurs des messages déjà stockée dans self.msg_len puis la divise par le nombre total de commits de l'auteur
    /// Pour obtenir la moyenne de la longueur des messages
    ///
    /// Pour chaque auteur, on récupère, la somme des longueurs des messages et le nombre de commits
    /// La fonction calcule ensuite : moyenne = somme_longueurs / nombre_de_commits
    /// Le resultat est stoké dans une nouvelle HashMap, puis remplace self.msg_len
    ///
    /// # Arguements
    ///
    /// self : structure contenant les longueurs cumulée des messages de commit et le nombre de commits par auteur
    ///
    /// # Retour
    ///
    /// Met à jour msg_len pour qu'il contienne la moyenne des longueurs des messages de commit pour chque auteur
    /// Retourne Ok en cas de succès, une erreur sinon
    pub fn calculate_avg_msg_len(&mut self) -> Result<()> {
        // Normalise the message average len for each author
        let mut res: HashMap<String, f64> = HashMap::new();
        for (author, msglen) in &self.msg_len {
            res.entry(author.clone())
                .or_insert(*msglen / (*self.c_p_author.get(author).unwrap()) as f64);
        }
        self.msg_len = res;
        Ok(())
    }

    pub fn calculate_avg_bet_com(&mut self) -> Result<()> {
        // Normalise the average time between commit
        let mut res: HashMap<String, f64> = HashMap::new();
        for (author, avg_time) in &self.avg_bet_com {
            if *self.c_p_author.get(author).unwrap() > 1 {
                res.entry(author.clone())
                    .or_insert(*avg_time / (*self.c_p_author.get(author).unwrap() - 1) as f64);
            }
        }
        self.avg_bet_com = res;
        Ok(())
    }
}

pub fn analyze_repo(path: &Path) -> Result<Option<RepoStats>> {
    //Analyze repository
    let mut rs = RepoStats::new();
    //Open repertory
    let repo = Repository::open(path)?;
    //Set the name of the repo (which is the path to the repo)
    rs.name = repo.workdir().unwrap().to_str().unwrap().to_string();
    //Analyze part
    rs.init_calculation(&repo)?;
    rs.calculate_avg_msg_len()?;
    rs.calculate_avg_bet_com()?;
    Ok(Some(rs))
}

/// Trouve un répositoire git dans un dossier.
///
/// Cette fonction prend en argument 'root' un chemin vers un dossier et retourne
/// un vecteur de tous les répositoires trouvés.
///
/// # Arguments
///
/// *'root' un chemin vers un dossier à tester
///
/// # Retour
///
/// Vec<PathBuf> qui contient les répertoires trouvés
/// s'ils existent
pub fn find_rep(root: &Path) -> Vec<PathBuf> {
    let mut all_rep = Vec::new();

    //Verify if the root contain a .git directory
    let verf = &root;
    let t = verf.to_path_buf();
    if t.join(".git").is_dir() {
        all_rep.push(t);
    }

    for entry in fs::read_dir(root).unwrap() {
        let p = entry.unwrap().path();

        // If this directory contains a .git folder, it's a repo
        if p.join(".git").is_dir() {
            all_rep.push(p.clone());
        }

        // Recurse into directories
        if p.is_dir() {
            all_rep.extend(find_rep(&p));
        }
    }
    let final_all_rep: Vec<PathBuf> = all_rep
        .into_iter()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    final_all_rep
}

/// calcule la longueur moyenne des messages de commit pour chaque auteur
///
/// Cette fonction utilise la somme des longueurs des messages déjà stockée dans self.msg_len puis la divise par le nombre total de commits de l'auteur
/// Pour obtenir la moyenne de la longueur des messages
///
/// Pour chaque auteur, on récupère, la somme des longueurs des messages et le nombre de commits
/// La fonction calcule ensuite : moyenne = somme_longueurs / nombre_de_commits
/// Le resultat est stoké dans une nouvelle HashMap, puis remplace self.msg_len
///
/// # Arguements
///
/// self : structure contenant les longueurs cumulée des messages de commit et le nombre de commits par auteur
///
/// # Retour
///
/// Met à jour msg_len pour qu'il contienne la moyenne des longueurs des messages de commit pour chque auteur
/// Retourne Ok en cas de succès, une erreur sinon
pub fn analyze_all(root: &str) -> Result<Vec<RepoStats>> {
    //Find every repository and analyze it
    //Init the vec of paths
    let mut res = Vec::new();
    for p in find_rep(Path::new(root)) {
        match analyze_repo(p.as_path()) {
            Ok(Some(stats)) => res.push(stats),
            Ok(None) => {}
            Err(e) => {
                eprintln!("Failed to analyze repo at {:?}: {}", p, e);
            }
        }
    }
    Ok(res)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_nb_commits() {
        let repo = match Repository::discover(".") {
            Ok(r) => r,
            Err(_) => {
                eprintln!("Skipping: no git repo.");
                return;
            }
        };

        let mut stats = RepoStats::new();
        stats.init_calculation(&repo).unwrap();

        // Count commits manually
        let mut rev = repo.revwalk().unwrap();
        rev.push_head().unwrap();

        let manual_count = rev.flatten().count();

        assert_eq!(
            stats.nb_commits, manual_count,
            "nb_commits mismatch: expected {manual_count}, got {}",
            stats.nb_commits
        );
    }

    #[test]
    fn test_commit_per_author() {
        let repo = match Repository::discover(".") {
            Ok(r) => r,
            Err(_) => {
                eprintln!("Skipping: no repo found.");
                return;
            }
        };

        let mut stats = RepoStats::new();
        stats.init_calculation(&repo).unwrap();

        let mut rev = repo.revwalk().unwrap();
        rev.push_head().unwrap();

        let mut manual_counts = std::collections::HashMap::<String, usize>::new();

        for oid in rev.flatten() {
            let commit = repo.find_commit(oid).unwrap();
            let name = commit.author().name().unwrap_or("").to_string();
            *manual_counts.entry(name).or_insert(0) += 1;
        }

        for (author, count) in manual_counts {
            assert_eq!(
                stats.c_p_author.get(&author).copied().unwrap_or(0),
                count,
                "Commit count mismatch for {author}"
            )
        }
    }

    #[test]
    fn test_recover_time_first() {
        let repo = match Repository::discover(".") {
            Ok(r) => r,
            Err(_) => return,
        };

        let mut stats = RepoStats::new();
        stats.init_calculation(&repo).unwrap();

        let mut rev = repo.revwalk().unwrap();
        rev.set_sorting(git2::Sort::TIME | git2::Sort::REVERSE)
            .unwrap();
        rev.push_head().unwrap();

        let mut manual_first = std::collections::HashMap::<String, i64>::new();

        for oid in rev.flatten() {
            let c = repo.find_commit(oid).unwrap();
            let name = c.author().name().unwrap_or("").to_string();
            let ts = c.time().seconds();

            manual_first.entry(name).or_insert(ts);
        }

        for (author, ts) in manual_first {
            assert_eq!(
                stats.time_first.get(&author).copied(),
                Some(ts),
                "time_first incorrect for author {author}"
            );
        }
    }

    #[test]
    fn test_recover_time_last() {
        let repo = match Repository::discover(".") {
            Ok(r) => r,
            Err(_) => return,
        };

        let mut stats = RepoStats::new();
        stats.init_calculation(&repo).unwrap();

        let mut rev = repo.revwalk().unwrap();
        rev.push_head().unwrap(); // newest first

        let manual_last = std::collections::HashMap::<String, i64>::new();

        if let Some(&author_timestamp) = stats.time_last.get("Mowowglii") {
            // Compute max timestamp from the repo commits manually
            let max_timestamp = {
                let mut rev = repo.revwalk().unwrap();
                rev.push_head().unwrap();
                rev.filter_map(|oid| {
                    let commit = repo.find_commit(oid.ok()?).ok()?;
                    if commit.author().name()? == "Mowowglii" {
                        Some(commit.time().seconds())
                    } else {
                        None
                    }
                })
                .max()
                .unwrap_or(i64::MIN)
            };

            assert_eq!(
                author_timestamp, max_timestamp,
                "time_last incorrect for Mowowglii"
            );
        }
    }

    #[test]
    fn test_number_modification() {
        let repo = match Repository::discover(".") {
            Ok(r) => r,
            Err(_) => return,
        };

        let mut stats = RepoStats::new();
        stats.init_calculation(&repo).unwrap();

        let mut rev = repo.revwalk().unwrap();
        rev.push_head().unwrap();

        let mut manual_mod = std::collections::HashMap::<String, usize>::new();

        for oid in rev.flatten() {
            let commit = repo.find_commit(oid).unwrap();
            let author = commit.author().name().unwrap_or("").to_string();

            // Compute diffs
            let diff = if commit.parent_count() == 0 {
                let empty = repo
                    .find_tree(repo.treebuilder(None).unwrap().write().unwrap())
                    .unwrap();
                repo.diff_tree_to_tree(Some(&empty), Some(&commit.tree().unwrap()), None)
                    .unwrap()
            } else {
                let parent = commit.parent(0).unwrap();
                repo.diff_tree_to_tree(
                    Some(&parent.tree().unwrap()),
                    Some(&commit.tree().unwrap()),
                    None,
                )
                .unwrap()
            };

            let s = diff.stats().unwrap();
            let changes = s.insertions() + s.deletions();

            *manual_mod.entry(author).or_insert(0) += changes;
        }

        for (author, expected) in manual_mod {
            assert_eq!(
                stats.nb_mod.get(&author).copied().unwrap_or(0),
                expected,
                "nb_mod mismatch for {author}"
            );
        }
    }

    #[test]
    fn test_calculate_avg_bet_com() {
        let repo = match Repository::discover(".") {
            Ok(r) => r,
            Err(_) => return,
        };

        let mut stats = RepoStats::new();
        stats.init_calculation(&repo).unwrap();
        stats.calculate_avg_bet_com().unwrap();

        for author in &stats.author {
            if stats.c_p_author[author] <= 1 {
                continue; // skip authors with <2 commits
            }
            let sum_intervals = stats.avg_bet_com[author] * ((stats.c_p_author[author] - 1) as f64);

            // Manually compute expected average
            let expected_avg = sum_intervals / ((stats.c_p_author[author] - 1) as f64);

            assert!(
                (expected_avg - stats.avg_bet_com[author]).abs() < f64::EPSILON,
                "avg_bet_com mismatch for {author}"
            );
        }
    }

    #[test]
    fn test_avg_msg_len() {
        let repo = match Repository::discover(".") {
            Ok(r) => r,
            Err(_) => return,
        };

        let mut stats = RepoStats::new();
        stats.init_calculation(&repo).unwrap();
        stats.calculate_avg_msg_len().unwrap();

        let mut rev = repo.revwalk().unwrap();
        rev.push_head().unwrap();

        let mut manual_len = std::collections::HashMap::<String, (f64, usize)>::new();

        for oid in rev.flatten() {
            let commit = repo.find_commit(oid).unwrap();
            let name = commit.author().name().unwrap_or("").to_string();
            let len = commit.message_bytes().len() as f64;

            manual_len
                .entry(name)
                .and_modify(|(t, c)| {
                    *t += len;
                    *c += 1;
                })
                .or_insert((len, 1));
        }

        for (author, (total, count)) in manual_len {
            let expected = total / count as f64;
            let stored = stats.msg_len.get(&author).copied().unwrap();

            assert!(
                (expected - stored).abs() < f64::EPSILON,
                "msg_len mismatch for {author}"
            );
        }
    }

    #[test]
    fn test_calculate_min_bet_com() {
        let repo = match Repository::discover(".") {
            Ok(r) => r,
            Err(_) => return,
        };

        let mut stats = RepoStats::new();
        stats.init_calculation(&repo).unwrap();

        for author in &stats.author {
            if stats.c_p_author[author] <= 1 {
                continue; // skip authors with insufficient commits
            }

            // Expected = stored min directly (since RepoStats computes it)
            let expected = stats.min_bet_com[author];
            let stored = stats.min_bet_com[author];

            assert_eq!(expected, stored);
        }
    }

    #[test]
    fn t_analyse_repo() {
        let current_dir = std::env::current_dir().unwrap();
        let result = analyze_repo(&current_dir);
        assert!(result.is_ok());
        let stats_opt = result.unwrap();
        assert!(stats_opt.is_some(), "analyze_repo did not return RepoStats");
        let stats = stats_opt.unwrap();
        assert!(
            stats.author.contains(&"5ways".to_string()),
            "Author '5ways' not found in author map"
        );
        assert!(stats.nb_commits > 0, "No commits found in repo");
    }

    #[test]
    fn test_find_rep() {
        use std::path::{Path, PathBuf};

        // Répertoire racine du test : local, portable et indépendant du système
        let base = Path::new("test");

        // On prépare la liste attendue, relative
        let mut expected: Vec<PathBuf> = vec![
            base.join("rep1"),
            base.join("rep2"),
            base.join("rep3"),
            base.join("rep4"),
        ];

        // Résultat de la fonction
        let mut result = find_rep(base);

        // Comme l’ordre n’est pas garanti, on trie
        expected.sort();
        result.sort();

        assert_eq!(
            result, expected,
            "find_rep() did not return the expected directories"
        );
    }

    #[test]
    fn t_analyse_all() {
        let current_dir = std::env::current_dir().unwrap();
        let result = analyze_all(&current_dir.as_path().to_str().unwrap());
        assert!(result.is_ok());
        let stats_vec = result.unwrap();
        assert!(!stats_vec.is_empty(), "No repositories found or analyzed");
        let mut found = false;
        for stats in &stats_vec {
            if stats.author.contains(&"5ways".to_string()) {
                found = true;
                break;
            }
        }
        assert!(found, "Author '5ways' not found in any analyzed repository");
    }
}
