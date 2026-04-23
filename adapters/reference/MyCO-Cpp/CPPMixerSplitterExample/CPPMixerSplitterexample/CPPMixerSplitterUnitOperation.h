// CPPMixerSplitterUnitOperation.h : Declaration of the CCPPMixerSplitterUnitOperation

#pragma once
#include "resource.h"       // main symbols

#include "CPPMixerSplitterexample.h"
#include "CAPEOPENBaseObject.h"
#include "Collection.h"
#include "RealParameter.h"
#include "MaterialPort.h"
#include "EditDialog.h"

#define CURRENTFILEVERSIONNUMBER 0

//! Unit operation implementation class
/*!
  This is the main Unit Operation implementation class.
  Other than the standard interfaces (in CAPEOPENBaseObject)
  this object supplies ICapeUnit, ICapeUtilities and 
  IPersistStream implementations.
  
  This is the only object that will be created externally by 
  CAPE-OPEN simulation environments. Therefore, COM registration 
  of this object is required. The registry entries are in the 
  CPPMixerSplitterUnitOperation.rgs file; the rgs files have 
  been modified for registration in the user part of the 
  registry, rather than in HKCR. Additional registry entries
  for the Unit Operation includes the CATIDs for CAPE-OPEN
  objects and CAPE-OPEN Unit Operations, as well as the 
  CapeDescription key and its values.
  
*/

class ATL_NO_VTABLE CCPPMixerSplitterUnitOperation :
	public CComCoClass<CCPPMixerSplitterUnitOperation, &CLSID_CPPMixerSplitterUnitOperation>,
	public IDispatchImpl<ICapeUnit, &__uuidof(ICapeUnit), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>,
	public IDispatchImpl<ICapeUtilities, &__uuidof(ICapeUtilities), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>,
	public IPersistStream,
	public CAPEOPENBaseObject,
	public IDispatchImpl<ICapeUnitReport, &__uuidof(ICapeUnitReport), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>

