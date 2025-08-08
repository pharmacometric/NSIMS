use std::path::Path;
use std::collections::HashMap;
use crate::config::*;
use crate::error::{PKError, PKResult};

pub fn parse_control_stream<P: AsRef<Path>>(path: P) -> PKResult<Config> {
    let content = std::fs::read_to_string(path)?;
    let mut parser = ControlStreamParser::new(&content);
    parser.parse()
}

struct ControlStreamParser {
    lines: Vec<String>,
    current_line: usize,
}

impl ControlStreamParser {
    fn new(content: &str) -> Self {
        let lines: Vec<String> = content
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty() && !line.starts_with(';'))
            .collect();
        
        Self {
            lines,
            current_line: 0,
        }
    }
    
    fn parse(&mut self) -> PKResult<Config> {
        let mut model_config = None;
        let mut dosing_config = None;
        let mut population_config = None;
        let mut simulation_config = None;
        
        while self.current_line < self.lines.len() {
            let line = &self.lines[self.current_line];
            
            if line.starts_with("$PROBLEM") {
                self.current_line += 1;
                continue;
            } else if line.starts_with("$INPUT") {
                self.current_line += 1;
                continue;
            } else if line.starts_with("$DATA") {
                self.current_line += 1;
                continue;
            } else if line.starts_with("$SUBROUTINES") || line.starts_with("$SUBROUTINE") {
                model_config = Some(self.parse_subroutines()?);
            } else if line.starts_with("$PK") {
                if model_config.is_none() {
                    return Err(PKError::InvalidModel(
                        "$SUBROUTINES block must come before $PK".to_string()
                    ));
                }
                self.parse_pk_block(model_config.as_mut().unwrap())?;
            } else if line.starts_with("$THETA") {
                if model_config.is_none() {
                    return Err(PKError::InvalidModel(
                        "$SUBROUTINES block must come before $THETA".to_string()
                    ));
                }
                self.parse_theta_block(model_config.as_mut().unwrap())?;
            } else if line.starts_with("$OMEGA") {
                if model_config.is_none() {
                    return Err(PKError::InvalidModel(
                        "$SUBROUTINES block must come before $OMEGA".to_string()
                    ));
                }
                self.parse_omega_block(model_config.as_mut().unwrap())?;
            } else if line.starts_with("$SIGMA") {
                simulation_config = Some(self.parse_sigma_block()?);
            } else if line.starts_with("$DOSING") {
                dosing_config = Some(self.parse_dosing_block()?);
            } else if line.starts_with("$POPULATION") {
                population_config = Some(self.parse_population_block()?);
            } else if line.starts_with("$SIMULATION") {
                if simulation_config.is_none() {
                    simulation_config = Some(SimulationConfig {
                        time_points: vec![],
                        sigma: 0.1,
                        integration_method: IntegrationMethod::Analytical,
                        tolerance: None,
                    });
                }
                self.parse_simulation_block(simulation_config.as_mut().unwrap())?;
            } else {
                self.current_line += 1;
            }
        }
        
        // Set defaults if not specified
        let model_config = model_config.ok_or_else(|| 
            PKError::InvalidModel("Missing $SUBROUTINES block".to_string()))?;
        
        let dosing_config = dosing_config.unwrap_or_else(|| DosingConfig {
            route: DosingRoute::IvBolus,
            amount: 100.0,
            times: vec![0.0],
            additional: None,
        });
        
        let population_config = population_config.unwrap_or_else(|| PopulationConfig {
            demographics: DemographicsConfig {
                weight_mean: 70.0,
                weight_sd: 15.0,
                age_mean: 45.0,
                age_sd: 12.0,
            },
            covariates: None,
        });
        
        let simulation_config = simulation_config.unwrap_or_else(|| SimulationConfig {
            time_points: vec![0.0, 1.0, 2.0, 4.0, 8.0, 12.0, 24.0],
            sigma: 0.1,
            integration_method: IntegrationMethod::Analytical,
            tolerance: None,
        });
        
        Ok(Config {
            model: model_config,
            dosing: dosing_config,
            population: population_config,
            simulation: simulation_config,
        })
    }
    
    fn parse_subroutines(&mut self) -> PKResult<ModelConfig> {
        let line = &self.lines[self.current_line];
        self.current_line += 1;
        
        let compartments = if line.contains("ADVAN1") {
            1
        } else if line.contains("ADVAN3") {
            2
        } else if line.contains("ADVAN11") {
            3
        } else {
            return Err(PKError::InvalidModel(
                "Unsupported ADVAN subroutine. Use ADVAN1, ADVAN3, or ADVAN11".to_string()
            ));
        };
        
        Ok(ModelConfig {
            compartments,
            parameters: HashMap::new(),
        })
    }
    
    fn parse_pk_block(&mut self, _model_config: &mut ModelConfig) -> PKResult<()> {
        self.current_line += 1;
        
        // Skip PK block content for now - parameters are defined in $THETA
        while self.current_line < self.lines.len() {
            let line = &self.lines[self.current_line];
            if line.starts_with('$') {
                break;
            }
            self.current_line += 1;
        }
        
        Ok(())
    }
    
    fn parse_theta_block(&mut self, model_config: &mut ModelConfig) -> PKResult<()> {
        self.current_line += 1;
        
        let param_names = match model_config.compartments {
            1 => vec!["CL", "V", "KA"],
            2 => vec!["CL", "V1", "Q", "V2", "KA"],
            3 => vec!["CL", "V1", "Q2", "V2", "Q3", "V3", "KA"],
            _ => return Err(PKError::InvalidModel("Invalid compartment number".to_string())),
        };
        
        let mut param_index = 0;
        
        while self.current_line < self.lines.len() && param_index < param_names.len() {
            let line = &self.lines[self.current_line];
            
            if line.starts_with('$') {
                break;
            }
            
            // Parse theta values: (lower, init, upper) or just init
            let theta_value = self.parse_theta_line(line)?;
            
            if param_index < param_names.len() {
                model_config.parameters.insert(
                    param_names[param_index].to_string(),
                    ParameterConfig {
                        theta: theta_value.1,
                        omega: None,
                        bounds: if theta_value.0.is_some() && theta_value.2.is_some() {
                            Some((theta_value.0.unwrap(), theta_value.2.unwrap()))
                        } else {
                            None
                        },
                    }
                );
                param_index += 1;
            }
            
            self.current_line += 1;
        }
        
        Ok(())
    }
    
    fn parse_theta_line(&self, line: &str) -> PKResult<(Option<f64>, f64, Option<f64>)> {
        let cleaned = line.replace("(", "").replace(")", "").replace(",", " ");
        let parts: Vec<&str> = cleaned.split_whitespace().collect();
        
        match parts.len() {
            1 => {
                let value = parts[0].parse::<f64>()
                    .map_err(|_| PKError::Validation(format!("Invalid theta value: {}", parts[0])))?;
                Ok((None, value, None))
            },
            3 => {
                let lower = parts[0].parse::<f64>()
                    .map_err(|_| PKError::Validation(format!("Invalid lower bound: {}", parts[0])))?;
                let init = parts[1].parse::<f64>()
                    .map_err(|_| PKError::Validation(format!("Invalid initial value: {}", parts[1])))?;
                let upper = parts[2].parse::<f64>()
                    .map_err(|_| PKError::Validation(format!("Invalid upper bound: {}", parts[2])))?;
                Ok((Some(lower), init, Some(upper)))
            },
            _ => Err(PKError::Validation(
                format!("Invalid theta specification: {}", line)
            )),
        }
    }
    
    fn parse_omega_block(&mut self, model_config: &mut ModelConfig) -> PKResult<()> {
        self.current_line += 1;
        
        let param_names = match model_config.compartments {
            1 => vec!["CL", "V", "KA"],
            2 => vec!["CL", "V1", "Q", "V2", "KA"],
            3 => vec!["CL", "V1", "Q2", "V2", "Q3", "V3", "KA"],
            _ => return Err(PKError::InvalidModel("Invalid compartment number".to_string())),
        };
        
        let mut param_index = 0;
        
        while self.current_line < self.lines.len() && param_index < param_names.len() {
            let line = &self.lines[self.current_line];
            
            if line.starts_with('$') {
                break;
            }
            
            let omega_value = self.parse_omega_line(line)?;
            
            if param_index < param_names.len() {
                if let Some(param_config) = model_config.parameters.get_mut(param_names[param_index]) {
                    param_config.omega = Some(omega_value);
                }
                param_index += 1;
            }
            
            self.current_line += 1;
        }
        
        Ok(())
    }
    
    fn parse_omega_line(&self, line: &str) -> PKResult<f64> {
        let cleaned = line.replace("(", "").replace(")", "");
        let value = cleaned.trim().parse::<f64>()
            .map_err(|_| PKError::Validation(format!("Invalid omega value: {}", line)))?;
        
        // Convert variance to CV%
        let cv_percent = (value.sqrt()) * 100.0;
        Ok(cv_percent)
    }
    
    fn parse_sigma_block(&mut self) -> PKResult<SimulationConfig> {
        self.current_line += 1;
        
        let line = &self.lines[self.current_line];
        let sigma_value = self.parse_sigma_line(line)?;
        self.current_line += 1;
        
        Ok(SimulationConfig {
            time_points: vec![0.0, 1.0, 2.0, 4.0, 8.0, 12.0, 24.0],
            sigma: sigma_value.sqrt(), // Convert variance to SD
            integration_method: IntegrationMethod::Analytical,
            tolerance: None,
        })
    }
    
    fn parse_sigma_line(&self, line: &str) -> PKResult<f64> {
        let cleaned = line.replace("(", "").replace(")", "");
        let value = cleaned.trim().parse::<f64>()
            .map_err(|_| PKError::Validation(format!("Invalid sigma value: {}", line)))?;
        Ok(value)
    }
    
    fn parse_dosing_block(&mut self) -> PKResult<DosingConfig> {
        self.current_line += 1;
        
        let mut route = DosingRoute::IvBolus;
        let mut amount = 100.0;
        let mut times = vec![0.0];
        let mut duration = None;
        let mut bioavailability = None;
        let mut lag_time = None;
        
        while self.current_line < self.lines.len() {
            let line = &self.lines[self.current_line];
            
            if line.starts_with('$') {
                break;
            }
            
            if line.to_uppercase().contains("ROUTE") {
                if line.to_uppercase().contains("ORAL") {
                    route = DosingRoute::Oral;
                } else if line.to_uppercase().contains("IVBOLUS") {
                    route = DosingRoute::IvBolus;
                } else if line.to_uppercase().contains("INFUSION") {
                    route = DosingRoute::IvInfusion;
                }
            } else if line.to_uppercase().contains("AMOUNT") {
                amount = self.extract_numeric_value(line, "AMOUNT")?;
            } else if line.to_uppercase().contains("TIMES") {
                times = self.extract_time_values(line)?;
            } else if line.to_uppercase().contains("DURATION") {
                duration = Some(self.extract_numeric_value(line, "DURATION")?);
            } else if line.to_uppercase().contains("BIOAVAILABILITY") {
                bioavailability = Some(self.extract_numeric_value(line, "BIOAVAILABILITY")?);
            } else if line.to_uppercase().contains("LAG_TIME") {
                lag_time = Some(self.extract_numeric_value(line, "LAG_TIME")?);
            }
            
            self.current_line += 1;
        }
        
        let additional = if duration.is_some() || bioavailability.is_some() || lag_time.is_some() {
            Some(AdditionalDosingParams {
                duration,
                lag_time,
                bioavailability,
            })
        } else {
            None
        };
        
        Ok(DosingConfig {
            route,
            amount,
            times,
            additional,
        })
    }
    
    fn parse_population_block(&mut self) -> PKResult<PopulationConfig> {
        self.current_line += 1;
        
        let mut weight_mean = 70.0;
        let mut weight_sd = 15.0;
        let mut age_mean = 45.0;
        let mut age_sd = 12.0;
        let mut covariates = HashMap::new();
        
        while self.current_line < self.lines.len() {
            let line = &self.lines[self.current_line];
            
            if line.starts_with('$') {
                break;
            }
            
            if line.to_uppercase().contains("WEIGHT_MEAN") {
                weight_mean = self.extract_numeric_value(line, "WEIGHT_MEAN")?;
            } else if line.to_uppercase().contains("WEIGHT_SD") {
                weight_sd = self.extract_numeric_value(line, "WEIGHT_SD")?;
            } else if line.to_uppercase().contains("AGE_MEAN") {
                age_mean = self.extract_numeric_value(line, "AGE_MEAN")?;
            } else if line.to_uppercase().contains("AGE_SD") {
                age_sd = self.extract_numeric_value(line, "AGE_SD")?;
            } else if line.to_uppercase().contains("COV_") {
                let (param, covariate_config) = self.parse_covariate_line(line)?;
                covariates.insert(param, covariate_config);
            }
            
            self.current_line += 1;
        }
        
        Ok(PopulationConfig {
            demographics: DemographicsConfig {
                weight_mean,
                weight_sd,
                age_mean,
                age_sd,
            },
            covariates: if covariates.is_empty() { None } else { Some(covariates) },
        })
    }
    
    fn parse_simulation_block(&mut self, sim_config: &mut SimulationConfig) -> PKResult<()> {
        self.current_line += 1;
        
        while self.current_line < self.lines.len() {
            let line = &self.lines[self.current_line];
            
            if line.starts_with('$') {
                break;
            }
            
            if line.to_uppercase().contains("TIME_POINTS") {
                sim_config.time_points = self.extract_time_values(line)?;
            } else if line.to_uppercase().contains("METHOD") {
                if line.to_uppercase().contains("RK4") {
                    sim_config.integration_method = IntegrationMethod::Rk4;
                } else if line.to_uppercase().contains("EULER") {
                    sim_config.integration_method = IntegrationMethod::Euler;
                }
            }
            
            self.current_line += 1;
        }
        
        Ok(())
    }
    
    fn extract_numeric_value(&self, line: &str, keyword: &str) -> PKResult<f64> {
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() != 2 {
            return Err(PKError::Validation(
                format!("Invalid {} specification: {}", keyword, line)
            ));
        }
        
        parts[1].trim().parse::<f64>()
            .map_err(|_| PKError::Validation(
                format!("Invalid numeric value for {}: {}", keyword, parts[1])
            ))
    }
    
    fn extract_time_values(&self, line: &str) -> PKResult<Vec<f64>> {
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() != 2 {
            return Err(PKError::Validation(
                format!("Invalid time specification: {}", line)
            ));
        }
        
        let time_str = parts[1].trim();
        let times: Result<Vec<f64>, _> = time_str
            .split(',')
            .map(|s| s.trim().parse::<f64>())
            .collect();
        
        times.map_err(|_| PKError::Validation(
            format!("Invalid time values: {}", time_str)
        ))
    }
    
    fn parse_covariate_line(&self, line: &str) -> PKResult<(String, CovariateConfig)> {
        // Parse lines like "COV_CL_WT_EFFECT = 0.75"
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() != 2 {
            return Err(PKError::Validation(
                format!("Invalid covariate specification: {}", line)
            ));
        }
        
        let key = parts[0].trim().to_uppercase();
        let value = parts[1].trim().parse::<f64>()
            .map_err(|_| PKError::Validation(
                format!("Invalid covariate value: {}", parts[1])
            ))?;
        
        // Extract parameter and covariate from key like "COV_CL_WT_EFFECT"
        let key_parts: Vec<&str> = key.split('_').collect();
        if key_parts.len() < 4 {
            return Err(PKError::Validation(
                format!("Invalid covariate key format: {}", key)
            ));
        }
        
        let param = format!("{}_{}", key_parts[1], key_parts[2]);
        let reference = 70.0; // Default reference value
        
        Ok((param, CovariateConfig {
            effect: value,
            reference,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    
    #[test]
    fn test_parse_simple_control_stream() {
        let content = r#"
$PROBLEM One compartment oral model
$SUBROUTINES ADVAN1 TRANS2
$PK
CL = THETA(1)
V = THETA(2)
KA = THETA(3)
$THETA
(0.1, 2.0, 10.0)  ; CL
(5.0, 15.0, 50.0) ; V
(0.1, 1.5, 5.0)   ; KA
$OMEGA
0.09  ; CL
0.0625 ; V
0.16   ; KA
$SIGMA
0.0225
"#;
        
        let mut parser = ControlStreamParser::new(content);
        let config = parser.parse().unwrap();
        
        assert_eq!(config.model.compartments, 1);
        assert_eq!(config.model.parameters.len(), 3);
        assert_eq!(config.model.parameters["CL"].theta, 2.0);
        assert_eq!(config.model.parameters["V"].theta, 15.0);
        assert_eq!(config.model.parameters["KA"].theta, 1.5);
    }
}