# Population Pharmacokinetics Simulation Program

A comprehensive Rust implementation for population pharmacokinetics simulation with NONMEM-inspired algorithms supporting 1, 2, and 3-compartment models with various dosing regimens.

## Features

- **Multiple Compartment Models**: 1, 2, and 3-compartment pharmacokinetic models
- **Dosing Regimens**: Oral, IV bolus, and IV infusion administration
- **Population Variability**: Inter-individual (Omega) and residual (Sigma) variability
- **NONMEM-Style Algorithms**: Similar parameterization and error models
- **Flexible Configuration**: JSON-based configuration files
- **Comprehensive Output**: CSV and JSON formatted results with detailed reports

## Installation

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **Clone and build the project**:
   ```bash
   cargo build --release
   ```

## Usage

### Basic Command Structure

```bash
cargo run --release -- --config <config_file> --output <output_directory> --patients <number>
```

### Command Line Options

- `--config, -c`: Path to JSON configuration file
- `--output, -o`: Output directory for results
- `--patients, -p`: Number of patients to simulate (default: 100)
- `--seed, -s`: Random seed for reproducibility (optional)
- `--verbose, -v`: Enable verbose logging

## Example Simulations

### 1. One-Compartment Oral Model

Simulate 200 patients with oral dosing:

```bash
cargo run --release -- \
  --config examples/one_compartment_oral.json \
  --output results/one_compartment_oral \
  --patients 200 \
  --seed 12345
```

**Model Specifications:**
- Route: Oral administration
- Dose: 100 mg every 12 hours for 3 doses
- Parameters: CL (2.0 L/h), V (15.0 L), KA (1.5 h⁻¹)
- Variability: 30% CV on CL, 25% CV on V, 40% CV on KA
- Residual Error: 15% proportional

### 2. Two-Compartment IV Bolus Model

Simulate 150 patients with IV bolus dosing:

```bash
cargo run --release -- \
  --config examples/two_compartment_iv_bolus.json \
  --output results/two_compartment_iv \
  --patients 150 \
  --seed 67890
```

**Model Specifications:**
- Route: IV bolus
- Dose: 500 mg single dose
- Parameters: CL (3.5 L/h), V1 (12.0 L), Q2 (2.0 L/h), V2 (8.0 L)
- Variability: 25% CV on CL, 20% CV on V1, 35% CV on Q2, 30% CV on V2
- Residual Error: 12% proportional

### 3. Three-Compartment IV Infusion Model

Simulate 100 patients with IV infusion:

```bash
cargo run --release -- \
  --config examples/three_compartment_infusion.json \
  --output results/three_compartment_infusion \
  --patients 100 \
  --seed 54321
```

**Model Specifications:**
- Route: IV infusion over 2 hours
- Dose: 1000 mg at 0 and 24 hours
- Parameters: CL (4.2 L/h), V1 (10.0 L), Q2 (3.0 L/h), V2 (15.0 L), Q3 (1.5 L/h), V3 (25.0 L)
- Variability: 28% CV on CL, 22% CV on V1, 40% CV on Q2, 35% CV on V2, 50% CV on Q3, 45% CV on V3
- Residual Error: 18% proportional

## Configuration File Format

The simulation uses JSON configuration files with the following structure:

```json
{
  "model": {
    "compartments": 1,
    "parameters": {
      "CL": {
        "theta": 2.0,      // Typical value
        "omega": 30.0,     // Inter-individual variability (CV%)
        "bounds": [0.1, 10.0]  // Optional bounds
      }
    }
  },
  "dosing": {
    "route": "oral",           // "oral", "ivbolus", "ivinfusion"
    "amount": 100.0,
    "times": [0.0, 12.0, 24.0],
    "additional": {
      "duration": 2.0,         // For infusions
      "bioavailability": 0.8,  // For oral dosing
      "lag_time": 0.5          // For oral dosing
    }
  },
  "population": {
    "demographics": {
      "weight_mean": 70.0,
      "weight_sd": 15.0,
      "age_mean": 45.0,
      "age_sd": 12.0
    },
    "covariates": {
      "CL_WT": {
        "effect": 0.75,        // Allometric exponent
        "reference": 70.0      // Reference weight
      }
    }
  },
  "simulation": {
    "time_points": [0.0, 1.0, 2.0, 4.0, 8.0, 12.0, 24.0],
    "sigma": 0.15,             // Residual variability (proportional SD)
    "integration_method": "analytical"
  }
}
```

## Output Files

Each simulation generates several output files:

1. **`individual_data.csv`**: Patient demographics and PK endpoints
   - Columns: PATIENT_ID, WEIGHT, AGE, CMAX, AUC, TMAX

