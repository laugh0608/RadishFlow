#pragma once
#include <COBIA.h>

using namespace COBIA;

// Get component name
CapeStringImpl getName(CapeInterface param) {
	CapeStringImpl paramName;
	CAPEOPEN_1_2::CapeIdentification identification(param);
	identification.getComponentName(paramName);
	return paramName;
}

CapeReal calcOverallFromPhaseProps(/*in*/ CAPEOPEN_1_2::CapeThermoMaterial thermoMaterial,
	/*in*/ CapeStringImpl prop) {

	/* This function will calculate phase properties and phase fraction for a given stream
	and return the overall value of the property */

	CapeArrayStringImpl props(1);
	props[0] = prop;

	CapeArrayRealImpl value(1);
	CapeReal overallValue = 0;

	CapeArrayStringImpl phaseLabels;
	CapeArrayEnumerationImpl<CAPEOPEN_1_2::CapePhaseStatus> phaseStatus;
	CapeArrayRealImpl phaseFraction;

	// Get present phaseLabels to calculate their properties
	thermoMaterial.GetPresentPhases(phaseLabels, phaseStatus);

	// Iterate over present phases and calculate selected phase properties
	for (ConstCapeString phaseLabel : phaseLabels) {

		// Calculate phase fraction
		thermoMaterial.GetSinglePhaseProp(ConstCapeString(COBIATEXT("phaseFraction")),
			phaseLabel, ConstCapeString(COBIATEXT("mole")), phaseFraction);

		/* Before getting a property from a material object,
		property must be calculated using CalcSinglePhaseProp.
		Exceptions are for properties that are available at phase equilibrium:
		flow rate, composition, pressure, temperature and phase fraction */
		CAPEOPEN_1_2::CapeThermoPropertyRoutine routine(thermoMaterial);
		routine.CalcSinglePhaseProp(props, phaseLabel);

		thermoMaterial.GetSinglePhaseProp(prop, phaseLabel,
			ConstCapeString(COBIATEXT("mole")), value);
		overallValue += phaseFraction[0] * value[0];
		return overallValue;
	}
}