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
            .map(|line| {
                // Remove comments (everything after semicolon)
                let line = if let Some(pos) = line.find(';') {
                    &line[..pos]
                } else {
                    line
                };
                line.trim().to_string()
            })
            .filter(|line| !line.is_empty())
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
                        error_model: ErrorModel::Proportional { sigma: 0.1 },
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
            error_model: ErrorModel::Proportional { sigma: 0.1 },
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
        
        let mut error_model = ErrorModel::Proportional { sigma: 0.1 };
        let mut model_type = "proportional";
        
        while self.current_line < self.lines.len() {
            let line = &self.lines[self.current_line];
            
            if line.starts_with('$') {
                break;
            }
            
            if line.to_uppercase().contains("MODEL") {
                if line.to_uppercase().contains("ADDITIVE") {
                    model_type = "additive";
                } else if line.to_uppercase().contains("COMBINED") {
                    model_type = "combined";
                } else if line.to_uppercase().contains("PROPORTIONAL") {
                    model_type = "proportional";
                }
            } else {
                let sigma_values = self.parse_sigma_line(line)?;
                
                error_model = match model_type {
                    "additive" => ErrorModel::Additive { 
                        sigma: sigma_values[0].sqrt() 
                    },
                    "combined" => ErrorModel::Combined { 
                        sigma_prop: sigma_values[0].sqrt(),
                        sigma_add: if sigma_values.len() > 1 { 
                            sigma_values[1].sqrt() 
                        } else { 
                            0.1 
                        }
                    },
                    _ => ErrorModel::Proportional { 
                        sigma: sigma_values[0].sqrt() 
                    },
                };
            }
            
            self.current_line += 1;
        }
        
        Ok(SimulationConfig {
            time_points: vec![0.0, 1.0, 2.0, 4.0, 8.0, 12.0, 24.0],
            error_model,
            integration_method: IntegrationMethod::Analytical,
            tolerance: None,
        })
    }
    
    fn parse_sigma_line(&self, line: &str) -> PKResult<Vec<f64>> {
        let cleaned = line.replace("(", "").replace(")", "");
        
        if cleaned.contains(',') {
            // Multiple sigma values (for combined error model)
            let values: Result<Vec<f64>, _> = cleaned
                .split(',')
                .map(|s| s.trim().parse::<f64>())
                .collect();
            values.map_err(|_| PKError::Validation(format!("Invalid sigma values: {}", line)))
        } else {
            // Single sigma value
            let value = cleaned.trim().parse::<f64>()
                .map_err(|_| PKError::Validation(format!("Invalid sigma value: {}", line)))?;
            Ok(vec![value])
        }
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
        let covariate_name = key_parts[2];
        
        // Set reference value based on covariate type
        let reference = match covariate_name {
            "WT" | "WEIGHT" => 70.0,
            "AGE" => 40.0,
            "HEIGHT" => 170.0,
            "BMI" => 25.0,
            "CRCL" => 100.0,
            "SEX" | "GENDER" => 0.0,  // 0=female, 1=male
            "RACE" | "ETHNIC" => 1.0, // 1=Caucasian, 2=Asian, 3=African, etc.
            _ => 1.0, // Default reference
        };
        
        // Determine covariate model based on covariate type
        let model = match covariate_name {
            "SEX" | "RACE" | "GENDER" | "ETHNIC" => CovariateModel::Linear,
            "WT" | "WEIGHT" | "AGE" | "HEIGHT" | "BMI" | "CRCL" => CovariateModel::Power,
            _ => {
                // Default based on typical values - categorical if small integers
                if value.abs() < 2.0 && value.fract() == 0.0 {
                    CovariateModel::Linear
                } else {
                    CovariateModel::Power
                }
            }
        };
        
        Ok((param, CovariateConfig {
            effect: value,
            reference,
            model,
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
    
    #[test]
    fn test_parse_theta_with_bounds() {
        let parser = ControlStreamParser::new("");
        
        // Test simple value
        let result = parser.parse_theta_line("2.5").unwrap();
        assert_eq!(result, (None, 2.5, None));
        
        // Test with bounds
        let result = parser.parse_theta_line("(0.1, 2.0, 10.0)").unwrap();
        assert_eq!(result, (Some(0.1), 2.0, Some(10.0)));
        
        // Test with spaces
        let result = parser.parse_theta_line("( 0.5 , 3.0 , 15.0 )").unwrap();
        assert_eq!(result, (Some(0.5), 3.0, Some(15.0)));
    }
    
    #[test]
    fn test_parse_omega_conversion() {
        let parser = ControlStreamParser::new("");
        
        // Test variance to CV% conversion
        let result = parser.parse_omega_line("0.09").unwrap();
        assert!((result - 30.0).abs() < 1e-6); // sqrt(0.09) * 100 = 30%
        
        let result = parser.parse_omega_line("0.0625").unwrap();
        assert!((result - 25.0).abs() < 1e-6); // sqrt(0.0625) * 100 = 25%
    }
    
    #[test]
    fn test_parse_covariate_line() {
        let parser = ControlStreamParser::new("");
        
        // Test weight effect
        let (param, config) = parser.parse_covariate_line("COV_CL_WT_EFFECT = 0.75").unwrap();
        assert_eq!(param, "CL_WT");
        assert_eq!(config.effect, 0.75);
        assert_eq!(config.reference, 70.0);
        assert!(matches!(config.model, CovariateModel::Power));
        
        // Test categorical effect
        let (param, config) = parser.parse_covariate_line("COV_CL_SEX_EFFECT = 0.2").unwrap();
        assert_eq!(param, "CL_SEX");
        assert_eq!(config.effect, 0.2);
        assert_eq!(config.reference, 0.0);
        assert!(matches!(config.model, CovariateModel::Linear));
    }
    
    #[test]
    fn test_parse_time_values() {
        let parser = ControlStreamParser::new("");
        
        let result = parser.extract_time_values("TIME_POINTS = 0.0, 1.0, 2.0, 4.0").unwrap();
        assert_eq!(result, vec![0.0, 1.0, 2.0, 4.0]);
        
        let result = parser.extract_time_values("TIMES = 0.0,12.0,24.0").unwrap();
        assert_eq!(result, vec![0.0, 12.0, 24.0]);
    }
    
    #[test]
    fn test_parse_error_models() {
        let content_prop = r#"
$SIGMA
MODEL = PROPORTIONAL
0.0225
"#;
        let mut parser = ControlStreamParser::new(content_prop);
        parser.current_line = 1; // Skip to $SIGMA
        let config = parser.parse_sigma_block().unwrap();
        assert!(matches!(config.error_model, ErrorModel::Proportional { .. }));
        
        let content_combined = r#"
$SIGMA
MODEL = COMBINED
0.0144, 0.0025
"#;
        let mut parser = ControlStreamParser::new(content_combined);
        parser.current_line = 1; // Skip to $SIGMA
        let config = parser.parse_sigma_block().unwrap();
        assert!(matches!(config.error_model, ErrorModel::Combined { .. }));
    }
}