{

public:

	//data members

	CapeValidationStatus valStatus; /*!< current valiation status */
	CollectionObject *portCollection; /*!< the port collection */
	CollectionObject *parameterCollection; /*!< the parameter collection */
	IDispatch *simulationContext; /*!< reference to the simulation context, if any */
	int nCompounds; /*!< number of compounds; set at Validate(), used at Calculate() */
	int selectedReportIndex; /*!< index of the currently selected report, -1 if no report selected */

	//! Constructor
	/*!
	Calls the constructor of the CAPEOPENBaseObject base class with name and description initializers
	and initializes variables
	*/

	CCPPMixerSplitterUnitOperation() : 
	CAPEOPENBaseObject(true,
		L"CPP Mixer Splitter Example",
		L"Microsoft Visual C++ 2005 Mixer and Splitter Example according to CAPE-OPEN Unit Operation specification"
		)
	{//init variables
		valStatus=CAPE_NOT_VALIDATED;
		simulationContext=NULL;
		dirty=false;
		selectedReportIndex=-1;
		//create port collection
		portCollection=CCollection::CreateCollection(L"Port collection",L"Port collection for CPP Mixer Splitter");
		//create the ports
		MaterialPortObject *port;
		port=MaterialPortObject::CreateMaterialPort(L"Feed 1",L"Feed port for CPP Mixer Splitter Unit Operation example",CAPE_INLET);
		portCollection->AddItem(port); //item 0
		port=MaterialPortObject::CreateMaterialPort(L"Feed 2",L"Feed port for CPP Mixer Splitter Unit Operation example",CAPE_INLET);
		portCollection->AddItem(port); //item 1
		port=MaterialPortObject::CreateMaterialPort(L"Product 1",L"Product port for CPP Mixer Splitter Unit Operation example",CAPE_OUTLET);
		portCollection->AddItem(port); //item 2
		port=MaterialPortObject::CreateMaterialPort(L"Product 2",L"Product port for CPP Mixer Splitter Unit Operation example",CAPE_OUTLET);
		portCollection->AddItem(port); //item 3
		//create parameter collection
		parameterCollection=CCollection::CreateCollection(L"Parameter collection",L"Parameter collection for CPP Mixer Splitter");
		//create the parameters
		RealParameterObject *par;
		vector<double> dimensionality; //order: m, kg, S, A, K, mole, cd, rad, optionally followed by a delta indicator; trailing zeroes can be omitted
		dimensionality.resize(0); //no dimension for this parameter
		par=RealParameterObject::CreateParameter(L"Split factor",L"Split factor: fraction of product that goes to Product 1 stream",0,1,0.5,dimensionality,&valStatus); 
		parameterCollection->AddItem(par); //parameter 0
		//the dimensionality of heat input is W = J / s = kg m ^2 / s ^3
		dimensionality.resize(3);
		dimensionality[0]=2.0; //m
		dimensionality[1]=1.0; //kg
		dimensionality[2]=-3.0; //s
		double NaN=numeric_limits<double>::quiet_NaN();
		par=RealParameterObject::CreateParameter(L"Heat input",L"Heat input: energy added to the total product",NaN,NaN,0,dimensionality,&valStatus); //no min or max value, we pass NaN
		parameterCollection->AddItem(par); //parameter 1
	}

	//! Destructor
	/*!
	Cleans up variables and objects allocated by this unit operation
	*/

	~CCPPMixerSplitterUnitOperation()
	{	unsigned int i;
		//in case terminate has not been called, clean up references to external objects
		Terminate();
		//clean up ports
		for (i=0;i<portCollection->items.size();i++) ((ICapeIdentification*)portCollection->items[i])->Release(); //it does not matter which Release we use, but we have to cast to one of the implemented interfaces as the C++ compiler wants to know which Release we mean
		//clean up port collection
		portCollection->Release();
		//clean up parameters
		for (i=0;i<parameterCollection->items.size();i++) ((ICapeIdentification*)parameterCollection->items[i])->Release();
		//clean up parameter collection
		parameterCollection->Release();
	}

	//! Registration entry points
	/*!
	The IDR_CPPMIXERSPLITTERUNITOPERATION resource ID maps to the CPPMixerSplitterUnitOperation.rgs resource in resource.h
	*/

	DECLARE_REGISTRY_RESOURCEID(IDR_CPPMIXERSPLITTERUNITOPERATION)

	//! COM Map
	/*!
	These macros tell the underlying ATL COM object how to map from QueryInterface calls interface pointers. The 
	interfaces implemented in the base class are mapped in the BASEMAP macro, and include ICapeIdentification as
	well as the error interfaces
	*/

	BEGIN_COM_MAP(CCPPMixerSplitterUnitOperation)
		COM_INTERFACE_ENTRY2(IDispatch, ICapeUnit)
		COM_INTERFACE_ENTRY(ICapeUnit)
		COM_INTERFACE_ENTRY(ICapeUtilities)
		COM_INTERFACE_ENTRY(IPersistStream)
		BASEMAP
		COM_INTERFACE_ENTRY(ICapeUnitReport)
	END_COM_MAP()


	DECLARE_PROTECT_FINAL_CONSTRUCT()

	// ICapeUnit Methods

	//! ICapeUnit::ports
	/*!
	Returns the port collection for this unit operation.
	\param ports [out, retval] receives the port collection, cannot be NULL
	*/

	STDMETHOD(get_ports)(LPDISPATCH * ports)
	{	if (!ports) return E_POINTER; //invalid pointer
		//instead of doing this, we can also call QueryInterface on the port collection
		*ports=(ICapeCollection*)portCollection;
		portCollection->AddRef(); //the caller must release this object
		//all done
		return NOERROR;
	}

	//! ICapeUnit::get_ValStatus
	/*!
	Returns the validation status.
	\param ValStatus [out, retval] receives the current validation status, cannot be NULL
	*/

	STDMETHOD(get_ValStatus)(CapeValidationStatus * ValStatus)
	{	if (!ValStatus) return E_POINTER; //invalid pointer
		*ValStatus=valStatus;
		return NOERROR;
	}

	//! ICapeUnit::Calculate
	/*!
	Calculate the unit operation. This is the function that performs the actual model calculation.
	\sa Calculate()
	*/

	STDMETHOD(Calculate)()
	{	unsigned int i;
		int j,k;
		double d;
		wstring error; 
		MaterialPortObject *port;
		double pressure; //[Pa]
		double temperature; //[K]
		double phaseFraction; //[mol/mol]
		vector<double> componentFlows; //[mol/s]
		double totalFlow,flow; //[mol/s]
		double enthalpy; //[J/s]
		CVariant value,phaseList;
		Material material,duplicateMaterial;
		//first let us make sure we are in a valid state
		if (valStatus==CAPE_INVALID)
		 {SetError(L"Unit is not valid",L"ICapeUnit",L"Calculate");
		  return ECapeUnknownHR;
		 }
		if (valStatus==CAPE_NOT_VALIDATED)
		 {SetError(L"Unit has not been validated",L"ICapeUnit",L"Calculate");
	   	  return ECapeUnknownHR;
		 }
		ATLASSERT(valStatus==CAPE_VALID);
		//init variables
		componentFlows.resize(nCompounds);
		for (j=0;j<nCompounds;j++) componentFlows[j]=0;
		totalFlow=0; 
		enthalpy=0;
		pressure=0;
		//loop over the connected feed ports, get the minimum pressure and the total component and enthalpy flows
		for (i=0;i<2;i++)
		   {port=(MaterialPortObject *)portCollection->items[i];
			if (port->IsConnected())
			   {//get the pressure
				material=port->GetMaterial();
				if (!material.GetOverallProperty(L"pressure",NULL,value,error))
				   {SetError(error.c_str(),L"ICapeUnit",L"Calculate");
					return ECapeUnknownHR;
				   }
				//check count
				if (value.GetCount()!=1)
				   {SetError(L"Invalid values for pressure from material object: scalar expected",L"ICapeUnit",L"Calculate");
					return ECapeUnknownHR;
				   }
				//use minimum pressure
				d=value.GetDoubleAt(0);
				if ((pressure==0)||(d<pressure)) pressure=d;
				//get total flow
				if (!material.GetOverallProperty(L"totalFlow",L"mole",value,error))
				   {SetError(error.c_str(),L"ICapeUnit",L"Calculate");
					return ECapeUnknownHR;
				   }
				//check count
				if (value.GetCount()!=1)
				   {SetError(L"Invalid values for total flow from material object: scalar expected",L"ICapeUnit",L"Calculate");
					return ECapeUnknownHR;
				   }
				flow=value.GetDoubleAt(0); //flow of this feed
				if (flow>0)
				   {//add to total flow
					totalFlow+=flow;
					//get the composition
					if (!material.GetOverallProperty(L"fraction",L"mole",value,error))
					   {SetError(error.c_str(),L"ICapeUnit",L"Calculate");
						return ECapeUnknownHR;
					   }
					//check count 
					if (value.GetCount()!=nCompounds)
					   {SetError(L"Invalid values for overall fraction from material object: unexpected number of values",L"ICapeUnit",L"Calculate");
						return ECapeUnknownHR;
					   }
					//add to component flows
					for (j=0;j<nCompounds;j++) componentFlows[j]+=flow*value.GetDoubleAt(j); // [mol/s] += [mol/s]*[mol/mol]
					//calculate enthalpy contributions of present phases on duplicate material object
					// (we are not allowed to change the status of material objects connected to the feed, this includes performing property calculations)
					if (!material.Duplicate(duplicateMaterial,error))
					   {SetError(error.c_str(),L"ICapeUnit",L"Calculate");
						return ECapeUnknownHR;
					   }
					//get the list of present phases
					if (!duplicateMaterial.GetListOfPresentPhases(phaseList,error))
					   {SetError(error.c_str(),L"ICapeUnit",L"Calculate");
						return ECapeUnknownHR;
					   }
					//loop over all phases to get phase contribution of enthalpy
					for (k=0;k<phaseList.GetCount();k++)
					   {CBSTR phaseName=phaseList.GetStringAt(k);
						//get the phase fraction for this phase
						if (!duplicateMaterial.GetSinglePhaseProperty(L"phaseFraction",phaseName,NULL,L"mole",value,error))
						   {SetError(error.c_str(),L"ICapeUnit",L"Calculate");
							return ECapeUnknownHR;
						   }
						//check count 
						if (value.GetCount()!=1)
						   {SetError(L"Invalid values for phase fraction from material object: scalar expected",L"ICapeUnit",L"Calculate");
							return ECapeUnknownHR;
						   }
						phaseFraction=value.GetDoubleAt(0);
						if (phaseFraction>0)
						   {//calculate enthalpy for this phase
							if (!duplicateMaterial.CalcSinglePhaseProperty(L"enthalpy",phaseName,error))
							   {SetError(error.c_str(),L"ICapeUnit",L"Calculate");
								return ECapeUnknownHR;
							   }
							//get the value of enthalpy
							if (!duplicateMaterial.GetSinglePhaseProperty(L"enthalpy",phaseName,L"mixture",L"mole",value,error))
							   {SetError(error.c_str(),L"ICapeUnit",L"Calculate");
								return ECapeUnknownHR;
							   }
							//check count 
							if (value.GetCount()!=1)
							   {SetError(L"Invalid values for enthalpy from material object: scalar expected",L"ICapeUnit",L"Calculate");
								return ECapeUnknownHR;
							   }
							//add contribution to total enthalpy
							enthalpy+=flow*phaseFraction*value.GetDoubleAt(0); // [J/s]+=[mol/s]*[mol/mol]*[J/mol]
						   }
					   }
				   }
			   }
		   }
		//we have the feed values, for the remainder of the calculations we need to know the heat input and the split factor
		double splitFactor;
		double heatInput;
		RealParameterObject *par;
		par=(RealParameterObject *)parameterCollection->items[0]; //split factor
		splitFactor=par->value;
		par=(RealParameterObject *)parameterCollection->items[1]; //heat input
		heatInput=par->value;
		//calculate the product composition and temperature
		CVariant composition; //[mol/mol]
		composition.MakeArray(nCompounds,VT_R8);
		if (totalFlow==0)
		   {//we can fail the calculation at this point. It is however best if we can produce an answer that will satisfy the mass
			// and energy balance. The mass balance is satisfied with all zero product flows, at any composition and temperature. The 
			// energy balance can only be satisfied for energy,in = energy,out = 0
			if (heatInput!=0)
			   {SetError(L"Total flow is zero. Cannot satisfy energy balance with non-zero heat input",L"ICapeUnit",L"Calculate");
				return ECapeUnknownHR;
			   }
			//take temperature as the average feed temperature, take composition as average feed composition
			d=0;
			temperature=0;
			for (j=0;j<nCompounds;j++) composition.SetDoubleAt(j,0);
			for (i=0;i<2;i++)
			   {port=(MaterialPortObject *)portCollection->items[i];
				if (port->IsConnected())
				   {//add to division
					d+=1.0;
					//get the temperature
					material=port->GetMaterial();
					if (!material.GetOverallProperty(L"temperature",NULL,value,error))
					   {SetError(error.c_str(),L"ICapeUnit",L"Calculate");
						return ECapeUnknownHR;
					   }
					//check count
					if (value.GetCount()!=1)
					   {SetError(L"Invalid values for temperature from material object: scalar expected",L"ICapeUnit",L"Calculate");
						return ECapeUnknownHR;
					   }
					//add 
					temperature+=value.GetDoubleAt(0);
					//get the composition
					if (!material.GetOverallProperty(L"fraction",L"mole",value,error))
					   {SetError(error.c_str(),L"ICapeUnit",L"Calculate");
						return ECapeUnknownHR;
					   }
					//check count
					if (value.GetCount()!=nCompounds)
					   {SetError(L"Invalid values for overall fraction from material object: unexpected number of values",L"ICapeUnit",L"Calculate");
						return ECapeUnknownHR;
					   }
					//add
					for (j=0;j<nCompounds;j++) composition.SetDoubleAt(j,composition.GetDoubleAt(j)+value.GetDoubleAt(j));
				   }
			   }
			//divide by d, if not unity
			if (d!=1.0)
			   {ATLASSERT(d>0);
				d=1.0/d; //now we multiply by d
				for (j=0;j<nCompounds;j++) composition.SetDoubleAt(j,composition.GetDoubleAt(j)*d);
				temperature*=d;
			   }
		   }
		else
		   {//we have a non-zero total flow; calculate composition
			for (j=0;j<nCompounds;j++) composition.SetDoubleAt(j,componentFlows[j]/totalFlow); //[mol/mol]=[mol/s]/[mol/s]
			//add the work to total enthalpy
			enthalpy+=heatInput;
			//we calculate temperature from a PH flash at a total molar enthalpy of 
			double molarEnthalpy=enthalpy/totalFlow; //[J/mol]=[J/s]/[mol/s]
			//we perform this calculation on a duplicate material. In case of non-zero flow, the duplicate material 
			// should still be set from the enthalpy calculations
			ATLASSERT(duplicateMaterial.IsValid());
			//we can use this material:
			if (!duplicateMaterial.GetTemperatureFromPHFlash(composition,pressure,molarEnthalpy,temperature,error))
			   {SetError(error.c_str(),L"ICapeUnit",L"Calculate");
				return ECapeUnknownHR;
			   }
		   }
		//set the output values; the split factor applies only in case there are two connected product ports. Count them
		int numberOfConnectedProductPorts=0;
		for (i=2;i<4;i++)
		   {port=(MaterialPortObject *)portCollection->items[i];
			if (port->IsConnected()) numberOfConnectedProductPorts++;
		   }
		//loop over the connected outlet ports to set the result
		for (i=2;i<4;i++)
		   {port=(MaterialPortObject *)portCollection->items[i];
			if (port->IsConnected())
			   {material=port->GetMaterial();
				//calc total flow for this stream 
				if (numberOfConnectedProductPorts==2)
				   {//use splitFactor for stream 2, 1-splitFactor for stream 3
					if (i==2) flow=totalFlow*splitFactor;
					else flow=totalFlow*(1.0-splitFactor);
				   }
				else
				   {//use totalFlow
					flow=totalFlow;
				   }
				//set from composition, T and P and perform a flash
				if (!material.SetFromFlowTPX(composition,flow,temperature,pressure,error))
				   {SetError(error.c_str(),L"ICapeUnit",L"Calculate");
					return ECapeUnknownHR;
				   }
			   }
		   }
		//all ok
		return NOERROR;
	}

	//! ICapeUnit::Validate
	/*!
	Validate the unit operation. The simulation environment must ensure that a unit operation is in 
	a valid state before calling Calculate. The purpose of this function is two-fold: 
	-# to signal to the simulation environment that the unit is ready or not ready to be calculated, with a textual
	message describing the problem in case the unit is not ready
	-# to perform any tasks that can be length and do not need to be repeated at each calculation, 
	such as obtaining component constant values

	If the unit is not valid, this should be reflected in the isValid parameter, Validate should not 
	produce an error return just because the unit is not valid.

	Validate must also update the validation status.
	\param message [in, out] textual description of reason for not being valid, if isValid is VARIANT_FALSE
	\param isValid [out, retval]
	\sa Calculate(), get_ValStatus()
	*/

	STDMETHOD(Validate)(BSTR * message, VARIANT_BOOL * isValid)
	{	if ((!message)||(!isValid)) return E_POINTER; //invalid pointer
		//assume innocent, until proven guilty
		*isValid=VARIANT_TRUE;
		//note that message is marked [in, out]; this implies if we put something in it, we should free whatever is in it already. Let us do that now
		if (*message)
		{   //it is not wise of the simulation environment to put something in here and assume that we are indeed freeing it... 
			SysFreeString(*message);
			*message=NULL;
		}
		//we need at least one connected feed and one connected product
		unsigned int i,j;
		int k;
		wstring error,portName1;
		MaterialPortObject *port;
		bool haveConnectedFeed=false,haveConnectedProduct=false;
		CVariant compList1,compList2;
		Material material;
		for (i=0;i<portCollection->items.size();i++)
		   {port=(MaterialPortObject*)portCollection->items[i]; //item is stored as CAPEOPENBaseObject, cast to port
			if (port->IsConnected())
			   {if (i<2) 
				   {//this is a feed
					haveConnectedFeed=true;
				   }
				else
				   {//this is a product
					haveConnectedProduct=true;
				   }
			   }
		   }
		if (!haveConnectedFeed)
		   {*message=SysAllocString(L"At least one feed port must be connected.");
			*isValid=VARIANT_FALSE;
		   }
		else if (!haveConnectedProduct)
		   {*message=SysAllocString(L"At least one product port must be connected.");
			*isValid=VARIANT_FALSE;
		   }
		if (*isValid)
		   {//let us verify that the list of compounds on each port is the same; find the first connected port
			for (i=0;i<portCollection->items.size();i++)
			   {port=(MaterialPortObject*)portCollection->items[i]; //item is stored as CAPEOPENBaseObject, cast to port
				if (port->IsConnected())
				   {//get a material from this port
					material=port->GetMaterial();
					if (!material.GetCompoundIDs(compList1,error))
					   {*message=SysAllocString(error.c_str());
						*isValid=VARIANT_FALSE;
						break; //break out of loop i
					   }
					//store port name in case of error
					portName1=port->name;
					//store the number of compounds for calculation
					nCompounds=compList1.GetCount();
					//loop over all the other ports to compare the list 
					for (j=i+1;j<portCollection->items.size();j++)
					   {port=(MaterialPortObject*)portCollection->items[j]; //item is stored as CAPEOPENBaseObject, cast to port
						if (port->IsConnected())
						   {material=port->GetMaterial();
							if (!material.GetCompoundIDs(compList2,error))
							   {*message=SysAllocString(error.c_str());
								*isValid=VARIANT_FALSE;
								break; //break out of loop j
							   }
							//check same
							bool same=true; 
							if (compList1.GetCount()!=compList2.GetCount()) same=false;
							else
							   {//check each compound
								for (k=0;k<compList1.GetCount();k++)
								   {CBSTR comp1=compList1.GetStringAt(k);
									CBSTR comp2=compList2.GetStringAt(k);
									if (!CBSTR::Same(comp1,comp2))
									   {same=false;
										break;
									   }
								   }
							   }
							if (!same)
							   {//invalid
								error=L"Compound list on material connected to port ";
								error+=portName1;
								error+=L" is not the same as compound list on material connected to port ";
								error+=port->name;
								error+=L'.';
								*message=SysAllocString(error.c_str());
								*isValid=VARIANT_FALSE;
								break; //break out of loop j
							   }
						   }
					   }
					break; //break out of loop i
				   }
			   }
		   }
		if (*isValid)
		   {//this unit needs enthalpy, see if it is available. Get from first connected port
			for (i=0;i<portCollection->items.size();i++)
			   {port=(MaterialPortObject*)portCollection->items[i]; //item is stored as CAPEOPENBaseObject, cast to port
				if (port->IsConnected())
				   {//get a material from this port
					material=port->GetMaterial();
					CVariant propList;
					if (!material.GetSinglePhasePropList(propList,error))
					   {//failed to get the prop list
						*message=SysAllocString(error.c_str());
						*isValid=VARIANT_FALSE;
					   }
					else
					   {//check that enthalpy is in there:
						for (k=0;k<propList.GetCount();k++)
						   {CBSTR prop=propList.GetStringAt(k);
							if (CBSTR::Same(prop,L"enthalpy")) break; //comparison is case-insensitive
						   }
						if (k==propList.GetCount())
						   {//enthalpy is not available (or at least not in this list)
							*message=SysAllocString(L"Property Enthalpy is not available. Enthalpy is required by this unit operation.");
							*isValid=VARIANT_FALSE;
						   }
					   }
					break;
				   }
			   }
		   } 
		//update the validation status:
		valStatus=(*isValid)?CAPE_VALID:CAPE_INVALID;
		return NOERROR;
	}

	// ICapeUtilities Methods

	//! ICapeUtilities::get_parameters
	/*!
	Returns the parameter collection for a unit operation, if any.
	\param parameters [out, retval] receives the parameters collection, cannot be NULL
	*/

	STDMETHOD(get_parameters)(LPDISPATCH * parameters)
	{	if (!parameters) return E_POINTER; //invalid pointer
		//instead of doing this, we can also call QueryInterface on the parameter collection
		*parameters=(ICapeCollection*)parameterCollection;
		parameterCollection->AddRef(); //the caller must release this object
		//all done
		return NOERROR;
	}

	//! ICapeUtilities::put_simulationContext
	/*!
	Specifies a pointer to the simulation context. The simulation context may perform certain tasks that the 
	Unit Operation wants to use, such a logging via ICapeDiagnostics. The Simulation Context is stored, but
	unused by this implementation.
	\param simContext [in] new simulation context. Can be NULL.
	*/

	STDMETHOD(put_simulationContext)(LPDISPATCH simContext)
	{	//release old simulation context
		if (simulationContext)
		   {simulationContext->Release();
			simulationContext=NULL;
		   }
		//set new one 
		simulationContext=simContext;
		if (simulationContext) simulationContext->AddRef(); //we could have also added a reference using QueryInterface
		return NOERROR;
	}

	//! ICapeUtilities::Initialize
	/*!
	Must be called by the simulation environment to initialize the unit operation. Initialize must be called
	after IPersistStream::Load or IPersistStreamInit::Load, if the unit is restored from persistence, and 
	after IPersistStreamInit::InitNew, if implemented. All other calls to the unit operation must be made
	after the Initialize call.
	\sa Terminate()
	*/

	STDMETHOD(Initialize)()
	{	//does nothing
		return NOERROR;
	}

	//! ICapeUtilities::Terminate
	/*!
	Terminate the unit operation. Clean up resources and release any references to objects that are 
	owned by the simulation environment, such as the simulation context and objects connected to 
	ports. Must be the last function called by the simulation environment (in case the object was
	succesfully initialized) before releasing the object
	\sa Initialize()
	*/

	STDMETHOD(Terminate)()
	{	unsigned int i;
		//release the simulation context, if any
		if (simulationContext) 
		   {simulationContext->Release();
			simulationContext=NULL;
		   }
		//disconnect the ports
		for (i=0;i<portCollection->items.size();i++)
		   {MaterialPortObject *port=(MaterialPortObject *)portCollection->items[i]; //item is stored as CAPEOPENBaseObject, cast to port
			port->Disconnect();
		   }
		return NOERROR;
	}

	//! ICapeUtilities::Edit
	/*!
	Unit Operations can be configured via their parameters as exposed from the parameter collection.
	Edit provides another way for configuration of Unit Operations. Edit is typically called in 
	response to the user's request to configure the Unit Operation
	*/

	STDMETHOD(Edit)()
	{	//get references to parameters that we are editing
		RealParameterObject *splitFactor,*heatInput;
		splitFactor=(RealParameterObject *)parameterCollection->items[0];
		heatInput=(RealParameterObject *)parameterCollection->items[1];
		//create edit dialog
		CEditDialog *dlg=new CEditDialog(&splitFactor->value,&heatInput->value);
		dlg->DoModal();
		delete dlg;
		//we are no longer in a validated state
		valStatus=CAPE_NOT_VALIDATED;
		dirty=true; //we need to be saved
		return NOERROR;
	}

	//IPersistStream methods

	//! IPersistStream::IsDirty
	/*!
	Check whether we need to be saved
	\return S_OK if we need to be saved, or S_FALSE
	*/

	STDMETHOD(IsDirty)()
	{//we are dirty if the parameters are dirty as well
		unsigned int i;
		for (i=0;i<parameterCollection->items.size();i++)
		   {RealParameterObject *par=(RealParameterObject *)parameterCollection->items[i];
			if (par->dirty) 
			   {dirty=true;
				break;
			   }
		   }
		return (dirty)?S_OK:S_FALSE;
	}

	//! IPersistStream::Load
	/*!
	Restore from persistence
	\param pstm [in] IStream to load from
	\return S_OK for success, or S_FALSE
	\sa Save()
	*/

	STDMETHOD(Load)(IStream * pstm)
	{   if (!pstm) return E_POINTER;
		UINT fileVersion;
		unsigned int i;
		RealParameterObject *par;
		ULONG read;
		UINT length;
		OLECHAR *buf;
		if (FAILED(pstm->Read(&fileVersion,sizeof(UINT),&read))) return E_FAIL; //this is not a CAPE-OPEN function, we do not return a CAPE-OPEN error
		if (read!=sizeof(UINT)) return E_FAIL;
		if (fileVersion>CURRENTFILEVERSIONNUMBER)
		   {//popping up messages is in general not a good idea; however, this one is a rather important one, so we make an exception here:
			MessageBox(NULL,L"This unit operation was saved with a newer version of the software. Please obtain the latest CPP Mixer Splitter Example from the CO-LaN web site",L"Error loading:",MB_ICONHAND);
			return E_FAIL;
		   }
		//read name
		if (FAILED(pstm->Read(&length,sizeof(UINT),&read))) return E_FAIL; 
		if (read!=sizeof(UINT)) return E_FAIL;
		buf=new OLECHAR[length+1]; //space for terminating zero
		if (FAILED(pstm->Read(buf,2*(length+1),&read))) {delete []buf;return E_FAIL;}
		if (read!=2*(length+1)) {delete []buf;return E_FAIL;}
		name=buf;
		delete []buf;
		//read description
		if (FAILED(pstm->Read(&length,sizeof(UINT),&read))) return E_FAIL; 
		if (read!=sizeof(UINT)) return E_FAIL;
		buf=new OLECHAR[length+1]; //space for terminating zero
		if (FAILED(pstm->Read(buf,2*(length+1),&read))) {delete []buf;return E_FAIL;}
		if (read!=2*(length+1)) {delete []buf;return E_FAIL;}
		description=buf;
		delete []buf;
		//read parameter values
		for (i=0;i<2;i++)
		   {par=(RealParameterObject *)parameterCollection->items[i];
			if (FAILED(pstm->Read(&par->value,sizeof(double),&read))) return E_FAIL; 
			if (read!=sizeof(double)) return E_FAIL;
   		   }
		//all ok 
		return S_OK;
	}

	//! IPersistStream::Save
	/*!
	Save to persistence
	\param pstm [in] IStream to save to
	\param fClearDirty [in] if set, we must clear the dirty flags
	\return S_OK for success, or S_FALSE
	\sa Load(), GetSizeMax()
	*/

	STDMETHOD(Save)(IStream * pstm, BOOL fClearDirty)
	{   if (!pstm) return E_POINTER;
		unsigned int i;
		ULONG written;
		UINT length;
		UINT fileVersion=CURRENTFILEVERSIONNUMBER;
		RealParameterObject *par;
		//save version number, in case of future changes to the format
		if (FAILED(pstm->Write(&fileVersion,sizeof(UINT),&written))) return E_FAIL; //this is not a CAPE-OPEN function, we do not return a CAPE-OPEN error
		if (written!=sizeof(UINT)) return E_FAIL;
		//save name
		length=(UINT)name.size();
		if (FAILED(pstm->Write(&length,sizeof(UINT),&written))) return E_FAIL; 
		if (written!=sizeof(UINT)) return E_FAIL;
		if (FAILED(pstm->Write(name.c_str(),2*(length+1),&written))) return E_FAIL; 
		if (written!=2*(length+1)) return E_FAIL;
		//save description
		length=(UINT)description.size();
		if (FAILED(pstm->Write(&length,sizeof(UINT),&written))) return E_FAIL; 
		if (written!=sizeof(UINT)) return E_FAIL;
		if (FAILED(pstm->Write(description.c_str(),2*(length+1),&written))) return E_FAIL; 
		if (written!=2*(length+1)) return E_FAIL;
		//save parameter values
		for (i=0;i<2;i++)
		   {par=(RealParameterObject *)parameterCollection->items[i];
			if (FAILED(pstm->Write(&par->value,sizeof(double),&written))) return E_FAIL; 
			if (written!=sizeof(double)) return E_FAIL;
		   }
		//clear the dirty flags if so asked
		if (fClearDirty)
		   {dirty=false; 
			//also on the parameters
			for (i=0;i<parameterCollection->items.size();i++)
			   {par=(RealParameterObject *)parameterCollection->items[i];
				par->dirty=false;
			   }
		   }
		return S_OK;
	}

	//! IPersistStream::GetSizeMax
	/*!
	Return the maximum size required to save this object
	\param pcbSize [out] receives the size, cannot be NULL
	\sa Save()
	*/

	STDMETHOD(GetSizeMax)(_ULARGE_INTEGER * pcbSize)
	{   if (!pcbSize) return E_POINTER;
		//calculate total size
		UINT total;
		UINT length;
		total=sizeof(UINT); //version number
		length=(UINT)name.size();
		total+=sizeof(UINT)+2*(length+1); //size and data of name
		length=(UINT)description.size();
		total+=sizeof(UINT)+2*(length+1); //size and data of description
		total+=sizeof(double)*2; //size of values of parameters
		pcbSize->QuadPart=total;
		return NOERROR;
	}

	//! IPersistStream::GetClassID
	/*!
	Return the CLSID of this object
	\param pClassID [out] receives the CLSID, cannot be NULL
	*/

	STDMETHOD(GetClassID)(CLSID *pClassID)
	{   if (!pClassID) return E_POINTER;
		*pClassID=CLSID_CPPMixerSplitterUnitOperation;
		return NOERROR;
	}


	// ICapeUnitReport Methods
	//  this is an optional interface; a sample report is implemented to show how it is done

	//! ICapeUnitReport::get_reports
	/*!
	Return the list of reports
	\param reports [out, retval] receives the list of reports, cannot be NULL
	*/

	STDMETHOD(get_reports)(VARIANT * reports)
	{	if (!reports) return E_POINTER; //not a valid pointer
	    CVariant reportList;
	    reportList.MakeArray(1,VT_BSTR);
	    reportList.AllocStringAt(0,L"Sample report");
	    *reports=reportList.ReturnValue(); //will be freed by caller, so make sure we do not own this value
		return NOERROR;
	}

	//! ICapeUnitReport::get_selectedReport
	/*!
	Return the selected report
	\param report [out, retval] receives the name of the selected report, cannot be NULL
	*/

	STDMETHOD(get_selectedReport)(BSTR * report)
	{	if (!report) return E_POINTER; //not a valid pointer
	    if (selectedReportIndex<0)
	     {SetError(L"A report was not selected",L"ICapeUnitReport",L"get_selectedReport");
	      return ECapeUnknownHR;
	     }
	    //the selected report must be the sample report
	    *report=SysAllocString(L"Sample report"); //will be freed by caller
		return NOERROR;
	}

	//! ICapeUnitReport::put_selectedReport
	/*!
	Select a report
	\param report [in] report to select, must be in the list returned by get_reports
	\sa get_reports()
	*/

	STDMETHOD(put_selectedReport)(BSTR report)
	{	//request to select report
	    if (CBSTR::Same(report,L"Sample report"))
	     {selectedReportIndex=0; 
	      return NOERROR;
	     }
	    //report not supported
	    SetError(L"Invalid report selection: no such report",L"ICapeUnitReport",L"put_selectedReport");
		return ECapeUnknownHR;
	}
	
	//! ICapeUnitReport::ProduceReport
	/*!
	Return the content of the selected report
	\param reportContent [in, out] receives the content of the report; cannot be NULL
	*/
	
	STDMETHOD(ProduceReport)(BSTR * reportContent)
	{	if (!reportContent) return E_POINTER; //not a valid pointer
	    //this is [in, out]; this means we have to free what is currently in there
	    if (*reportContent) 
	     {SysFreeString(*reportContent);
	      *reportContent=NULL;
	     }
	    //did we select a report?
	    if (selectedReportIndex<0)
	     {SetError(L"A report was not selected",L"ICapeUnitReport",L"ProduceReport");
	      return ECapeUnknownHR;
	     }
	    //the selected report must be the sample report, return its content
	    *reportContent=SysAllocString(L"Example Mixer Splitter Report Content"); //caller must free this value
		return NOERROR;
	}
	
};

OBJECT_ENTRY_AUTO(__uuidof(CPPMixerSplitterUnitOperation), CCPPMixerSplitterUnitOperation)
