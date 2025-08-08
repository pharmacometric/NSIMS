use super::{PKModel, DoseEvent, DoseRoute, ModelParameters};
use crate::error::{PKError, PKResult};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TwoCompartmentModel {
    params: ModelParameters,
}

impl TwoCompartmentModel {
    pub fn new() -> Self {
        Self {
            params: ModelParameters::new(2),
        }
    }
    
    fn calculate_hybrid_constants(&self) -> (f64, f64, f64) {
        let k10 = self.params.cl / self.params.v1;
        let k12 = self.params.q2.unwrap_or(0.0) / self.params.v1;
        let k21 = self.params.q2.unwrap_or(0.0) / self.params.v2.unwrap_or(1.0);
        
        let a = k10 + k12 + k21;
        let b = k10 * k21;
        
        let discriminant = a * a - 4.0 * b;
        let sqrt_disc = discriminant.sqrt();
        
        let alpha = (a + sqrt_disc) / 2.0;
        let beta = (a - sqrt_disc) / 2.0;
        
        (alpha, beta, k21)
    }
    
    fn calculate_iv_bolus_concentration(&self, time: f64, dose_events: &[DoseEvent]) -> f64 {
        let (alpha, beta, k21) = self.calculate_hybrid_constants();
        let mut concentration = 0.0;
        
        for dose in dose_events {
            if dose.time <= time && dose.route == DoseRoute::IvBolus {
                let t = time - dose.time;
                
                let a_coeff = (alpha - k21) / (alpha - beta);
                let b_coeff = (k21 - beta) / (alpha - beta);
                
                let conc_contrib = (dose.amount / self.params.v1) * 
                    (a_coeff * (-alpha * t).exp() + b_coeff * (-beta * t).exp());
                
                concentration += conc_contrib;
            }
        }
        
        concentration.max(0.0)
    }
    
    fn calculate_iv_infusion_concentration(&self, time: f64, dose_events: &[DoseEvent]) -> f64 {
        let (alpha, beta, k21) = self.calculate_hybrid_constants();
        let mut concentration = 0.0;
        
        for dose in dose_events {
            if dose.time <= time && dose.route == DoseRoute::IvInfusion {
                let t = time - dose.time;
                let duration = dose.duration.unwrap_or(1.0);
                let rate = dose.amount / duration;
                
                let a_coeff = (alpha - k21) / (alpha - beta);
                let b_coeff = (k21 - beta) / (alpha - beta);
                
                if t <= duration {
                    // During infusion
                    let term1 = a_coeff * (1.0 - (-alpha * t).exp()) / alpha;
                    let term2 = b_coeff * (1.0 - (-beta * t).exp()) / beta;
                    let conc_contrib = (rate / self.params.v1) * (term1 + term2);
                    concentration += conc_contrib;
                } else {
                    // After infusion
                    let term1_end = a_coeff * (1.0 - (-alpha * duration).exp()) / alpha;
                    let term2_end = b_coeff * (1.0 - (-beta * duration).exp()) / beta;
                    let conc_end = (rate / self.params.v1) * (term1_end + term2_end);
                    
                    let decay_term1 = a_coeff * (-alpha * (t - duration)).exp();
                    let decay_term2 = b_coeff * (-beta * (t - duration)).exp();
                    let conc_contrib = conc_end * (decay_term1 + decay_term2);
                    concentration += conc_contrib;
                }
            }
        }
        
        concentration.max(0.0)
    }
    
    fn calculate_oral_concentration(&self, time: f64, dose_events: &[DoseEvent]) -> f64 {
        let ka = self.params.ka.unwrap_or(1.0);
        let (alpha, beta, k21) = self.calculate_hybrid_constants();
        let mut concentration = 0.0;
        
        for dose in dose_events {
            if dose.time <= time && dose.route == DoseRoute::Oral {
                let t = time - dose.time;
                let bioavailability = 1.0;
                
                let a_coeff = (alpha - k21) / (alpha - beta);
                let b_coeff = (k21 - beta) / (alpha - beta);
                
                let term_ka = ka * bioavailability * dose.amount / self.params.v1;
                
                let term1 = a_coeff * (-alpha * t).exp() / (ka - alpha);
                let term2 = b_coeff * (-beta * t).exp() / (ka - beta);
                let term3 = (-ka * t).exp() / ((alpha - ka) * (beta - ka));
                
                let conc_contrib = term_ka * (term1 + term2 + term3);
                concentration += conc_contrib;
            }
        }
        
        concentration.max(0.0)
    }
}

