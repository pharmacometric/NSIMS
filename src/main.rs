use clap::Parser;
use log::{info, warn, error};
use std::path::PathBuf;

mod config;
mod models;
mod dosing;
mod simulation;
mod output;
mod error;

use crate::config::Config;
use crate::simulation::Simulator;
use crate::error::PKError;

#[derive(Parser)]
#[command(name = "pk_simulation")]
#[command(about = "Population pharmacokinetics simulation program")]
struct Cli {
    /// Configuration file path
    #[arg(short, long)]
    config: PathBuf,
    
    /// Output directory
    #[arg(short, long)]
    output: PathBuf,
    
    /// Number of patients to simulate
    #[arg(short, long, default_value = "100")]
    patients: usize,
    
    /// Random seed for reproducibility
    #[arg(short, long)]
    seed: Option<u64>,
    
    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<(), PKError> {
    let cli = Cli::parse();
    
    // Initialize logging
    if cli.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();
    }
    
    if let Some(seed) = cli.seed {
        info!("Starting PK simulation with {} patients (seed: {})", cli.patients, seed);
    } else {
        info!("Starting PK simulation with {} patients (random seed)", cli.patients);
    }
    
    // Load configuration
    let config = Config::from_file(&cli.config)?;
    info!("Loaded configuration from {:?}", cli.config);
    
    // Create simulator
    let mut simulator = Simulator::new(config, cli.seed)?;
    
    // Run simulation
    let results = simulator.simulate_population(cli.patients)?;
    info!("Simulation completed for {} patients", results.len());
    
    // Create output directory if it doesn't exist
    std::fs::create_dir_all(&cli.output)?;
    
    // Save results
    crate::output::save_results(&results, &cli.output)?;
    info!("Results saved to {:?}", cli.output);
    
    Ok(())
}