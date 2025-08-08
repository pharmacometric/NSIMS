use rand_distr::{Normal, LogNormal, Distribution};
use crate::error::{PKError, PKResult};

/// NONMEM-style log-normal variability
pub fn apply_log_normal_variability<R: rand::Rng>(
    base_value: f64,
    cv_percent: f64,
    rng: &mut R,
) -> PKResult<f64> {
    if cv_percent <= 0.0 {
        return Ok(base_value);
    }
    
    // Convert CV% to log-normal parameters
    let cv = cv_percent / 100.0;
    let sigma_log = (cv * cv + 1.0).ln().sqrt();
    let mu_log = base_value.ln() - sigma_log * sigma_log / 2.0;
    
    let log_normal = LogNormal::new(mu_log, sigma_log)
        .map_err(|_| PKError::Random)?;
    Ok(log_normal.sample(rng))
}

/// NONMEM-style proportional error model
pub fn apply_proportional_error<R: rand::Rng>(
    predicted: f64,
    proportional_sd: f64,
    rng: &mut R,
) -> PKResult<f64> {
    if predicted <= 0.0 {
        return Ok(0.0);
    }
    
    let normal = Normal::new(0.0, proportional_sd)
        .map_err(|_| PKError::Random)?;
    let epsilon = normal.sample(rng);
    
    // Y = F * (1 + EPS(1))
    let observed = predicted * (1.0 + epsilon);
    Ok(observed.max(0.0))
}

/// Combined additive and proportional error model
pub fn apply_combined_error<R: rand::Rng>(
    predicted: f64,
    additive_sd: f64,
    proportional_sd: f64,
    rng: &mut R,
) -> PKResult<f64> {
    if predicted <= 0.0 {
        return Ok(0.0);
    }
    
    let normal_add = Normal::new(0.0, additive_sd)
        .map_err(|_| PKError::Random)?;
    let normal_prop = Normal::new(0.0, proportional_sd)
        .map_err(|_| PKError::Random)?;
    
    let eps1 = normal_add.sample(rng);
    let eps2 = normal_prop.sample(rng);
    
    // Y = F * (1 + EPS(1)) + EPS(2)
    let observed = predicted * (1.0 + eps2) + eps1;
    Ok(observed.max(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    
    #[test]
    fn test_log_normal_variability() {
        let mut rng = StdRng::seed_from_u64(42);
        let base_value = 10.0;
        let cv_percent = 30.0;
        
        let varied_value = apply_log_normal_variability(base_value, cv_percent, &mut rng).unwrap();
        assert!(varied_value > 0.0);
        assert!(varied_value != base_value); // Should be different due to variability
    }
    
    #[test]
    fn test_proportional_error() {
        let mut rng = StdRng::seed_from_u64(42);
        let predicted = 5.0;
        let prop_sd = 0.1;
        
        let observed = apply_proportional_error(predicted, prop_sd, &mut rng).unwrap();
        assert!(observed >= 0.0);
    }
    
    #[test]
    fn test_combined_error() {
        let mut rng = StdRng::seed_from_u64(42);
        let predicted = 5.0;
        let add_sd = 0.5;
        let prop_sd = 0.1;
        
        let observed = apply_combined_error(predicted, add_sd, prop_sd, &mut rng).unwrap();
        assert!(observed >= 0.0);
    }
}