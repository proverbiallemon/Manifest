use clap::{ArgGroup, Parser, Subcommand};
use manifest_core::{config, pins, report, scan::scan_library};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "manifest",
    version,
    about = "SoH load order analyzer and sorter"
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
    #[arg(long, global = true)]
    mods_dir: Option<PathBuf>,
    #[arg(long, global = true)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Cmd {
    Scan {
        #[arg(long)]
        json: bool,
    },
    #[command(group(ArgGroup::new("mode").required(true).args(["dry_run", "write"])))]
    Sort {
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        write: bool,
    },
    Explain {
        mod_name: String,
    },
}

fn resolve_paths(cli: &Cli) -> Result<(PathBuf, PathBuf), String> {
    let config_path = cli
        .config
        .clone()
        .or_else(config::default_config_path)
        .ok_or("could not determine config path; pass --config")?;
    let mods_dir = cli.mods_dir.clone().unwrap_or_else(|| {
        config_path
            .parent()
            .map(|p| p.join("mods"))
            .unwrap_or_else(|| PathBuf::from("mods"))
    });
    Ok((mods_dir, config_path))
}

fn run() -> Result<u8, String> {
    let cli = Cli::parse();
    let (mods_dir, config_path) = resolve_paths(&cli)?;
    let mods = scan_library(&mods_dir);
    let order = config::read_order(&config_path)?;
    let pin_rules = pins::read_pins(&pins::default_pins_path(&config_path))?;
    let rpt = report::build(&mods, &order, &pin_rules);
    let has_conflicts = !rpt.conflicts.is_empty() || !rpt.warnings.is_empty();

    match cli.command {
        Cmd::Scan { json } => {
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&rpt).map_err(|e| e.to_string())?
                );
            } else {
                println!(
                    "{} mods, {} conflicting assets, {} warnings",
                    rpt.mods.len(),
                    rpt.conflicts.len(),
                    rpt.warnings.len()
                );
                for w in &rpt.warnings {
                    println!("  warning: {w:?}");
                }
            }
            Ok(if has_conflicts { 3 } else { 0 })
        }
        Cmd::Sort { write, .. } => {
            if rpt.proposed_order == rpt.current_order {
                println!("load order already optimal");
                return Ok(0);
            }
            for m in &rpt.moves {
                println!("{}: {}", m.name, m.reason);
            }
            if write {
                config::write_order(&config_path, &rpt.proposed_order)?;
                println!(
                    "wrote {} entries to {}",
                    rpt.proposed_order.len(),
                    config_path.display()
                );
            } else {
                println!("dry run: {} moves proposed", rpt.moves.len());
            }
            Ok(0)
        }
        Cmd::Explain { mod_name } => {
            let overlaps: Vec<_> = rpt
                .conflicts
                .iter()
                .filter(|c| c.providers.contains(&mod_name))
                .collect();
            println!("{}: {} contested assets", mod_name, overlaps.len());
            for c in overlaps.iter().take(20) {
                println!("  {} -> winner: {}", c.asset, c.winner);
            }
            Ok(0)
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}
