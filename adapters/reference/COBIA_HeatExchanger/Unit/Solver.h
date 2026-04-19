#pragma once
#include <COBIA.h>
#include "Collection.h"
#include "MaterialPort.h"

using namespace COBIA;

class Solver :

	/*
	* TODO: Description
	*/

	public CapeOpenObject<Solver>,
	public CAPEOPEN_1_2::CapeIdentificationAdapter<Solver> {

	// Members
	PortCollectionPtr& portCollection;

	CapeInteger streamCount;
	std::vector<CAPEOPEN_1_2::CapeThermoMaterial> inletMaterials, outletMaterials;
	std::vector<CapeReal> totalMolarFlow, T, P, enthalpyF;
	std::vector<CapeArrayReal> X;

public:

	const CapeStringImpl getDescriptionForErrorSource() {
		return COBIATEXT("Solver");
	}

	Solver(PortCollectionPtr& _portCollection) : portCollection(_portCollection) {

		// Store inlet & outlet material
		for (MaterialPortPtr& portPtr : portCollection->iterateOverItems()) {
			if (portPtr->getConnectedObject()) {
				if (portPtr->getDirection() == CAPEOPEN_1_2::CAPE_INLET) {
					inletMaterials.emplace_back(portPtr->getMaterial());
				}
				else {
					outletMaterials.emplace_back(portPtr->getMaterial());

				}
			}
		}

		streamCount = inletMaterials.size();

		// CAPE-OPEN unit operations may not have side effects on material objects connected to feeds.
		// Product material object id copied from feed material object allowing to perfrom calculations
		// before setting its properties to override feed peoperties' values
		for (size_t i = 0, length = outletMaterials.size(); i < length; i++) {
			outletMaterials[i].CopyFromMaterial(inletMaterials[i]);
		}
	}

	~Solver() {
	}
	
	void getInitialConditions() {

		CapeArrayReal propValues;
		for (size_t i = 0; i < streamCount; i++) {
			// Get molarFlow (inlet=outlet)
			outletMaterials[i].GetOverallProp(ConstCapeString(COBIATEXT("totalflow")),
				ConstCapeString(COBIATEXT("mole")), propValues);
			totalMolarFlow[i] = propValues[0];
			// Get T, P & X
			outletMaterials[i].GetOverallTPFraction(T[i], P[i], X[i]);
			// Get enthalpyF
			enthalpyF[i] = calcOverallFromPhaseProps(outletMaterials[i], COBIATEXT("enthalpyF"));
		}
	}

	void setProduct() {
		// Set product(s) overall props.
	}

	void flashProduct(/*in*/ std::vector<CapeArrayStringImpl> phaseIDs,
		/*in*/ std::vector<CapeArrayEnumerationImpl<CAPEOPEN_1_2::CapePhaseStatus>> phaseStatus,
		/*in*/ CapeArrayStringImpl flashCond1, /*in*/ CapeArrayStringImpl flashCond2) {

		size_t j = 0;
		for (MaterialPortPtr portPtr : portCollection->iterateOverItems()) {
			if (portPtr->getPortType() == CAPEOPEN_1_2::CAPE_MATERIAL &&
				portPtr->getDirection() == CAPEOPEN_1_2::CAPE_OUTLET &&
				portPtr->getConnectedObject()) {

				// Allow all phases to take part in product flash
				CAPEOPEN_1_2::CapeThermoMaterial material = portPtr->getMaterial();
				material.SetPresentPhases(phaseIDs[j], phaseStatus[j]);

				// Flash product(s) at specified Conditions
				CAPEOPEN_1_2::CapeThermoEquilibriumRoutine equilibriumRoutine(material);
				equilibriumRoutine.CalcEquilibrium(flashCond1, flashCond2, ConstCapeEmptyString());

				j++;
			}
		}
	}

	//CAPEOPEN_1_2::ICapeIdentification
	void getComponentName(/*out*/ CapeString name) {
		name = COBIATEXT("Solver");
	}
	void putComponentName(/*in*/ CapeString name) {
		throw cape_open_error(COBIAERR_Denied);
	}
	void getComponentDescription(/*out*/ CapeString desc) {
		// TODO
		desc = COBIATEXT("Solver Class");
	}
	void putComponentDescription(/*in*/ CapeString desc) {
		throw cape_open_error(COBIAERR_Denied);
	}
};

using SolverPtr = CapeOpenObjectSmartPointer<Solver>;