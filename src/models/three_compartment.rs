use super::{PKModel, DoseEvent, DoseRoute, ModelParameters};
use crate::error::{PKError, PKResult};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ThreeCompartmentModel {
    params: ModelParameters,
}

impl ThreeCompartmentModel {
    pub fn new() -> Self {
        Self {
            params: ModelParameters::new(3),
        }
    }
    
    fn calculate_hybrid_constants(&self) -> (f64, f64, f64) {
        let k10 = self.params.cl / self.params.v1;
        let k12 = self.params.q2.unwrap_or(0.0) / self.params.v1;
        let k21 = self.params.q2.unwrap_or(0.0) / self.params.v2.unwrap_or(1.0);
        let k13 = self.params.q3.unwrap_or(0.0) / self.params.v1;
        let k31 = self.params.q3.unwrap_or(0.0) / self.params.v3.unwrap_or(1.0);
        
        // For 3-compartment model, we need to solve cubic equation
        // Simplified approach using numerical methods or approximations
        let a = k10 + k12 + k21 + k13 + k31;
        let b = k10 * (k21 + k31) + k12 * k31 + k13 * k21;
        let c = k10 * k21 * k31;
        
        // Approximate solution for the three exponential terms
        // This is a simplified version - full implementation would solve cubic
        let alpha = a / 3.0 + ((a*a - 3.0*b)/9.0).sqrt();
        let beta = a / 3.0;
        let gamma = a / 3.0 - ((a*a - 3.0*b)/9.0).sqrt();
        
        (alpha, beta, gamma)
    }
    
    fn calculate_iv_bolus_concentration(&self, time: f64, dose_events: &[DoseEvent]) -> f64 {
        let (alpha, beta, gamma) = self.calculate_hybrid_constants();
        let mut concentration = 0.0;
        
        for dose in dose_events {
            if dose.time <= time && dose.route == DoseRoute::IvBolus {
                let t = time - dose.time;
                
                // Simplified coefficients (in practice, these would be derived from
                // the full solution of the 3-compartment differential equations)
                let a_coeff = 0.4;
                let b_coeff = 0.4;
                let c_coeff = 0.2;
                
                let conc_contrib = (dose.amount / self.params.v1) * 
                    (a_coeff * (-alpha * t).exp() + 
                     b_coeff * (-beta * t).exp() + 
                     c_coeff * (-gamma * t).exp());
                
                concentration += conc_contrib;
            }
        }
        
        concentration.max(0.0)
    }
    
    fn calculate_iv_infusion_concentration(&self, time: f64, dose_events: &[DoseEvent]) -> f64 {
        let (alpha, beta, gamma) = self.calculate_hybrid_constants();
        let mut concentration = 0.0;
        
        for dose in dose_events {
            if dose.time <= time && dose.route == DoseRoute::IvInfusion {
                let t = time - dose.time;
                let duration = dose.duration.unwrap_or(1.0);
                let rate = dose.amount / duration;
                
                let a_coeff = 0.4;
                let b_coeff = 0.4;
                let c_coeff = 0.2;
                
                if t <= duration {
                    // During infusion
                    let term1 = a_coeff * (1.0 - (-alpha * t).exp()) / alpha;
                    let term2 = b_coeff * (1.0 - (-beta * t).exp()) / beta;
                    let term3 = c_coeff * (1.0 - (-gamma * t).exp()) / gamma;
                    let conc_contrib = (rate / self.params.v1) * (term1 + term2 + term3);
                    concentration += conc_contrib;
                } else {
                    // After infusion
                    let term1_end = a_coeff * (1.0 - (-alpha * duration).exp()) / alpha;
                    let term2_end = b_coeff * (1.0 - (-beta * duration).exp()) / beta;
                    let term3_end = c_coeff * (1.0 - (-gamma * duration).exp()) / gamma;
                    let conc_end = (rate / self.params.v1) * (term1_end + term2_end + term3_end);
                    
                    let decay_term1 = a_coeff * (-alpha * (t - duration)).exp();
                    let decay_term2 = b_coeff * (-beta * (t - duration)).exp();
                    let decay_term3 = c_coeff * (-gamma * (t - duration)).exp();
                    let conc_contrib = conc_end * (decay_term1 + decay_term2 + decay_term3);
                    concentration += conc_contrib;
                }
            }
        }
        
        concentration.max(0.0)
    }
    
    fn calculate_oral_concentration(&self, time: f64, dose_events: &[DoseEvent]) -> f64 {
        let ka = self.params.ka.unwrap_or(1.0);
        let (alpha, beta, gamma) = self.calculate_hybrid_constants();
        let mut concentration = 0.0;
        
        for dose in dose_events {
            if dose.time <= time && dose.route == DoseRoute::Oral {
                let t = time - dose.time;
                let bioavailability = 1.0;
                
                let a_coeff = 0.4;
                let b_coeff = 0.4;
                let c_coeff = 0.2;
                
                let term_ka = ka * bioavailability * dose.amount / self.params.v1;
                
                let term1 = a_coeff * (-alpha * t).exp() / (ka - alpha);
                let term2 = b_coeff * (-beta * t).exp() / (ka - beta);
                let term3 = c_coeff * (-gamma * t).exp() / (ka - gamma);
                let term4 = (-ka * t).exp() / ((alpha - ka) * (beta - ka) * (gamma - ka));
                
                let conc_contrib = term_ka * (term1 + term2 + term3 + term4);
                concentration += conc_contrib;
            }
        }
        
        concentration.max(0.0)
    }
}

impl PKModel for ThreeCompartmentModel {
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
        vec!["CL", "V1", "Q2", "V2", "Q3", "V3"]
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
                "Q2" => {
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
                "Q3" => {
                    if value <= 0.0 {
                        return Err(PKError::Validation("Q3 must be positive".to_string()));
                    }
                    self.params.q3 = Some(value);
                },
                "V3" => {
                    if value <= 0.0 {
                        return Err(PKError::Validation("V3 must be positive".to_string()));
                    }
                    self.params.v3 = Some(value);
                },
                "KA" => {
                    if value <= 0.0 {
                        return Err(PKError::Validation("KA must be positive".to_string()));
                    }
                    self.params.ka = Some(value);
                },
                _ => return Err(PKError::InvalidModel(
                    format!("Unknown parameter for 3-compartment model: {}", name)
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
    fn test_three_compartment_iv_bolus() {
        let mut model = ThreeCompartmentModel::new();
        let mut params = HashMap::new();
        params.insert("CL".to_string(), 2.0);
        params.insert("V1".to_string(), 10.0);
        params.insert("Q2".to_string(), 1.0);
        params.insert("V2".to_string(), 5.0);
        params.insert("Q3".to_string(), 0.5);
        params.insert("V3".to_string(), 3.0);
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