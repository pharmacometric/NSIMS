use serde::{Deserialize, Serialize};
use std::path::Path;
use std::collections::HashMap;
use crate::error::{PKError, PKResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub model: ModelConfig,
    pub dosing: DosingConfig,
    pub population: PopulationConfig,
    pub simulation: SimulationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub compartments: u8, // 1, 2, or 3
    pub parameters: HashMap<String, ParameterConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterConfig {
    pub theta: f64,           // Typical value
    pub omega: Option<f64>,   // Inter-individual variability (CV%)
    pub bounds: Option<(f64, f64)>, // Lower and upper bounds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DosingConfig {
    pub route: DosingRoute,
    pub amount: f64,
    pub times: Vec<f64>,
    pub additional: Option<AdditionalDosingParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DosingRoute {
    Oral,
    IvBolus,
    IvInfusion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdditionalDosingParams {
    pub duration: Option<f64>, // For infusions
    pub lag_time: Option<f64>, // For oral dosing
    pub bioavailability: Option<f64>, // For oral dosing
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationConfig {
    pub demographics: DemographicsConfig,
    pub covariates: Option<HashMap<String, CovariateConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemographicsConfig {
    pub weight_mean: f64,
    pub weight_sd: f64,
    pub age_mean: f64,
    pub age_sd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CovariateConfig {
    pub effect: f64,          // Covariate effect
    pub reference: f64,       // Reference value
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub time_points: Vec<f64>,
    pub sigma: f64,           // Residual variability (SD)
    pub integration_method: IntegrationMethod,
    pub tolerance: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IntegrationMethod {
    Analytical,
    Rk4,
    Euler,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> PKResult<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }
    
    pub fn validate(&self) -> PKResult<()> {
        // Validate compartments
        if ![1, 2, 3].contains(&self.model.compartments) {
            return Err(PKError::InvalidModel(
                "Number of compartments must be 1, 2, or 3".to_string()
            ));
        }
        
        // Validate required parameters based on compartments
        self.validate_model_parameters()?;
        
        // Validate dosing
        self.validate_dosing()?;
        
        // Validate simulation parameters
        if self.simulation.time_points.is_empty() {
            return Err(PKError::Validation(
                "At least one time point must be specified".to_string()
            ));
        }
        
        Ok(())
    }
    
    fn validate_model_parameters(&self) -> PKResult<()> {
        let required_params = match self.model.compartments {
            1 => vec!["CL", "V"],
            2 => vec!["CL", "V1", "Q", "V2"],
            3 => vec!["CL", "V1", "Q2", "V2", "Q3", "V3"],
            _ => return Err(PKError::InvalidModel("Invalid compartment number".to_string())),
        };
        
        // Add KA for oral dosing
        let mut all_params = required_params;
        if matches!(self.dosing.route, DosingRoute::Oral) {
            all_params.push("KA");
        }
        
        for param in all_params {
            if !self.model.parameters.contains_key(param) {
                return Err(PKError::InvalidModel(
                    format!("Missing required parameter: {}", param)
                ));
            }
            
            let param_config = &self.model.parameters[param];
            if param_config.theta <= 0.0 {
                return Err(PKError::Validation(
                    format!("Parameter {} must be positive", param)
                ));
            }
        }
        
        Ok(())
    }
    
    fn validate_dosing(&self) -> PKResult<()> {
        if self.dosing.amount <= 0.0 {
            return Err(PKError::InvalidDosing(
                "Dose amount must be positive".to_string()
            ));
        }
        
        if self.dosing.times.is_empty() {
            return Err(PKError::InvalidDosing(
                "At least one dosing time must be specified".to_string()
            ));
        }
        
        // Validate route-specific parameters
        match self.dosing.route {
            DosingRoute::IvInfusion => {
                if self.dosing.additional.as_ref()
                    .and_then(|a| a.duration)
                    .unwrap_or(0.0) <= 0.0 {
                    return Err(PKError::InvalidDosing(
                        "Infusion duration must be specified and positive".to_string()
                    ));
                }
            },
            _ => {}
        }
        
        Ok(())
    }
}