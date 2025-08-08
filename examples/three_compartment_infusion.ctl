$PROBLEM Three compartment IV infusion model with covariate effects

$SUBROUTINES ADVAN11 TRANS4

$PK
; Three compartment model with IV infusion
CL = THETA(1) * (WT/70)**0.75 * (AGE/40)**(-0.3)
V1 = THETA(2) * (WT/70)
Q2 = THETA(3)
V2 = THETA(4)
Q3 = THETA(5)
V3 = THETA(6)

$THETA
(1.0, 4.2, 20.0)    ; CL (L/h) - Clearance
(3.0, 10.0, 25.0)   ; V1 (L) - Central volume
(0.5, 3.0, 15.0)    ; Q2 (L/h) - Inter-compartmental clearance 1->2
(5.0, 15.0, 40.0)   ; V2 (L) - Peripheral volume 2
(0.1, 1.5, 8.0)     ; Q3 (L/h) - Inter-compartmental clearance 1->3
(8.0, 25.0, 80.0)   ; V3 (L) - Peripheral volume 3

$OMEGA
0.0784   ; CL - 28% CV
0.0484   ; V1 - 22% CV
0.16     ; Q2 - 40% CV
0.1225   ; V2 - 35% CV
0.25     ; Q3 - 50% CV
0.2025   ; V3 - 45% CV

$SIGMA
MODEL = ADDITIVE
0.0625   ; Additive error - 0.25 mg/L SD

$DOSING
ROUTE = INFUSION
AMOUNT = 1000.0
TIMES = 0.0, 24.0
DURATION = 2.0

$POPULATION
WEIGHT_MEAN = 72.0
WEIGHT_SD = 16.0
AGE_MEAN = 55.0
AGE_SD = 18.0
COV_CL_WT_EFFECT = 0.75
COV_V1_WT_EFFECT = 1.0
COV_CL_AGE_EFFECT = -0.3

$SIMULATION
TIME_POINTS = 0.0, 0.5, 1.0, 2.0, 2.5, 3.0, 4.0, 6.0, 8.0, 12.0, 18.0, 24.0, 24.5, 25.0, 26.0, 28.0, 32.0, 36.0, 48.0, 72.0, 96.0
METHOD = ANALYTICAL