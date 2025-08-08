pub mod population;
pub mod individual;
pub mod variability;

use crate::config::Config;
use crate::models::{create_model, PKModel};
use crate::dosing::DosingRegimen;
use crate::error::{PKError, PKResult};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use rand_distr::Normal;
use log::{info, debug};

pub use population::*;
pub use individual::*;
pub use variability::*;

pub struct Simulator {
    config: Config,
    rng: StdRng,
}

impl Simulator {
    pub fn new(config: Config, seed: Option<u64>) -> PKResult<Self> {
        let rng = match seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::from_entropy(),
        };
        
        Ok(Self { config, rng })
    }
    
    pub fn simulate_population(&mut self, n_patients: usize) -> PKResult<Vec<PatientResult>> {
        info!("Starting population simulation for {} patients", n_patients);
        
        // Clone the dosing config to avoid borrowing conflicts
        let dosing_config = self.config.dosing.clone();
        let dosing_regimen = DosingRegimen::from_config(&dosing_config)?;
        
        let mut results = Vec::with_capacity(n_patients);

        for patient_id in 1..=n_patients {
            if patient_id % 10 == 0 || patient_id <= 10 {
                info!("Simulating patient {}/{}", patient_id, n_patients);
            }
            
            let patient_result = self.simulate_individual(patient_id, &dosing_regimen)?;
            results.push(patient_result);
        }
        
        info!("Population simulation completed");
        Ok(results)
    }
    
    fn simulate_individual(&mut self, patient_id: usize, dosing_regimen: &DosingRegimen) -> PKResult<PatientResult> {
        debug!("Simulating patient {}", patient_id);
        
        // Clone necessary config values to avoid borrowing conflicts
        let model_compartments = self.config.model.compartments;
        let time_points = self.config.simulation.time_points.clone();
        
        let individual_params = self.generate_individual_parameters()?;
        let mut model = create_model(model_compartments)?;
        model.set_parameters(&individual_params)?;
        let demographics = self.generate_demographics()?;
        
        let mut observations = Vec::new();
        for &time in &time_points {
            let dose_history = dosing_regimen.get_events_before(time);
            let predicted_conc = model.calculate_concentration(time, &dose_history)?;
            let observed_conc = self.add_residual_variability(predicted_conc)?;
            
            observations.push(Observation {
                time,
                concentration: observed_conc,
                predicted_concentration: predicted_conc,
            });
        }
        
        Ok(PatientResult {
            patient_id,
            demographics,
            parameters: individual_params,
            observations,
        })
    }
    
    fn generate_individual_parameters(&mut self) -> PKResult<std::collections::HashMap<String, f64>> {
        let mut params = std::collections::HashMap::new();
        
        // Clone the parameters to avoid borrowing conflicts
        let model_parameters = self.config.model.parameters.clone();
        
        for (name, param_config) in &model_parameters {
            let mut value = param_config.theta;
            
            if let Some(omega) = param_config.omega {
                let omega_sd = omega / 100.0;
                let eta: f64 = self.rng.sample(rand_distr::Normal::new(0.0, omega_sd)
                    .map_err(|_| PKError::Random)?);
                value *= eta.exp();
            }
            
            if let Some((lower, upper)) = param_config.bounds {
                value = value.max(lower).min(upper);
            }
            
            params.insert(name.clone(), value);
        }
        
        Ok(params)
    }
    
    fn generate_demographics(&mut self) -> PKResult<Demographics> {
        // Clone demographic config to avoid borrowing conflicts
        let demo_config = self.config.population.demographics.clone();
        
        let weight = self.rng.sample(Normal::new(
            demo_config.weight_mean,
            demo_config.weight_sd,
        ).map_err(|_| PKError::Random)?);
        
        let age = self.rng.sample(Normal::new(
            demo_config.age_mean,
            demo_config.age_sd,
        ).map_err(|_| PKError::Random)?);
        
        Ok(Demographics {
            weight: weight.max(30.0).min(200.0),
            age: age.max(18.0).min(100.0),
        })
    }
    
    fn add_residual_variability(&mut self, predicted: f64) -> PKResult<f64> {
        if predicted <= 0.0 {
            return Ok(0.0);
        }
        
        // Clone sigma to avoid borrowing conflicts
        let sigma = self.config.simulation.sigma;
        
        let epsilon: f64 = self.rng.sample(Normal::new(0.0, sigma)
            .map_err(|_| PKError::Random)?);
        let observed = predicted * (1.0 + epsilon);
        
        Ok(observed.max(0.0))
    }
}