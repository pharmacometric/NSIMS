use super::{PKModel, DoseEvent, DoseRoute, ModelParameters};
use crate::error::{PKError, PKResult};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct OneCompartmentModel {
    params: ModelParameters,
}

impl OneCompartmentModel {
    pub fn new() -> Self {
        Self {
            params: ModelParameters::new(1),
        }
    }
    
    fn calculate_oral_concentration(&self, time: f64, dose_events: &[DoseEvent]) -> f64 {
        let ka = self.params.ka.unwrap_or(1.0);
        let ke = self.params.cl / self.params.v1;
        let mut concentration = 0.0;
        
        for dose in dose_events {
            if dose.time <= time {
                let t = time - dose.time;
                let lag_time = 0.0; // Can be extended for lag time
                
                if t >= lag_time {
                    let t_adj = t - lag_time;
                    let bioavailability = 1.0; // Can be extended for F
                    
                    if (ka - ke).abs() > 1e-10 {
                        // Standard solution
                        let conc_contrib = (dose.amount * bioavailability * ka / self.params.v1) *
                            ((-ke * t_adj).exp() - (-ka * t_adj).exp()) / (ka - ke);
                        concentration += conc_contrib;
                    } else {
                        // Flip-flop kinetics (ka ≈ ke)
                        let conc_contrib = (dose.amount * bioavailability / self.params.v1) *
                            t_adj * (-ke * t_adj).exp();
                        concentration += conc_contrib;
                    }
                }
            }
        }
        
        concentration.max(0.0)
    }
    
    fn calculate_iv_concentration(&self, time: f64, dose_events: &[DoseEvent]) -> f64 {
        let ke = self.params.cl / self.params.v1;
        let mut concentration = 0.0;
        
        for dose in dose_events {
            if dose.time <= time {
                let t = time - dose.time;
                
                match dose.route {
                    DoseRoute::IvBolus => {
                        let conc_contrib = (dose.amount / self.params.v1) * (-ke * t).exp();
                        concentration += conc_contrib;
                    },
                    DoseRoute::IvInfusion => {
                        let duration = dose.duration.unwrap_or(1.0);
                        let rate = dose.amount / duration;
                        
                        if t <= duration {
                            // During infusion
                            let conc_contrib = (rate / self.params.cl) * (1.0 - (-ke * t).exp());
                            concentration += conc_contrib;
                        } else {
                            // After infusion
                            let conc_end = (rate / self.params.cl) * (1.0 - (-ke * duration).exp());
                            let conc_contrib = conc_end * (-ke * (t - duration)).exp();
                            concentration += conc_contrib;
                        }
                    },
                    _ => {}
                }
            }
        }
        
        concentration.max(0.0)
    }
}

impl PKModel for OneCompartmentModel {
    fn calculate_concentration(&self, time: f64, dose_history: &[DoseEvent]) -> PKResult<f64> {
        if dose_history.is_empty() {
            return Ok(0.0);
        }
        
        let concentration = match dose_history[0].route {
            DoseRoute::Oral => self.calculate_oral_concentration(time, dose_history),
            DoseRoute::IvBolus | DoseRoute::IvInfusion => 
                self.calculate_iv_concentration(time, dose_history),
        };
        
        Ok(concentration)
    }
    
    fn get_parameter_names(&self) -> Vec<&'static str> {
        vec!["CL", "V"]
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
                "V" | "V1" => {
                    if value <= 0.0 {
                        return Err(PKError::Validation("V must be positive".to_string()));
                    }
                    self.params.v1 = value;
                },
                "KA" => {
                    if value <= 0.0 {
                        return Err(PKError::Validation("KA must be positive".to_string()));
                    }
                    self.params.ka = Some(value);
                },
                _ => return Err(PKError::InvalidModel(
                    format!("Unknown parameter for 1-compartment model: {}", name)
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
    fn test_one_compartment_iv_bolus() {
        let mut model = OneCompartmentModel::new();
        let mut params = HashMap::new();
        params.insert("CL".to_string(), 2.0);
        params.insert("V".to_string(), 10.0);
        model.set_parameters(&params).unwrap();
        
        let dose = DoseEvent {
            time: 0.0,
            amount: 100.0,
            route: DoseRoute::IvBolus,
            duration: None,
        };
        
        let conc_0 = model.calculate_concentration(0.0, &[dose.clone()]).unwrap();
        assert_relative_eq!(conc_0, 10.0, epsilon = 1e-6);
        
        let conc_5 = model.calculate_concentration(5.0, &[dose]).unwrap();
        let expected = 10.0 * (-0.2 * 5.0).exp(); // ke = CL/V = 0.2
        assert_relative_eq!(conc_5, expected, epsilon = 1e-6);
    }
    
    #[test]
    fn test_one_compartment_oral() {
        let mut model = OneCompartmentModel::new();
        let mut params = HashMap::new();
        params.insert("CL".to_string(), 2.0);
        params.insert("V".to_string(), 10.0);
        params.insert("KA".to_string(), 1.0);
        model.set_parameters(&params).unwrap();
        
        let dose = DoseEvent {
            time: 0.0,
            amount: 100.0,
            route: DoseRoute::Oral,
            duration: None,
        };
        
        let conc_1 = model.calculate_concentration(1.0, &[dose]).unwrap();
        let ke = 0.2;
        let ka = 1.0;
        let expected = (100.0 * ka / 10.0) * ((-ke).exp() - (-ka).exp()) / (ka - ke);
        assert_relative_eq!(conc_1, expected, epsilon = 1e-6);
    }
}