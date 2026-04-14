#pragma once
#include <COBIA.h>
#include "MaterialPort.h"
#include "ParameterReal.h"
#include "ParameterOption.h"
#include "Collection.h"
#include "Validator.h"
#include "Solver.h"
#include "Helpers.h"


#ifdef _DEBUG
#ifdef _WIN64
#define unitName COBIATEXT("CO MSHEX x64 Debug")
// Class UUID = AAF02E89-291C-4D7C-836F-10EC28A705A9
#define unitUUID 0xaa,0xf0,0x2e,0x89,0x29,0x1c,0x4d,0x7c,0x83,0x6f,0x10,0xec,0x28,0xa7,0x05,0xa9
#else // _WIN64
#define unitName COBIATEXT("CO MSHEX x86 Debug")
// Class UUID = AAF02E89-291C-4D7C-836F-10EC28A705A8
#define unitUUID 0xaa,0xf0,0x2e,0x89,0x29,0x1c,0x4d,0x7c,0x83,0x6f,0x10,0xec,0x28,0xa7,0x05,0xa8
#endif // _WIN64
#endif // _DEBUG

#ifndef _DEBUG
#ifdef _WIN64
#define unitName COBIATEXT("CO MSHEX x64")
// Class UUID = AAF02E89-291C-4D7C-836F-10EC28A705FF
#define unitUUID 0xaa,0xf0,0x2e,0x89,0x29,0x1c,0x4d,0x7c,0x83,0x6f,0x10,0xec,0x28,0xa7,0x05,0xff
#else // _WIN64
#define unitName COBIATEXT("CO MSHEX x86")
// Class UUID = AAF02E89-291C-4D7C-836F-10EC28A705AA
#define unitUUID 0xaa,0xf0,0x2e,0x89,0x29,0x1c,0x4d,0x7c,0x83,0x6f,0x10,0xec,0x28,0xa7,0x05,0xaa
#endif // _WIN64
#endif // ifndef _DEBUG

#define unitDescription COBIATEXT("MultiStream Heat Exchanger")

using namespace COBIA;

