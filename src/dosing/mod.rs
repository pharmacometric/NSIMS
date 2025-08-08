use crate::config::{DosingConfig, DosingRoute};
use crate::models::{DoseEvent, DoseRoute as ModelDoseRoute};
use crate::error::{PKError, PKResult};

pub struct DosingRegimen {
    pub events: Vec<DoseEvent>,
}

impl DosingRegimen {
    pub fn from_config(config: &DosingConfig) -> PKResult<Self> {
        let route = match config.route {
            DosingRoute::Oral => ModelDoseRoute::Oral,
            DosingRoute::IvBolus => ModelDoseRoute::IvBolus,
            DosingRoute::IvInfusion => ModelDoseRoute::IvInfusion,
        };
        
        let mut events = Vec::new();
        
        for &time in &config.times {
            let duration = if route == ModelDoseRoute::IvInfusion {
                config.additional.as_ref()
                    .and_then(|a| a.duration)
            } else {
                None
            };
            
            events.push(DoseEvent {
                time,
                amount: config.amount,
                route: route.clone(),
                duration,
            });
        }
        
        // Sort by time
        events.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        
        Ok(Self { events })
    }
    
    pub fn get_events_before(&self, time: f64) -> Vec<DoseEvent> {
        self.events.iter()
            .filter(|event| event.time <= time)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AdditionalDosingParams;
    
    #[test]
    fn test_dosing_regimen_creation() {
        let config = DosingConfig {
            route: DosingRoute::Oral,
            amount: 100.0,
            times: vec![0.0, 12.0, 24.0],
            additional: None,
        };
        
        let regimen = DosingRegimen::from_config(&config).unwrap();
        assert_eq!(regimen.events.len(), 3);
        assert_eq!(regimen.events[0].time, 0.0);
        assert_eq!(regimen.events[1].time, 12.0);
        assert_eq!(regimen.events[2].time, 24.0);
    }
    
    #[test]
    fn test_infusion_regimen() {
        let config = DosingConfig {
            route: DosingRoute::IvInfusion,
            amount: 1000.0,
            times: vec![0.0],
            additional: Some(AdditionalDosingParams {
                duration: Some(2.0),
                lag_time: None,
                bioavailability: None,
            }),
        };
        
        let regimen = DosingRegimen::from_config(&config).unwrap();
        assert_eq!(regimen.events.len(), 1);
        assert_eq!(regimen.events[0].duration, Some(2.0));
    }
}