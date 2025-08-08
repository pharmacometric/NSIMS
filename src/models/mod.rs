pub mod one_compartment;
pub mod two_compartment;
pub mod three_compartment;

use crate::error::{PKError, PKResult};
use crate::config::ModelConfig;
use std::collections::HashMap;

pub trait PKModel {
    fn calculate_concentration(&self, time: f64, dose_history: &[DoseEvent]) -> PKResult<f64>;
    fn get_parameter_names(&self) -> Vec<&'static str>;
    fn set_parameters(&mut self, params: &HashMap<String, f64>) -> PKResult<()>;
}

#[derive(Debug, Clone)]
pub struct DoseEvent {
    pub time: f64,
    pub amount: f64,
    pub route: DoseRoute,
    pub duration: Option<f64>, // For infusions
}

#[derive(Debug, Clone, PartialEq)]
pub enum DoseRoute {
    Oral,
    IvBolus,
    IvInfusion,
}

#[derive(Debug, Clone)]
pub struct ModelParameters {
    pub cl: f64,    // Clearance
    pub v1: f64,    // Central volume
    pub ka: Option<f64>,  // Absorption rate constant (oral only)
    pub q2: Option<f64>,  // Inter-compartmental clearance 1->2
    pub v2: Option<f64>,  // Peripheral volume 2
    pub q3: Option<f64>,  // Inter-compartmental clearance 1->3
    pub v3: Option<f64>,  // Peripheral volume 3
}

impl ModelParameters {
    pub fn new(compartments: u8) -> Self {
        Self {
            cl: 1.0,
            v1: 10.0,
            ka: None,
            q2: if compartments >= 2 { Some(0.5) } else { None },
            v2: if compartments >= 2 { Some(5.0) } else { None },
            q3: if compartments >= 3 { Some(0.2) } else { None },
            v3: if compartments >= 3 { Some(2.0) } else { None },
        }
    }
    
    pub fn from_config(config: &ModelConfig) -> PKResult<Self> {
        let mut params = Self::new(config.compartments);
        
        for (name, param_config) in &config.parameters {
            match name.as_str() {
                "CL" => params.cl = param_config.theta,
                "V" | "V1" => params.v1 = param_config.theta,
                "KA" => params.ka = Some(param_config.theta),
                "Q" | "Q2" => params.q2 = Some(param_config.theta),
                "V2" => params.v2 = Some(param_config.theta),
                "Q3" => params.q3 = Some(param_config.theta),
                "V3" => params.v3 = Some(param_config.theta),
                _ => return Err(PKError::InvalidModel(
                    format!("Unknown parameter: {}", name)
                )),
            }
        }
        
        Ok(params)
    }
}

pub fn create_model(compartments: u8) -> PKResult<Box<dyn PKModel>> {
    match compartments {
        1 => Ok(Box::new(one_compartment::OneCompartmentModel::new())),
        2 => Ok(Box::new(two_compartment::TwoCompartmentModel::new())),
        3 => Ok(Box::new(three_compartment::ThreeCompartmentModel::new())),
        _ => Err(PKError::InvalidModel(
            format!("Unsupported number of compartments: {}", compartments)
        )),
    }
}