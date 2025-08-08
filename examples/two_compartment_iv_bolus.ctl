$PROBLEM Two compartment IV bolus model

$SUBROUTINES ADVAN3 TRANS4

$PK
; Two compartment model with IV bolus administration
CL = THETA(1) * (WT/70)**0.75
V1 = THETA(2) * (WT/70)
Q = THETA(3)
V2 = THETA(4)

$THETA
(0.5, 3.5, 15.0)    ; CL (L/h) - Clearance
(5.0, 12.0, 30.0)   ; V1 (L) - Central volume
(0.1, 2.0, 10.0)    ; Q (L/h) - Inter-compartmental clearance
(2.0, 8.0, 25.0)    ; V2 (L) - Peripheral volume

$OMEGA
0.0625   ; CL - 25% CV
0.04     ; V1 - 20% CV
0.1225   ; Q - 35% CV
0.09     ; V2 - 30% CV

$SIGMA
0.0144   ; Proportional error - 12% CV

$DOSING
ROUTE = IVBOLUS
AMOUNT = 500.0
TIMES = 0.0

$POPULATION
WEIGHT_MEAN = 75.0
WEIGHT_SD = 18.0
AGE_MEAN = 50.0
AGE_SD = 15.0
COV_CL_WT_EFFECT = 0.75
COV_V1_WT_EFFECT = 1.0

$SIMULATION
TIME_POINTS = 0.0, 0.083, 0.25, 0.5, 1.0, 2.0, 4.0, 6.0, 8.0, 12.0, 18.0, 24.0, 36.0, 48.0, 72.0
METHOD = ANALYTICAL