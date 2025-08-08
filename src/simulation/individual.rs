use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientResult {
    pub patient_id: usize,
    pub demographics: Demographics,
    pub parameters: HashMap<String, f64>,
    pub observations: Vec<Observation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Demographics {
    pub weight: f64,
    pub age: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub time: f64,
    pub concentration: f64,
    pub predicted_concentration: f64,
}

impl PatientResult {
    pub fn get_max_concentration(&self) -> f64 {
        self.observations.iter()
            .map(|obs| obs.concentration)
            .fold(0.0, f64::max)
    }
    
    pub fn get_auc(&self) -> f64 {
        // Simple trapezoidal rule for AUC calculation
        let mut auc = 0.0;
        
        for window in self.observations.windows(2) {
            let dt = window[1].time - window[0].time;
            let avg_conc = (window[0].concentration + window[1].concentration) / 2.0;
            auc += dt * avg_conc;
        }
        
        auc
    }
    
    pub fn get_time_to_max(&self) -> Option<f64> {
        self.observations.iter()
            .max_by(|a, b| a.concentration.partial_cmp(&b.concentration).unwrap())
            .map(|obs| obs.time)
    }
}