2. **`concentrations.csv`**: Concentration-time data
   - Columns: PATIENT_ID, TIME, CONCENTRATION, PREDICTED_CONCENTRATION

3. **`parameters.csv`**: Individual patient parameters
   - Columns: PATIENT_ID, CL, V, KA, Q2, V2, Q3, V3 (as applicable)

4. **`population_summary.json`**: Population statistics in JSON format

5. **`simulation_report.md`**: Human-readable simulation report

## Model Parameters

### One-Compartment Model
- **CL**: Clearance (L/h)
- **V**: Volume of distribution (L)
- **KA**: Absorption rate constant (h⁻¹) - for oral dosing

### Two-Compartment Model
- **CL**: Clearance (L/h)
- **V1**: Central volume of distribution (L)
- **Q2**: Inter-compartmental clearance (L/h)
- **V2**: Peripheral volume of distribution (L)
- **KA**: Absorption rate constant (h⁻¹) - for oral dosing

### Three-Compartment Model
- **CL**: Clearance (L/h)
- **V1**: Central volume of distribution (L)
- **Q2**: Inter-compartmental clearance 1↔2 (L/h)
- **V2**: Peripheral volume 2 (L)
- **Q3**: Inter-compartmental clearance 1↔3 (L/h)
- **V3**: Peripheral volume 3 (L)
- **KA**: Absorption rate constant (h⁻¹) - for oral dosing

## Variability Models

### Inter-Individual Variability (Omega)
- Log-normal distribution
- Specified as coefficient of variation (CV%)
- Applied to all PK parameters

### Residual Variability (Sigma)
- Proportional error model: Y = F × (1 + ε)
- Where ε ~ N(0, σ²)

### Covariate Effects
- Allometric scaling for body weight
- Power model: PARAM = THETA × (COV/REF)^EFFECT

## Advanced Features

### Covariate Relationships
The program supports covariate effects on parameters:

```json
"covariates": {
  "CL_WT": {
    "effect": 0.75,
    "reference": 70.0
  },
  "CL_AGE": {
    "effect": -0.3,
    "reference": 40.0
  }
}
```

### Multiple Dosing
Support for multiple doses at different times:

```json
"dosing": {
  "route": "oral",
  "amount": 100.0,
  "times": [0.0, 12.0, 24.0, 36.0, 48.0]
}
```

## Testing

Run the test suite:

```bash
cargo test
```

Run tests with output:

```bash
cargo test -- --nocapture
```

Run specific test modules:

```bash
cargo test models::one_compartment
cargo test simulation
```

## Examples and Validation

### Running All Examples

```bash
# One-compartment oral
cargo run --release -- -c examples/one_compartment_oral.json -o results/example1 -p 500

# Two-compartment IV bolus
cargo run --release -- -c examples/two_compartment_iv_bolus.json -o results/example2 -p 300

# Three-compartment IV infusion
cargo run --release -- -c examples/three_compartment_infusion.json -o results/example3 -p 200
```

### Creating Custom Configurations

1. Copy an existing example configuration
2. Modify parameters, dosing, or simulation settings
3. Run with your custom configuration

### Reproducible Simulations

Use the `--seed` option for reproducible results:

```bash
cargo run --release -- -c config.json -o results -p 100 --seed 12345
```

Running the same command with the same seed will produce identical results, which is essential for:
- Validation studies
- Regulatory submissions
- Method comparison
- Debugging and troubleshooting

## Performance Notes

- **Memory Usage**: Approximately 1-2 MB per 1000 patients
- **Computation Time**: ~1-10 seconds per 1000 patients (depending on model complexity)
- **Optimization**: Built with `--release` flag for production performance

## Troubleshooting

### Common Issues

1. **Configuration Validation Errors**
   - Check that all required parameters are specified
   - Ensure parameter values are positive
   - Verify JSON syntax

2. **Numerical Issues**
   - Very small or large parameter values may cause instability
   - Check parameter bounds in configuration

3. **Memory Issues**
   - For very large populations (>10,000 patients), consider running in batches

### Debug Mode

Enable verbose logging for detailed debugging:

```bash
cargo run --release -- -c config.json -o results -p 100 --verbose
```

## Mathematical Background

This implementation follows standard pharmacokinetic equations and NONMEM conventions:

- **Analytical solutions** for compartmental models
- **Log-normal parameter distributions** for biological realism
- **Proportional error models** for concentration observations
- **Allometric scaling** for covariate relationships

## License

This software is provided for educational and research purposes. Please cite appropriately if used in publications.

## Contributing

For bug reports, feature requests, or contributions, please follow standard Rust development practices with proper testing and documentation.