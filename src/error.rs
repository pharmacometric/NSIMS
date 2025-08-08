use thiserror::Error;
use rand_distr::NormalError;

#[derive(Error, Debug)]
pub enum PKError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    
    #[error("Invalid model configuration: {0}")]
    InvalidModel(String),
    
    #[error("Invalid dosing configuration: {0}")]
    InvalidDosing(String),
    
    #[error("Simulation error: {0}")]
    Simulation(String),
    
    #[error("Parameter validation error: {0}")]
    Validation(String),
    
    #[error("Random number generation error")]
    Random,
}

pub type PKResult<T> = Result<T, PKError>;