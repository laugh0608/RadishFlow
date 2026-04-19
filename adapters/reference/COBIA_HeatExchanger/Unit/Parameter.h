/*
* Parameter abstract base class
*/
#pragma once
#include <COBIA.h>

using namespace COBIA;

class Parameter {

protected:
	// Members
	CapeStringImpl& unitName;
	CAPEOPEN_1_2::CapeValidationStatus& unitValidationStatus;
	CapeBoolean& dirty;

	CapeStringImpl paramName;
	CAPEOPEN_1_2::CapeValidationStatus paramValidationStatus;
	CAPEOPEN_1_2::CapeParamMode paramMode;

public:

	const CapeStringImpl getDescriptionForErrorSource() {
		return COBIATEXT("Parameter \"") + paramName + COBIATEXT("\" of ") + unitName;
	}

	Parameter(CapeStringImpl& _unitName, CAPEOPEN_1_2::CapeValidationStatus& _unitValidationStatus,
		CapeBoolean& _dirty, const COBIACHAR* _paramName, CAPEOPEN_1_2::CapeParamMode _paramMode) :
		unitName(_unitName), unitValidationStatus(_unitValidationStatus),
		dirty(_dirty), paramName(_paramName), paramMode(_paramMode) {
		paramValidationStatus = CAPEOPEN_1_2::CAPE_NOT_VALIDATED;
	}

	~Parameter() {
	}

	//CAPEOPEN_1_2::ICapeIdentification
	void getComponentName(/*out*/ CapeString name) {
		name = this->paramName;
	}
	void putComponentName(/*in*/ CapeString name) {
		throw cape_open_error(COBIAERR_Denied);
	}
	void getComponentDescription(/*out*/ CapeString desc) {
		desc = COBIATEXT("Option Parameter");
	}
	void putComponentDescription(/*in*/ CapeString desc) {
		throw cape_open_error(COBIAERR_Denied);
	}
	
	//CAPEOPEN_1_2::ICapeParameter
	CAPEOPEN_1_2::CapeValidationStatus getValStatus() {
		return paramValidationStatus;
	}
	CAPEOPEN_1_2::CapeParamMode getMode() {
		return paramMode;
	}
	virtual CAPEOPEN_1_2::CapeParamType getType() = 0;
	virtual CapeBoolean Validate(/*out*/ CapeString message) = 0;
	virtual void Reset() = 0;
	
};
