pub mod population;
pub mod individual;
pub mod variability;
use crate::config::{ErrorModel,CovariateModel,Config};
use crate::models::{create_model, PKModel};
use crate::dosing::DosingRegimen;
use crate::error::{PKError, PKResult};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
// Corrected: Import the Distribution trait
use rand_distr::{Normal, Distribution};
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
        
        let (demographics, individual_params) = self.generate_individual_parameters()?;
        
        let mut model = create_model(model_compartments)?;
        model.set_parameters(&individual_params)?;
        
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

    fn generate_individual_parameters(&mut self) -> PKResult<(Demographics, std::collections::HashMap<String, f64>)> {
        let mut params = std::collections::HashMap::new();
        
        // Clone the parameters to avoid borrowing conflicts
        let model_parameters = self.config.model.parameters.clone();
        let demographics = self.generate_demographics()?;
        
        for (name, param_config) in &model_parameters {
            let mut value = param_config.theta;
            
            // Apply covariate effects
            value = self.apply_covariate_effects(value, name, &demographics);
            
            if let Some(omega) = param_config.omega {
                let omega_sd = omega / 100.0;
                let normal_dist = Normal::new(0.0, omega_sd).map_err(|_| PKError::Random)?;
                let eta: f64 = self.rng.sample(normal_dist); // This now works
                value *= eta.exp();
            }
            
            if let Some((lower, upper)) = param_config.bounds {
                value = value.max(lower).min(upper);
            }
            
            params.insert(name.clone(), value);
        }
        
        Ok((demographics, params))
    }
    
    fn generate_demographics(&mut self) -> PKResult<Demographics> {
        // Clone demographic config to avoid borrowing conflicts
        let demo_config = self.config.population.demographics.clone();
        
        let weight_dist = Normal::new(demo_config.weight_mean, demo_config.weight_sd)
            .map_err(|_| PKError::Random)?;
        let weight = self.rng.sample(weight_dist); // This now works
        
        let age_dist = Normal::new(demo_config.age_mean, demo_config.age_sd)
            .map_err(|_| PKError::Random)?;
        let age = self.rng.sample(age_dist); // This now works
        
        Ok(Demographics {
            weight: weight.max(30.0).min(200.0),
            age: age.max(18.0).min(100.0),
        })
    }

    fn add_residual_variability(&mut self, predicted: f64) -> PKResult<f64> {
        match &self.config.simulation.error_model {
            ErrorModel::Proportional { sigma } => {
                apply_proportional_error(predicted, *sigma, &mut self.rng)
            },
            ErrorModel::Additive { sigma } => {
                if predicted <= 0.0 {
                    return Ok(0.0);
                }
                let normal = Normal::new(0.0, *sigma).map_err(|_| PKError::Random)?;
                let epsilon = normal.sample(&mut self.rng); // This now works
                Ok((predicted + epsilon).max(0.0))
            },
            ErrorModel::Combined { sigma_prop, sigma_add } => {
                apply_combined_error(predicted, *sigma_add, *sigma_prop, &mut self.rng)
            },
        }
    }
    
    fn apply_covariate_effects(&self, base_value: f64, param_name: &str, demographics: &Demographics) -> f64 {
        let mut value = base_value;
        
        if let Some(covariates) = &self.config.population.covariates {
            // Weight effect
            if let Some(wt_config) = covariates.get(&format!("{}_WT", param_name)) {
                value *= self.apply_covariate_effect(
                    demographics.weight, 
                    wt_config.reference, 
                    wt_config.effect, 
                    &wt_config.model
                );
            }
            
            // Age effect
            if let Some(age_config) = covariates.get(&format!("{}_AGE", param_name)) {
                value *= self.apply_covariate_effect(
                    demographics.age, 
                    age_config.reference, 
                    age_config.effect, 
                    &age_config.model
                );
            }
        }
        
        value
    }
    
    fn apply_covariate_effect(&self, covariate_value: f64, reference: f64, effect: f64, model: &CovariateModel) -> f64 {
        match model {
            CovariateModel::Power => {
                (covariate_value / reference).powf(effect)
            },
            CovariateModel::Exponential => {
                (effect * (covariate_value - reference)).exp()
            },
            CovariateModel::Linear => {
                1.0 + effect * (covariate_value - reference)
            },
        }
    }
}