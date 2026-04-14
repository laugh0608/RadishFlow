#pragma once
#include <COBIA.h>
#include "Collection.h"
#include "MaterialPort.h"

using namespace COBIA;

class Validator :

	/*
	* TODO: Description
	*/

	public CapeOpenObject<Validator>,
	public CAPEOPEN_1_2::CapeIdentificationAdapter<Validator> {
	
	// Members
	PortCollectionPtr& portCollection;
	ParameterCollectionPtr& paramCollection;
	CapeArrayStringImpl& sideOptions;
	#define IGNORED sideOptions[0]

public:

	const CapeStringImpl getDescriptionForErrorSource() {
		return COBIATEXT("Validator");
	}

	Validator(PortCollectionPtr& _portCollection, ParameterCollectionPtr& _paramCollection,
		CapeArrayStringImpl& _sideOptions) :
		portCollection(_portCollection), paramCollection(_paramCollection), sideOptions(_sideOptions) {
	}

	~Validator() {
	}

	CapeBoolean validateParameterSpecifications(/*out*/ CapeString message) {
		CapeBoolean val = true;
		for (CAPEOPEN_1_2::CapeParameter& param : paramCollection->iterateOverItems()) {

			// Only validate if not valid
			if (param.getValStatus() != CAPEOPEN_1_2::CAPE_VALID) {
				if (param.getType() == CAPEOPEN_1_2::CAPE_PARAMETER_REAL) {
					val = PARAMREALCAST(param)->Validate(PARAMREALCAST(param)->getValue(), message);
				}
				else if (param.getType() == CAPEOPEN_1_2::CAPE_PARAMETER_STRING) {
					CapeString value(new CapeStringImpl);
					PARAMSTRINGCAST(param)->getValue(value);
					val = PARAMSTRINGCAST(param)->Validate(value, message);
				}
			}
			// Return if last parameter has an error
			if (!val) {
				break;
			}
		}
		return val;
	}

	CapeBoolean validateMSHEXSides(/*out*/ CapeString message) {

		// Get inlet side parameter value for first 2 "primary" inlets
		CapeString in1sideValue(new CapeStringImpl), in2sideValue(new CapeStringImpl);
		PARAMSTRINGCAST(paramCollection->getItemImpl(0))->getValue(in1sideValue);
		PARAMSTRINGCAST(paramCollection->getItemImpl(1))->getValue(in2sideValue);

		// Validate that primary streams are not ignored
		if (in1sideValue == IGNORED) {
			message = COBIATEXT("Primary stream Inlet 1 cannot be ignored");
			return false;
		}
		if (in2sideValue == IGNORED) {
			message = COBIATEXT("Primary stream Inlet 2 cannot be ignored");
			return false;
		}
		
		// Validate that primary streams are not on the same side
		if (in1sideValue == in2sideValue) {
			message = COBIATEXT("Primary streams cannot be on the same side");
			return false;
		}

		return true;
	}


	CapeBoolean validateMaterialPorts(/*out*/ CapeString message) {

		// Check whether all ports are connected, and connected to materials with equal compound lists
		MaterialPortPtr portPtr;
		CapeInterface connectedObject;
		CapeArrayStringImpl refCompIDs, compIDs;
		CapeArrayStringImpl formulae, names, casNumbers;
		CapeArrayRealImpl boilTemps, molecularWeights;
		CapeBoolean sameCompList;

		for (CapeInteger index = 0, count = portCollection->getCount(); index < count; index++) {
			portPtr = portCollection->getItemImpl(index);
			connectedObject = portPtr->getConnectedObject();

			// Check whether port is connected
			if (!connectedObject) {
				// If not connected, check if the port if primary
				// or optional, but with additional requirments imposed by the unit
				if (portPtr->isPrimary() || requiredByMSHEX(message, sideOptions, portPtr, index)) {
					message = COBIATEXT("Port ") + getName(static_cast<CapeInterface>(portPtr)) + COBIATEXT(" is not connected");
					return false;
				}
			}
			else {
				if (portPtr->getPortType() == CAPEOPEN_1_2::CAPE_MATERIAL) {
					CAPEOPEN_1_2::CapeThermoCompounds compounds(connectedObject);
					// Get compound list for from first stream as reference
					if (refCompIDs.empty()) {
						compounds.GetCompoundList(refCompIDs, formulae, names,
							boilTemps, molecularWeights, casNumbers);
					}
					// Compare all other streams with it
					compounds.GetCompoundList(compIDs, formulae, names, boilTemps, molecularWeights, casNumbers);
					sameCompList = (refCompIDs.size() == compIDs.size());
					for (size_t i = 0; (i < compIDs.size()) && (sameCompList); i++) {
						sameCompList = (compIDs[i].compare(refCompIDs[i]) == 0);
					}
					if (!sameCompList) {
						message = COBIATEXT("Connected material streams expose inconsistent compound lists");
						return false;
					}
					else if (compIDs.empty()) {
						message = COBIATEXT("Connected material streams expose zero compound");
						return false;
					}
				}
			}
		}
		return true;
	}

	CapeBoolean requiredByMSHEX(/*out*/ CapeString message,
		/*in*/ CapeArrayStringImpl& sideOptions, /*in*/MaterialPortPtr& portPtr,
		/*in*/CapeInteger index) {


		// This function applies to material ports only
		if (portPtr->getPortType() != CAPEOPEN_1_2::CAPE_MATERIAL) { return false; }
		
		// 	If an optional inlet is not ignored or ignored but has a connected outlet, it is required
		if (portPtr->getDirection() == CAPEOPEN_1_2::CAPE_INLET) {
			// Get inlet side param value
			CapeString sideValue(new CapeStringImpl);
			PARAMSTRINGCAST(paramCollection->getItemImpl(index/2))->getValue(sideValue);
			if (portCollection->getItemImpl(index+1)->getConnectedObject() || sideValue.c_str() != IGNORED) {
				return true;
			}
		}

		// 	If an optional outlet has a connected inlet, it is required
		else if (portPtr->getDirection() == CAPEOPEN_1_2::CAPE_OUTLET) {
			if (portCollection->getItemImpl(index-1)->getConnectedObject()) {
				return true;
			}
		}
	
		return false;
	}

	void preparePhaseIDs (/*out*/ std::vector<CapeArrayStringImpl> &productsPhaseIDs,
		/*out*/ std::vector<CapeArrayEnumerationImpl<CAPEOPEN_1_2::CapePhaseStatus>> &productsPhaseStatus) {

		// Clear vectors before requesting phaseIDs of products
		productsPhaseIDs.clear();
		productsPhaseStatus.clear();
		// Prepare lists of supported phase label and phase status for product flash
		// This remains constant between validations
		CAPEOPEN_1_2::CapeThermoMaterial material;
		CapeArrayStringImpl phaseIDs, stateOfAggregation, keyCompounds;
		CapeArrayEnumerationImpl<CAPEOPEN_1_2::CapePhaseStatus> phaseStatus;

		
		for (MaterialPortPtr& portPtr: portCollection->iterateOverItems()) {
			if (portPtr->getPortType() == CAPEOPEN_1_2::CAPE_MATERIAL &&
				portPtr->getDirection() == CAPEOPEN_1_2::CAPE_OUTLET &&
				portPtr->getConnectedObject()) {
				material = portPtr->getMaterial();
				CAPEOPEN_1_2::CapeThermoPhases phases(material);
				phases.GetPhaseList(phaseIDs, stateOfAggregation, keyCompounds);
				phaseStatus.resize(phaseIDs.size());
				std::fill(phaseStatus.begin(), phaseStatus.end(), CAPEOPEN_1_2::CAPE_UNKNOWNPHASESTATUS);
				productsPhaseIDs.emplace_back(phaseIDs);
				productsPhaseStatus.emplace_back(phaseStatus);
			}
		}
	}

	//CAPEOPEN_1_2::ICapeIdentification
	void getComponentName(/*out*/ CapeString name) {
		name = COBIATEXT("Validator");
	}
	void putComponentName(/*in*/ CapeString name) {
		throw cape_open_error(COBIAERR_Denied);
	}
	void getComponentDescription(/*out*/ CapeString desc) {
		desc = COBIATEXT("Validator Class");
	}
	void putComponentDescription(/*in*/ CapeString desc) {
		throw cape_open_error(COBIAERR_Denied);
	}
};

using ValidatorPtr = CapeOpenObjectSmartPointer<Validator>;