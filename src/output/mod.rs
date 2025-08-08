use crate::simulation::{PatientResult, PopulationSummary};
use crate::error::{PKError, PKResult};
use std::path::Path;
use std::fs::File;
use log::info;

pub fn save_results<P: AsRef<Path>>(results: &[PatientResult], output_dir: P) -> PKResult<()> {
    let output_path = output_dir.as_ref();
    
    // Save individual patient data
    save_patient_data(results, &output_path.join("individual_data.csv"))?;
    
    // Save concentration-time data
    save_concentration_data(results, &output_path.join("concentrations.csv"))?;
    
    // Save population summary
    let summary = PopulationSummary::from_results(results);
    save_population_summary(&summary, &output_path.join("population_summary.json"))?;
    
    // Save parameters
    save_parameter_data(results, &output_path.join("parameters.csv"))?;
    
    info!("All results saved to {:?}", output_path);
    Ok(())
}

fn save_patient_data<P: AsRef<Path>>(results: &[PatientResult], path: P) -> PKResult<()> {
    let mut writer = csv::Writer::from_path(path)?;
    
    // Write header
    writer.write_record(&[
        "PATIENT_ID", "WEIGHT", "AGE", "CMAX", "AUC", "TMAX"
    ])?;
    
    // Write data
    for result in results {
        let cmax = result.get_max_concentration();
        let auc = result.get_auc();
        let tmax = result.get_time_to_max().unwrap_or(0.0);
        
        writer.write_record(&[
            result.patient_id.to_string(),
            result.demographics.weight.to_string(),
            result.demographics.age.to_string(),
            cmax.to_string(),
            auc.to_string(),
            tmax.to_string(),
        ])?;
    }
    
    writer.flush()?;
    Ok(())
}

fn save_concentration_data<P: AsRef<Path>>(results: &[PatientResult], path: P) -> PKResult<()> {
    let mut writer = csv::Writer::from_path(path)?;
    
    // Write header
    writer.write_record(&[
        "PATIENT_ID", "TIME", "CONCENTRATION", "PREDICTED_CONCENTRATION"
    ])?;
    
    // Write data
    for result in results {
        for obs in &result.observations {
            writer.write_record(&[
                result.patient_id.to_string(),
                obs.time.to_string(),
                obs.concentration.to_string(),
                obs.predicted_concentration.to_string(),
            ])?;
        }
    }
    
    writer.flush()?;
    Ok(())
}

fn save_parameter_data<P: AsRef<Path>>(results: &[PatientResult], path: P) -> PKResult<()> {
    if results.is_empty() {
        return Ok(());
    }
    
    let mut writer = csv::Writer::from_path(path)?;
    
    // Get parameter names from first patient
    let param_names: Vec<String> = results[0].parameters.keys().cloned().collect();
    
    // Write header
    let mut header = vec!["PATIENT_ID".to_string()];
    header.extend(param_names.clone());
    writer.write_record(&header)?;
    
    // Write data
    for result in results {
        let mut record = vec![result.patient_id.to_string()];
        for param_name in &param_names {
            let value = result.parameters.get(param_name).unwrap_or(&0.0);
            record.push(value.to_string());
        }
        writer.write_record(&record)?;
    }
    
    writer.flush()?;
    Ok(())
}

fn save_population_summary<P: AsRef<Path>>(summary: &PopulationSummary, path: P) -> PKResult<()> {
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, summary)?;
    Ok(())
}

/// Generate a comprehensive report
pub fn generate_report<P: AsRef<Path>>(results: &[PatientResult], output_dir: P) -> PKResult<()> {
    let output_path = output_dir.as_ref();
    let report_path = output_path.join("simulation_report.md");
    
    let summary = PopulationSummary::from_results(results);
    
    let report_content = format!(
        r#"# Population Pharmacokinetics Simulation Report

## Simulation Overview
- **Number of patients**: {}
- **Time points**: {:?}

## Population Parameters
### Clearance (CL)
- Mean: {:.3} L/h
- SD: {:.3} L/h
- CV%: {:.1}%

### Volume of Distribution (V)
- Mean: {:.3} L
- SD: {:.3} L
- CV%: {:.1}%

## Pharmacokinetic Endpoints
### Maximum Concentration (Cmax)
- Mean: {:.3} mg/L
- SD: {:.3} mg/L
- CV%: {:.1}%

### Area Under the Curve (AUC)
- Mean: {:.3} mg*h/L
- SD: {:.3} mg*h/L
- CV%: {:.1}%

### Time to Maximum Concentration (Tmax)
- Mean: {:.3} h
- SD: {:.3} h

## Files Generated
- `individual_data.csv`: Patient demographics and PK endpoints
- `concentrations.csv`: Concentration-time data for all patients
- `parameters.csv`: Individual patient parameters
- `population_summary.json`: Detailed population statistics

## Notes
This simulation was generated using NONMEM-inspired algorithms with appropriate
inter-individual and residual variability models.
"#,
        summary.n_patients,
        if results.is_empty() { vec![] } else { 
            results[0].observations.iter().map(|o| o.time).collect::<Vec<_>>() 
        },
        summary.parameters.cl_mean,
        summary.parameters.cl_sd,
        if summary.parameters.cl_mean > 0.0 { summary.parameters.cl_sd / summary.parameters.cl_mean * 100.0 } else { 0.0 },
        summary.parameters.v_mean,
        summary.parameters.v_sd,
        if summary.parameters.v_mean > 0.0 { summary.parameters.v_sd / summary.parameters.v_mean * 100.0 } else { 0.0 },
        summary.pharmacokinetics.cmax_mean,
        summary.pharmacokinetics.cmax_sd,
        if summary.pharmacokinetics.cmax_mean > 0.0 { summary.pharmacokinetics.cmax_sd / summary.pharmacokinetics.cmax_mean * 100.0 } else { 0.0 },
        summary.pharmacokinetics.auc_mean,
        summary.pharmacokinetics.auc_sd,
        if summary.pharmacokinetics.auc_mean > 0.0 { summary.pharmacokinetics.auc_sd / summary.pharmacokinetics.auc_mean * 100.0 } else { 0.0 },
        summary.pharmacokinetics.tmax_mean,
        summary.pharmacokinetics.tmax_sd,
    );
    
    std::fs::write(report_path, report_content)?;
    Ok(())
}