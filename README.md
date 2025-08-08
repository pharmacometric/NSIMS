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

### Configuration File Formats

The program supports two configuration file formats:

1. **JSON format** (`.json` extension) - Modern, structured format
2. **NONMEM control stream** (`.ctl`, `.mod`, or `.txt` extension) - Traditional NONMEM-style format

The program automatically detects the format based on file extension or content.

### Command Line Options

- `--config, -c`: Path to JSON configuration file
- `--output, -o`: Output directory for results
- `--patients, -p`: Number of patients to simulate (default: 100)
- `--seed, -s`: Random seed for reproducibility (optional)
- `--verbose, -v`: Enable verbose logging

## Example Simulations

### Using JSON Configuration Files

### 1. One-Compartment Oral Model

Simulate 200 patients with oral dosing:

```bash
cargo run --release -- \
  --config examples/one_compartment_oral.json \
  --output results/one_compartment_oral \
  --patients 200 \
  --seed 12345
```

### Using NONMEM Control Stream Files

### 1. One-Compartment Oral Model (NONMEM format)

Simulate 200 patients using NONMEM control stream:

```bash
cargo run --release -- \
  --config examples/one_compartment_oral.ctl \
  --output results/one_compartment_oral_ctl \
  --patients 200 \
  --seed 12345
```

### 2. Two-Compartment IV Bolus Model (NONMEM format)

Simulate 150 patients with NONMEM control stream:

```bash
cargo run --release -- \
  --config examples/two_compartment_iv_bolus.ctl \
  --output results/two_compartment_iv_ctl \
  --patients 150 \
  --seed 67890
```

### 3. Three-Compartment IV Infusion Model (NONMEM format)

Simulate 100 patients with NONMEM control stream:

```bash
cargo run --release -- \
  --config examples/three_compartment_infusion.ctl \
  --output results/three_compartment_infusion_ctl \
  --patients 100 \
  --seed 54321
```

### JSON Format Examples

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

## NONMEM Control Stream Format

The program supports NONMEM-style control streams with the following blocks:

### Required Blocks

- **$PROBLEM**: Problem description (optional)
- **$SUBROUTINES**: Model specification (ADVAN1, ADVAN3, ADVAN11)
- **$THETA**: Parameter initial estimates with optional bounds
- **$OMEGA**: Inter-individual variability (as variance)
- **$SIGMA**: Residual variability (as variance)

### Optional Blocks

- **$PK**: PK model code (parsed but not executed)
- **$DOSING**: Custom dosing specification
- **$POPULATION**: Population demographics and covariates
- **$SIMULATION**: Simulation settings

### Example Control Stream Structure

```
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
0.09     ; CL - 30% CV
0.0625   ; V - 25% CV
0.16     ; KA - 40% CV
$SIGMA
0.0225   ; Proportional error - 15% CV
$DOSING
ROUTE = ORAL
AMOUNT = 100.0
TIMES = 0.0, 12.0, 24.0
$SIMULATION
TIME_POINTS = 0.0, 1.0, 2.0, 4.0, 8.0, 12.0, 24.0
```

### ADVAN Subroutines Supported

- **ADVAN1**: One-compartment model
- **ADVAN3**: Two-compartment model  
- **ADVAN11**: Three-compartment model

## Configuration File Format

### JSON Format

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

### Comparison: JSON vs NONMEM Control Stream

| Feature | JSON Format | NONMEM Control Stream |
|---------|-------------|----------------------|
| **Syntax** | Modern JSON | Traditional NONMEM |
| **Readability** | Structured, clear | Familiar to NONMEM users |
| **Validation** | Built-in JSON validation | Custom parser |
| **Comments** | Limited | Full comment support |
| **Flexibility** | High | NONMEM-compatible |

Choose the format that best fits your workflow and team preferences.

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
- **Q**: Inter-compartmental clearance (L/h)
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

### Reproducible Simulations with Seeds

Use the `--seed` option for reproducible results:

```bash
# JSON format
cargo run --release -- -c examples/one_compartment_oral.json -o results -p 100 --seed 12345

# NONMEM control stream format
cargo run --release -- -c examples/one_compartment_oral.ctl -o results -p 100 --seed 12345
```

Running the same command with the same seed will produce identical results, essential for:
- Validation studies
- Regulatory submissions  
- Method comparison
- Debugging and troubleshooting

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
cargo test config::nonmem
```

## Examples and Validation

### Running All Examples

#### JSON Format Examples

```bash
# One-compartment oral
cargo run --release -- -c examples/one_compartment_oral.json -o results/example1 -p 500

# Two-compartment IV bolus
cargo run --release -- -c examples/two_compartment_iv_bolus.json -o results/example2 -p 300

# Three-compartment IV infusion
cargo run --release -- -c examples/three_compartment_infusion.json -o results/example3 -p 200
```

#### NONMEM Control Stream Examples

```bash
# One-compartment oral (NONMEM format)
cargo run --release -- -c examples/one_compartment_oral.ctl -o results/nonmem_example1 -p 500 --seed 12345

# Two-compartment IV bolus (NONMEM format)
cargo run --release -- -c examples/two_compartment_iv_bolus.ctl -o results/nonmem_example2 -p 300 --seed 67890

# Three-compartment IV infusion (NONMEM format)
cargo run --release -- -c examples/three_compartment_infusion.ctl -o results/nonmem_example3 -p 200 --seed 54321
```

### Creating Custom Configurations

#### JSON Format
1. Copy an existing example configuration
2. Modify parameters, dosing, or simulation settings
3. Run with your custom configuration

#### NONMEM Control Stream Format
1. Copy an existing `.ctl` file
2. Modify the $THETA, $OMEGA, $SIGMA, or $DOSING blocks
3. Update population demographics in $POPULATION block
4. Adjust simulation settings in $SIMULATION block

### Format Conversion

You can easily convert between formats by running simulations and examining the generated `population_summary.json` file, which contains all the configuration information in JSON format.

### Reproducible Simulations

```bash
# Both formats support reproducible simulations
cargo run --release -- -c examples/one_compartment_oral.json -o results -p 100 --seed 12345
cargo run --release -- -c examples/one_compartment_oral.ctl -o results -p 100 --seed 12345
```

Running the same command with the same seed will produce identical results regardless of configuration format, which is essential for:
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