class Unit :
	public CapeOpenObject<Unit>,
	public CAPEOPEN_1_2::CapeIdentificationAdapter<Unit>,
	public CAPEOPEN_1_2::CapeUnitAdapter<Unit>,
	public CAPEOPEN_1_2::CapeUtilitiesAdapter<Unit>,
	public CAPEOPEN_1_2::CapePersistAdapter<Unit> {

	// Members: Identification
	CapeStringImpl name, description;

	// Members: Persistence and Validation
	CapeBoolean dirty;
	CAPEOPEN_1_2::CapeValidationStatus validationStatus;

	// Members: Ports and Parameters
	MaterialPortPtr in1, in2, in3, in4, in5, out1, out2, out3, out4, out5;
	CapeArrayStringImpl sideOptions;
	ParameterOptionPtr in1side, in2side, in3side, in4side, in5side;

	// Members: Collections
	PortCollectionPtr portCollection;
	ParameterCollectionPtr paramCollection;

	// Validator and Solver
	ValidatorPtr validator;
	SolverPtr solver;

	// Flash arguments
	std::vector<CapeArrayStringImpl> productsPhaseIDs;
	std::vector<CapeArrayEnumerationImpl<CAPEOPEN_1_2::CapePhaseStatus>> productsPhaseStatus;
	CapeArrayStringImpl flashCond1, flashCond2;

public:

	// Returns a description of the current object for error handling
	const CapeStringImpl getDescriptionForErrorSource() {
		return COBIATEXT("Unit: ") + name;
	}

	// Registration info
	static const CapeUUID getObjectUUID() {
		return CapeUUID{ unitUUID };
	}

	static void Register(CapePMCRegistrar registrar) {
		registrar.putName(unitName);
		registrar.putDescription(unitDescription);
		registrar.putCapeVersion(COBIATEXT("1.1"));
		registrar.putComponentVersion(COBIATEXT("0.5.0"));
		registrar.putAbout(COBIATEXT("Sample Unit Operation using COBIA."));
		registrar.putVendorURL(COBIATEXT("www.polimi.it"));
		// registrar.putProgId(COBIATEXT("Polimi.Unit"));
		registrar.addCatID(CAPEOPEN::categoryId_UnitOperation);
		registrar.addCatID(CAPEOPEN_1_2::categoryId_Component_1_2);
		//registrar.putCreationFlags(CapePMCRegistationFlag_None);
	}

	// Constructor and members initialisation
	Unit() :
		name(unitName),
		description(unitDescription),
		validationStatus(CAPEOPEN_1_2::CAPE_NOT_VALIDATED),
		dirty(false),
		in1(new MaterialPort(name, validationStatus, COBIATEXT("Inlet 1"), CAPEOPEN_1_2::CAPE_INLET, true)),
		in2(new MaterialPort(name, validationStatus, COBIATEXT("Inlet 2"), CAPEOPEN_1_2::CAPE_INLET, true)),
		in3(new MaterialPort(name, validationStatus, COBIATEXT("Inlet 3"), CAPEOPEN_1_2::CAPE_INLET, false)),
		in4(new MaterialPort(name, validationStatus, COBIATEXT("Inlet 4"), CAPEOPEN_1_2::CAPE_INLET, false)),
		in5(new MaterialPort(name, validationStatus, COBIATEXT("Inlet 5"), CAPEOPEN_1_2::CAPE_INLET, false)),
		out1(new MaterialPort(name, validationStatus, COBIATEXT("Outlet 1"), CAPEOPEN_1_2::CAPE_OUTLET, true)),
		out2(new MaterialPort(name, validationStatus, COBIATEXT("Outlet 2"), CAPEOPEN_1_2::CAPE_OUTLET, true)),
		out3(new MaterialPort(name, validationStatus, COBIATEXT("Outlet 3"), CAPEOPEN_1_2::CAPE_OUTLET, false)),
		out4(new MaterialPort(name, validationStatus, COBIATEXT("Outlet 4"), CAPEOPEN_1_2::CAPE_OUTLET, false)),
		out5(new MaterialPort(name, validationStatus, COBIATEXT("Outlet 5"), CAPEOPEN_1_2::CAPE_OUTLET, false)),
		sideOptions(3),
		in1side(new ParameterOption(name, validationStatus, dirty, COBIATEXT("Inlet 1 Side"), sideOptions)),
		in2side(new ParameterOption(name, validationStatus, dirty, COBIATEXT("Inlet 2 Side"), sideOptions)),
		in3side(new ParameterOption(name, validationStatus, dirty, COBIATEXT("Inlet 3 Side"), sideOptions)),
		in4side(new ParameterOption(name, validationStatus, dirty, COBIATEXT("Inlet 4 Side"), sideOptions)),
		in5side(new ParameterOption(name, validationStatus, dirty, COBIATEXT("Inlet 5 Side"), sideOptions)),
		portCollection(new PortCollection(name)),
		paramCollection(new ParameterCollection(name)),
		validator(new Validator(portCollection, paramCollection, sideOptions)) {

		// Stream Side Options
		sideOptions[0] = COBIATEXT("Ignore");
		sideOptions[1] = COBIATEXT("Hot");
		sideOptions[2] = COBIATEXT("Cold");

		// Add ports to port collection
		// Inlet Stream should be followed by its outlet to reserve indexing.
		// Energy streams should be placed at the end
		portCollection->addItem(in1);
		portCollection->addItem(out1);
		portCollection->addItem(in2);
		portCollection->addItem(out2);
		portCollection->addItem(in3);
		portCollection->addItem(out3);
		portCollection->addItem(in4);
		portCollection->addItem(out4);
		portCollection->addItem(in5);
		portCollection->addItem(out5);

		// Add parameters to port collection
		paramCollection->addItem(in1side);
		paramCollection->addItem(in2side);
		paramCollection->addItem(in3side);
		paramCollection->addItem(in4side);
		paramCollection->addItem(in5side);

		// Prepare T & P flash specifications for products flash
		// specification format:
		// CapeArrayRealImpl = { propertyIdentifier, basis, phaseLabel [, compoundIdentifier] }
		// basis is undefined when it is not a dependency of the property (e.g. T, P)
		flashCond1.resize(3);
		flashCond1[0] = COBIATEXT("temperature");
		flashCond1[2] = COBIATEXT("overall");
		flashCond2.resize(3);
		flashCond2[0] = COBIATEXT("pressure");
		flashCond2[2] = COBIATEXT("overall");
	}

	~Unit() {
	}

	// CAPEOPEN_1_2::ICapeIdentification
	void getComponentName(/*out*/ CapeString name) {
		name = this->name;
	}
	void putComponentName(/*in*/ CapeString name) {
		this->name = name;
		dirty = true;
	}
	void getComponentDescription(/*out*/ CapeString desc) {
		desc = description;
	}
	void putComponentDescription(/*in*/ CapeString desc) {
		description = desc;
		dirty = true;
	}

	// CAPEOPEN_1_2::ICapeUnit
	CAPEOPEN_1_2::CapeCollection<CAPEOPEN_1_2::CapeUnitPort> ports() {
		return portCollection;
	}
	CAPEOPEN_1_2::CapeValidationStatus getValStatus() {
		return validationStatus;
	}
	CapeBoolean Validate(/*out*/ CapeString message) {
		// Only validate if Validation status changed
		if (validationStatus == CAPEOPEN_1_2::CAPE_VALID) { return true; }

		CapeBoolean val = validator->validateParameterSpecifications(message);
		if (val) { val = validator->validateMSHEXSides(message); }
		if (val) { val = validator->validateMaterialPorts(message); }
		if (val) { validator->preparePhaseIDs(productsPhaseIDs, productsPhaseStatus); }
			
		validationStatus = val ? CAPEOPEN_1_2::CAPE_VALID : CAPEOPEN_1_2::CAPE_INVALID;
		return val;
	}
	void Calculate() {
		// Check validation status before calculation
		if (validationStatus != CAPEOPEN_1_2::CAPE_VALID) {
			throw cape_open_error(COBIATEXT("Unit is not in a valid state"));
		}
		// Initiate Solver
		SolverPtr solver = new Solver(portCollection);
		solver->flashProduct(productsPhaseIDs, productsPhaseStatus, flashCond1, flashCond2);
	}

	// CAPEOPEN_1_2::ICapeUtilities
	CAPEOPEN_1_2::CapeCollection<CAPEOPEN_1_2::CapeParameter> getParameters() {
		if (paramCollection != NULL)
		{
			return paramCollection;
		}
		throw cape_open_error(COBIAERR_NotImplemented);
	}
	void putSimulationContext(/*in*/ CAPEOPEN_1_2::CapeSimulationContext context) {
		throw cape_open_error(COBIAERR_NotImplemented);
	}

	void Initialize() {
		// 1. The PME will order the PMC to get initialized through this method.
		// 2. Any initialisation that could fail must be placed here.
		// 3. Initialize is guaranteed to be the first method called by the client
		// (except low level methods such as class constructors or initialization persistence methods).
		// 4. Initialize has to be called once when the PMC is instantiated in a particular flowsheet.
		// 5. When the initialization fails, before signalling an error,
		// the PMC must free all the resources that were allocated before the failure occurred.
		// When the PME receives this error, it may not use the PMC anymore.
		// The method terminate of the current interface must not either be called.
		// Hence, the PME may only release the PMC through the middleware native mechanisms.
	}
	void Terminate() {
		// Disconnect ports
		for (CAPEOPEN_1_2::CapeUnitPort p : portCollection->iterateOverItems()) {
			p.Disconnect();
		}
		// In case a reference to the simulation context is stored, it too must be released at Terminate
	}
	CAPEOPEN_1_2::CapeEditResult Edit(CapeWindowId parent) {
		throw cape_open_error(COBIAERR_NotImplemented);
	}

	//CAPEOPEN_1_2::ICapePersist
	void Save(/*in*/ CAPEOPEN_1_2::CapePersistWriter writer,/*in*/ CapeBoolean clearDirty) {
		writer.Add(ConstCapeString(COBIATEXT("name")), name);
		writer.Add(ConstCapeString(COBIATEXT("description")), description);

		for (CAPEOPEN_1_2::CapeParameter& param : paramCollection->iterateOverItems()) {
			switch (param.getType()) {
			case CAPEOPEN_1_2::CAPE_PARAMETER_REAL:
				writer.Add(getName(param), PARAMREALCAST(param)->getValue());
				break;
			case CAPEOPEN_1_2::CAPE_PARAMETER_STRING: {
				CapeString value(new CapeStringImpl);
				PARAMSTRINGCAST(param)->getValue(value);
				writer.Add(getName(param), value);
				break;
			}
			default:
				throw cape_open_error(COBIAERR_UnknownError);
				break;
			}
		}
		if (clearDirty) {
			dirty = false;
		}
	}
	void Load(/*in*/ CAPEOPEN_1_2::CapePersistReader reader) {
		reader.GetString(ConstCapeString(COBIATEXT("name")), name);
		reader.GetString(ConstCapeString(COBIATEXT("description")), description);
		for (CAPEOPEN_1_2::CapeParameter& param : paramCollection->iterateOverItems()) {
			switch (param.getType()) {
			case CAPEOPEN_1_2::CAPE_PARAMETER_REAL:
				PARAMREALCAST(param)->putValue(reader.GetReal(getName(param)));
				break;
			case CAPEOPEN_1_2::CAPE_PARAMETER_STRING: {
				CapeString value(new CapeStringImpl);
				reader.GetString(getName(param), value);
				PARAMSTRINGCAST(param)->putValue(value);
				break;
			}
			default:
				throw cape_open_error(COBIAERR_UnknownError);
				break;
			}
		}
	}
	CapeBoolean getIsDirty() {
		return dirty;
	}
};
