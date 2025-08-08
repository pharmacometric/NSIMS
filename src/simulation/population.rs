use super::{PatientResult, Demographics};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PopulationSummary {
    pub n_patients: usize,
    pub parameters: ParameterSummary,
    pub pharmacokinetics: PKSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParameterSummary {
    pub cl_mean: f64,
    pub cl_sd: f64,
    pub v_mean: f64,
    pub v_sd: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PKSummary {
    pub cmax_mean: f64,
    pub cmax_sd: f64,
    pub auc_mean: f64,
    pub auc_sd: f64,
    pub tmax_mean: f64,
    pub tmax_sd: f64,
}

impl PopulationSummary {
    pub fn from_results(results: &[PatientResult]) -> Self {
        let n = results.len();
        
        // Calculate parameter statistics
        let cl_values: Vec<f64> = results.iter()
            .map(|r| *r.parameters.get("CL").unwrap_or(&0.0))
            .collect();
        let v_values: Vec<f64> = results.iter()
            .map(|r| *r.parameters.get("V").or_else(|| r.parameters.get("V1")).unwrap_or(&0.0))
            .collect();
        
        // Calculate PK statistics
        let cmax_values: Vec<f64> = results.iter()
            .map(|r| r.get_max_concentration())
            .collect();
        let auc_values: Vec<f64> = results.iter()
            .map(|r| r.get_auc())
            .collect();
        let tmax_values: Vec<f64> = results.iter()
            .filter_map(|r| r.get_time_to_max())
            .collect();
        
        Self {
            n_patients: n,
            parameters: ParameterSummary {
                cl_mean: mean(&cl_values),
                cl_sd: std_dev(&cl_values),
                v_mean: mean(&v_values),
                v_sd: std_dev(&v_values),
            },
            pharmacokinetics: PKSummary {
                cmax_mean: mean(&cmax_values),
                cmax_sd: std_dev(&cmax_values),
                auc_mean: mean(&auc_values),
                auc_sd: std_dev(&auc_values),
                tmax_mean: mean(&tmax_values),
                tmax_sd: std_dev(&tmax_values),
            },
        }
    }
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

fn std_dev(values: &[f64]) -> f64 {
    if values.len() < 2 {
        0.0
    } else {
        let mean_val = mean(values);
        let variance = values.iter()
            .map(|v| (v - mean_val).powi(2))
            .sum::<f64>() / (values.len() - 1) as f64;
        variance.sqrt()
    }
}