impl PKModel for TwoCompartmentModel {
    fn calculate_concentration(&self, time: f64, dose_history: &[DoseEvent]) -> PKResult<f64> {
        if dose_history.is_empty() {
            return Ok(0.0);
        }
        
        let mut total_concentration = 0.0;
        
        // Group doses by route
        let oral_doses: Vec<_> = dose_history.iter()
            .filter(|d| d.route == DoseRoute::Oral)
            .cloned()
            .collect();
        let iv_bolus_doses: Vec<_> = dose_history.iter()
            .filter(|d| d.route == DoseRoute::IvBolus)
            .cloned()
            .collect();
        let iv_infusion_doses: Vec<_> = dose_history.iter()
            .filter(|d| d.route == DoseRoute::IvInfusion)
            .cloned()
            .collect();
        
        if !oral_doses.is_empty() {
            total_concentration += self.calculate_oral_concentration(time, &oral_doses);
        }
        if !iv_bolus_doses.is_empty() {
            total_concentration += self.calculate_iv_bolus_concentration(time, &iv_bolus_doses);
        }
        if !iv_infusion_doses.is_empty() {
            total_concentration += self.calculate_iv_infusion_concentration(time, &iv_infusion_doses);
        }
        
        Ok(total_concentration)
    }
    
    fn get_parameter_names(&self) -> Vec<&'static str> {
        vec!["CL", "V1", "Q2", "V2"]
    }
    
    fn set_parameters(&mut self, params: &HashMap<String, f64>) -> PKResult<()> {
        for (name, &value) in params {
            match name.as_str() {
                "CL" => {
                    if value <= 0.0 {
                        return Err(PKError::Validation("CL must be positive".to_string()));
                    }
                    self.params.cl = value;
                },
                "V1" => {
                    if value <= 0.0 {
                        return Err(PKError::Validation("V1 must be positive".to_string()));
                    }
                    self.params.v1 = value;
                },
                "Q2" | "Q" => {
                    if value <= 0.0 {
                        return Err(PKError::Validation("Q2 must be positive".to_string()));
                    }
                    self.params.q2 = Some(value);
                },
                "V2" => {
                    if value <= 0.0 {
                        return Err(PKError::Validation("V2 must be positive".to_string()));
                    }
                    self.params.v2 = Some(value);
                },
                "KA" => {
                    if value <= 0.0 {
                        return Err(PKError::Validation("KA must be positive".to_string()));
                    }
                    self.params.ka = Some(value);
                },
                _ => return Err(PKError::InvalidModel(
                    format!("Unknown parameter for 2-compartment model: {}", name)
                )),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    
    #[test]
    fn test_two_compartment_iv_bolus() {
        let mut model = TwoCompartmentModel::new();
        let mut params = HashMap::new();
        params.insert("CL".to_string(), 2.0);
        params.insert("V1".to_string(), 10.0);
        params.insert("Q2".to_string(), 1.0);
        params.insert("V2".to_string(), 5.0);
        model.set_parameters(&params).unwrap();
        
        let dose = DoseEvent {
            time: 0.0,
            amount: 100.0,
            route: DoseRoute::IvBolus,
            duration: None,
        };
        
        let conc_0 = model.calculate_concentration(0.0, &[dose.clone()]).unwrap();
        assert_relative_eq!(conc_0, 10.0, epsilon = 1e-6);
        
        // Test that concentration decreases over time
        let conc_1 = model.calculate_concentration(1.0, &[dose.clone()]).unwrap();
        let conc_5 = model.calculate_concentration(5.0, &[dose]).unwrap();
        assert!(conc_1 > conc_5);
        assert!(conc_5 > 0.0);
    }
}