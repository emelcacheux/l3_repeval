use anyhow::Result;

mod cli;
mod command_exec;
mod config;
mod git_stats;
mod report;

fn main() -> Result<()> {
    //Récupère les arguments que l'utilisateur à entré lors de l'execution de l'application
    let args = cli::parse_args();
    // récupérer le fichier config
    let config = config::load(&args.config_path)?;

    //créer un RepoStats pour chacuns des .git et lancer les analyses
    let stats = git_stats::analyze_all(&args.root_dir)?;

    //Executer les commandes contenues dans l'objet config
    let results = command_exec::run_all(&config, &args.root_dir)?;

    //Displays the results in standard output
    command_exec::disp(results)?;

    //Générer le rapport à la toute fin à partir des repostats de chacun des .git
    report::generate(&args.input_csv, &args.output_csv, &stats)?;

    Ok(())
}
