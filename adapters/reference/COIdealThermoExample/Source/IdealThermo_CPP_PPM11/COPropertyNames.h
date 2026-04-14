#pragma once
#include <Properties.h>

//definition of version 1.0 CAPE-OPEN names for the properties
// (case insensitive)

#define ExposedTDependentPropertyCount 6 /*we do not expose LiquidDensity*/

extern const OLECHAR *TDependentPropertyNames[ExposedTDependentPropertyCount];
extern const OLECHAR *SinglePhasePropertyNames[SinglePhasePropertyCount];
extern const OLECHAR *TwoPhasePropertyNames[TwoPhasePropertyCount];

//we also need to know whether the properties are mole basis
// (the underlying calculations never return mass basis)
// (two phase properties are never on mole basis)

extern const bool TDependentPropertyMoleBasis[ExposedTDependentPropertyCount];
extern const bool SinglePhasePropertyMoleBasis[SinglePhasePropertyCount];

