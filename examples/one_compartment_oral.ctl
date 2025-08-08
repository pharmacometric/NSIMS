$PROBLEM One compartment oral absorption model with multiple dosing

$SUBROUTINES ADVAN1 TRANS2

$PK
; One compartment model with first-order absorption
CL = THETA(1) * (WT/70)**0.75
V = THETA(2) * (WT/70)
KA = THETA(3)

$THETA
(0.1, 2.0, 10.0)    ; CL (L/h) - Clearance
(5.0, 15.0, 50.0)   ; V (L) - Volume of distribution
(0.1, 1.5, 5.0)     ; KA (1/h) - Absorption rate constant

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
BIOAVAILABILITY = 0.8
LAG_TIME = 0.5

$POPULATION
WEIGHT_MEAN = 70.0
WEIGHT_SD = 15.0
AGE_MEAN = 45.0
AGE_SD = 12.0
COV_CL_WT_EFFECT = 0.75
COV_V_WT_EFFECT = 1.0

$SIMULATION
TIME_POINTS = 0.0, 0.5, 1.0, 2.0, 4.0, 6.0, 8.0, 12.0, 16.0, 24.0, 36.0, 48.0
METHOD = ANALYTICAL