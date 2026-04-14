// PropertyPackage.cpp : Implementation of CPropertyPackage

#include "stdafx.h"
#include "PropertyPackage.h"
#include "Helpers.h"
#include "PropertyPackageManager.h"
#include "COPropertyNames.h"

//! VECPTR macro
/*!
  Cast a vector to a pointer
  \param vec vector for which to obtain the pointer
  \return Const pointer to element type of vector
*/
  
#define VECPTR(vec) &(vec[0])
  

//! INITPP macro
/*!
  Initialize the property package, if not done already
  \param fnc Name of the function, called from
  \param ifac Name of the interface that fnc belongs to
*/

#define INITPP(fnc,ifac) \
 if (terminated) \
  {SetError(L"Terminate has been called",ifac,fnc); \
   operation=L"N/A"; \
   return ECapeBadInvOrderHR; \
  } \
 if (!pack) \
  {bool res; \
   pack=new PropertyPack(); \
   if (!ppfilename.empty()) \
    res=pack->Load(ppfilename.c_str()); \
   else \
    res=pack->LoadFromPPFile(CT2CA(name.c_str())); \
   if (!res) \
    {string error; \
     error=pack->LastError(); \
     SetError(CA2CT(error.c_str()),ifac,fnc); \
     delete pack; pack=NULL; \
     return ECapeFailedInitialisationHR; \
    } \
  } 

//! ICapeThermoCompounds::GetCompoundConstant
/*!
  Obtain values for the specified constants of the specified compounds
  
  For compound constants that we have some but not all values for, we can 
  return without an error message, but rather return partial results. In 
  this case we should return ECapeThrmPropertyNotAvailableHR. However, in 
  this implementation all values for all supported compound constants should
  be available.
  
  \param props [in] List of properties for which to obtain the constants
  \param compIds [in] List of compounds for which to obtain the constants
  \param propVals [out, retval] Receives the returned constants as a VARIANT array of VARIANT objects
*/

STDMETHODIMP CPropertyPackage::GetCompoundConstant(VARIANT props, VARIANT compIds, VARIANT * propVals)
{	if (!propVals) return E_POINTER;
 	INITPP(L"GetCompoundConstant",L"ICapeThermoCompounds");
 	//get the list of compound indices
 	wstring error;
 	vector<int> compIndices;
 	if (!GetCompounds(compIds,compIndices,TRUE,error))
 	 {SetError(error.c_str(),L"ICapeThermoCompounds",L"GetCompoundConstant");
      return ECapeInvalidArgumentHR;	 
 	 }
	//check the list of properties
	CVariant propList(props,FALSE);
	if (!propList.CheckArray(VT_BSTR,error))
	 {error=L"Invalid list of properties: "+error;
	  SetError(error.c_str(),L"ICapeThermoCompounds",L"GetCompoundConstant");
      return ECapeInvalidArgumentHR;	 
	 }
	//make the result list:  
	CVariant results;
	results.MakeArray((int)compIndices.size()*propList.GetCount(),VT_VARIANT);
	//loop over the compounds
	int index=0; //index into the result array
	CVariant v; //store the result here for a property for a compound 
	//loop over the properties
	// (production implementations should interpret all property names once)
	for (int prop=0;prop<propList.GetCount();prop++)
	 {CBSTR propName=propList.GetStringAt(prop);
      if (!propName)
       {SetError(L"Invalid list of properties: contains at least one empty string",L"ICapeThermoCompounds",L"GetCompoundConstant");
        return ECapeInvalidArgumentHR;	 
       }
	  for (int comp=0;comp<(int)compIndices.size();comp++)
	   {int compIndex=compIndices[comp];
	    double realVal;
	    const char *stringVal;
	    if (propName.Same(L"molecularWeight"))
	     {if (!pack->GetCompoundRealConstant(compIndex,MolecularWeight,realVal))
	       {failedGetValue:
	        error=L"Failed to get \"";
	        error+=propName;
	        error+=L"\": ";
	        error+=CA2CT(pack->LastError());
	        SetError(error.c_str(),L"ICapeThermoCompounds",L"GetCompoundConstant");
            return ECapeComputationHR;	 
	       }
	      v.Set(realVal);
	     }
	    else if (propName.Same(L"criticalTemperature"))
	     {if (!pack->GetCompoundRealConstant(compIndex,CriticalTemperature,realVal)) goto failedGetValue;
	      v.Set(realVal);
	     }
	    else if (propName.Same(L"criticalPressure"))
	     {if (!pack->GetCompoundRealConstant(compIndex,CriticalPressure,realVal)) goto failedGetValue;
	      v.Set(realVal);
	     }
	    else if (propName.Same(L"criticalVolume"))
	     {if (!pack->GetCompoundRealConstant(compIndex,CriticalVolume,realVal)) goto failedGetValue;
	      v.Set(realVal);
	     }
	    else if (propName.Same(L"criticalDensity"))
	     {if (!pack->GetCompoundRealConstant(compIndex,CriticalVolume,realVal)) goto failedGetValue;
	      realVal=1.0/realVal;
	      v.Set(realVal);
	     }
	    else if (propName.Same(L"normalBoilingPoint"))
	     {if (!pack->GetCompoundRealConstant(compIndex,NormalBoilingPoint,realVal)) goto failedGetValue;
	      v.Set(realVal);
	     }
	    else if (propName.Same(L"liquidDensityAt25C"))
	     {if (!pack->GetTemperatureDependentProperty(compIndex,LiquidDensity,298.15,realVal)) goto failedGetValue;
	      v.Set(realVal);
	     }
	    else if (propName.Same(L"liquidVolumeAt25C"))
	     {if (!pack->GetTemperatureDependentProperty(compIndex,LiquidDensity,298.15,realVal)) goto failedGetValue;
	      realVal=1.0/realVal;
	      v.Set(realVal);
	     }
	    else if (propName.Same(L"charge"))
	     {//we assume all compounds have zero charge
	      v.Set(0.0);
	     }
	    else if (propName.Same(L"casRegistryNumber"))
	     {stringVal=pack->GetCompoundStringConstant(compIndex,CASNumber);
	      if (!stringVal) goto failedGetValue;
	      v.Set(stringVal);
	     }
	    else if (propName.Same(L"chemicalFormula"))
	     {stringVal=pack->GetCompoundStringConstant(compIndex,ChemicalFormula);
	      if (!stringVal) goto failedGetValue;
	      v.Set(stringVal);
	     }
	    else
	     {//not supported
	      error=L"Component constant \"";
	      error+=propName;
	      error+=L"\" is not supported";
	      SetError(error.c_str(),L"ICapeThermoCompounds",L"GetCompoundConstant");
	      return ECapeInvalidArgumentHR;	 	     
	     }
	    results.SetVariantAt(index,v.Value()); 
	    index++;
	   }
	 }
	*propVals=results.ReturnValue();
	return NOERROR;
}

//! ICapeThermoCompounds::GetCompoundList
/*!
  Obtain the list of the compounds in this property package, and some of their properties
  \param compIds [in, out] Receives the identifiers for the compounds (this package uses the same as names)
  \param formulae [in, out] Receives the chemical formulae for the compounds
  \param names [in, out] Receives the names of the compounds
  \param boilTemps [in, out] Receives the normal boiling points of the compounds [K]
  \param molwt [in, out] Receives the relative molecular weights of the compounds
  \param casno [in, out] Receives the CAS registry numbers of the compounds
*/

STDMETHODIMP CPropertyPackage::GetCompoundList(VARIANT * compIds, VARIANT * formulae, VARIANT * names, VARIANT * boilTemps, VARIANT * molwt, VARIANT * casno)
{	if ((!compIds)||(!formulae)||(!names)||(!boilTemps)||(!molwt)||(!casno)) return E_POINTER;
 	INITPP(L"GetCompoundList",L"ICapeThermoCompounds");
    //clean out arguments
    VariantClear(compIds);
    VariantClear(formulae);
    VariantClear(names);
    VariantClear(boilTemps);
    VariantClear(molwt);
    VariantClear(casno);
    //alloc space for new compound data
    int i,count;
    CVariant ids,frms,tboil,mwt,cas;
    if (!pack->GetCompoundCount(&count))
     {SetError(CA2CT(pack->LastError()),L"ICapeThermoCompounds",L"GetCompoundList");
      return ECapeComputationHR;
     }
    ids.MakeArray(count,VT_BSTR);
    frms.MakeArray(count,VT_BSTR);
    tboil.MakeArray(count,VT_R8);
    mwt.MakeArray(count,VT_R8);
    cas.MakeArray(count,VT_BSTR);
    for (i=0;i<count;i++)
     {//get name
      const char *string=pack->GetCompoundStringConstant(i,Name);
      if (!string) 
       {//we do not have a name/ID, this is critical
        SetError(CA2CT(pack->LastError()),L"ICapeThermoCompounds",L"GetCompoundList");
        return ECapeComputationHR;
       }
      ids.AllocStringAt(i,CA2CT(string));
      //get formula
      string=pack->GetCompoundStringConstant(i,ChemicalFormula);
      if (string) frms.AllocStringAt(i,CA2CT(string));
      //get cas number 
      string=pack->GetCompoundStringConstant(i,CASNumber);
      if (string) cas.AllocStringAt(i,CA2CT(string));
      //get Tboil
      double d;
      if (!pack->GetCompoundRealConstant(i,NormalBoilingPoint,d)) d=numeric_limits<double>::quiet_NaN();
      tboil.SetDoubleAt(i,d);
      //get mol weight
      if (!pack->GetCompoundRealConstant(i,MolecularWeight,d)) d=numeric_limits<double>::quiet_NaN();
      mwt.SetDoubleAt(i,d);
     }
    //return results
    *compIds=ids.Copy();
    *names=ids.ReturnValue();
    *formulae=frms.ReturnValue();
    *boilTemps=tboil.ReturnValue();
    *molwt=mwt.ReturnValue();
    *casno=cas.ReturnValue();
	return NOERROR;
}

//! ICapeThermoCompounds::GetConstPropList
/*!
  Obtain the list available compound constants
  \param props [out, retval] Receives the list of constant properties supported by this property package
*/

STDMETHODIMP CPropertyPackage::GetConstPropList(VARIANT * props)
{	if (!props) return E_POINTER;
    //production implementations should create this list once and cache it
    CVariant list;
    list.MakeArray(11,VT_BSTR);
    list.AllocStringAt(0,L"molecularWeight");
    list.AllocStringAt(1,L"criticalTemperature");
    list.AllocStringAt(2,L"criticalPressure");
    list.AllocStringAt(3,L"criticalVolume");
    list.AllocStringAt(4,L"criticalDensity");
    list.AllocStringAt(5,L"normalBoilingPoint");
    list.AllocStringAt(6,L"liquidDensityAt25C");
    list.AllocStringAt(7,L"liquidVolumeAt25C");
    list.AllocStringAt(8,L"charge");
    list.AllocStringAt(9,L"casRegistryNumber");
    list.AllocStringAt(10,L"chemicalFormula");
    *props=list.ReturnValue();
	return NOERROR;
}

//! ICapeThermoCompounds::GetNumCompounds
/*!
  Obtain the number of compounds
  \param num [out, retval] Receives the number of compounds in this property package
*/

STDMETHODIMP CPropertyPackage::GetNumCompounds(long * num)
{	if (!num) return E_POINTER;
 	INITPP(L"GetNumCompounds",L"ICapeThermoCompounds");
    //get component count in property package
	int count;
    if (!pack->GetCompoundCount(&count))
     {wstring error;
      error=L"Failed to get component count in property package: ";
      error+=CA2CT(pack->LastError());
      SetError(error.c_str(),L"ICapeThermoCompounds",L"GetNumCompounds");
      return FALSE;
     } 
    *num=count;
	return NOERROR;
}

//! ICapeThermoCompounds::GetPDependentProperty
/*!
  Calculate and get specified pressure dependent properties for the specified compounds at the given pressure
  \param [in] props List of pressure dependent properties to calculate
  \param [in] pressure Pressure at which to calculate the properties [Pa]
  \param [in] compIds List of compounds for which to calculate the properties
  \param propVals [out, retval] Receives the calculated properties as a VARIANT array containing real values
*/

STDMETHODIMP CPropertyPackage::GetPDependentProperty(VARIANT props, double pressure, VARIANT compIds, VARIANT * propVals)
{	if (!propVals) return E_POINTER;
    //this package does not support any pressure dependent properties
    SetError(L"Pressure dependent properties are not supported by this package",L"ICapeThermoCompounds",L"GetPDependentProperty");
    return ECapeNoImplHR;
}

//! ICapeThermoCompounds::GetPDependentPropList
/*!
  Get the list of supported pressure dependent properties 
  \param props [out, retval] Receives the list of supported pressure dependent properties for this property package
*/

STDMETHODIMP CPropertyPackage::GetPDependentPropList(VARIANT * props)
{	if (!props) return E_POINTER;
    props->vt=VT_EMPTY; //we do not expose any pressure dependent properties
	return NOERROR;
}

//! ICapeThermoCompounds::GetTDependentProperty
/*!
  Calculate and get specified temperature dependent properties for the specified compounds at the given temperature
  \param [in] props List of temperature dependent properties to calculate
  \param [in] temperature Temperature at which to calculate the properties [K]
  \param [in] compIds List of compounds for which to calculate the properties
  \param propVals [out, retval] Receives the calculated properties as a VARIANT array containing real values
*/

STDMETHODIMP CPropertyPackage::GetTDependentProperty(VARIANT props, double temperature, VARIANT compIds, VARIANT * propVals)
{	if (!propVals) return E_POINTER;
 	INITPP(L"GetTDependentProperty",L"ICapeThermoCompounds");
 	//get the list of compound indices
 	wstring error;
 	vector<int> compIndices;
 	if (!GetCompounds(compIds,compIndices,TRUE,error))
 	 {SetError(error.c_str(),L"ICapeThermoCompounds",L"GetTDependentProperty");
      return ECapeInvalidArgumentHR;	 
 	 }
	//check the list of properties
	CVariant propList(props,FALSE);
	if (!propList.CheckArray(VT_BSTR,error))
	 {error=L"Invalid list of properties: "+error;
	  SetError(error.c_str(),L"ICapeThermoCompounds",L"GetTDependentProperty");
      return ECapeInvalidArgumentHR;	 
	 }
	//make the result list:  
	CVariant results;
	results.MakeArray((int)compIndices.size()*propList.GetCount(),VT_R8);
	//loop over the compounds
	int index=0; //index into the result array
	CVariant v; //store the result here for a property for a compound 
	//loop over the properties
	// (production implementations should interpret all property names once)
	for (int prop=0;prop<propList.GetCount();prop++)
	 {CBSTR propName=propList.GetStringAt(prop);
	  TDependentProperty propID;
	  int i;
	  for (i=0;i<ExposedTDependentPropertyCount;i++)
	   if (propName.Same(TDependentPropertyNames[i]))
	    {propID=(TDependentProperty)i;
	     break;
	    }
	  if (i==ExposedTDependentPropertyCount)
	   {//not found
	    error=L"Invalid or unsupported temperature dependent property: ";
	    error+=propName;
	    SetError(error.c_str(),L"ICapeThermoCompounds",L"GetTDependentProperty");
        return ECapeInvalidArgumentHR;	 
	   }
	  for (int comp=0;comp<(int)compIndices.size();comp++)
	   {double value;
	    if (!pack->GetTemperatureDependentProperty(compIndices[comp],propID,temperature,value))
	     {//fail
	      string s=pack->LastError();
	      error=L"Failed to get ";
	      error+=propName;
	      error+=L" for compound ";
	      error+=CA2CT(pack->GetCompoundStringConstant(compIndices[comp],Name));
	      error+=L": ";
	      error+=CA2CT(s.c_str());
	      SetError(error.c_str(),L"ICapeThermoCompounds",L"GetTDependentProperty");
          return ECapeComputationHR;	 
	     }
	    results.SetDoubleAt(index,value); 
	    index++;
	   }
	 }
	*propVals=results.ReturnValue();
	return NOERROR;
}

//! ICapeThermoCompounds::GetTDependentPropList
/*!
  Get the list of supported temperature dependent properties 
  \param props [out, retval] Receives the list of supported temperature dependent properties for this property package
*/

STDMETHODIMP CPropertyPackage::GetTDependentPropList(VARIANT * props)
{	if (!props) return E_POINTER;
 	INITPP(L"GetTDependentPropList",L"ICapeThermoCompounds");
 	//production implementations should cache this list
 	CVariant list;
 	int i;
 	list.MakeArray(ExposedTDependentPropertyCount,VT_BSTR);
 	for (i=0;i<ExposedTDependentPropertyCount;i++) list.AllocStringAt(i,TDependentPropertyNames[i]);
 	*props=list.ReturnValue();
	return NOERROR;
}

//! ICapeThermoEquilibriumRoutine::CalcEquilibrium
/*!
  Calculate a phase equilibrium. The list of present phases at the material object determines which phases are 
  allowed to take part in the equilibrium calculcation. After the equilibrium calculation, the list of 
  phases that exists at equilibrium must be set at the material object, along with for each phase the pressure, 
  temperature and composition. The overall pressure and temperature must also be set, if not part of the 
  equilibrium specification
  \param specification1 [in] First equilibrium specification
  \param specification2 [in] Second equilibrium specification
  \param solutionType [in] Solution type for flashes containing a phase fraction; could be "unspecified", "Normal" or "Retrograde"
*/

STDMETHODIMP CPropertyPackage::CalcEquilibrium(VARIANT specification1, VARIANT specification2, BSTR solutionType)
{	INITPP(L"CalcEquilibrium",L"ICapeThermoEquilibriumRoutine");
    HRESULT hr;
    int i,j;
    BOOL mustSetT=TRUE;
    BOOL mustSetP=TRUE;
	double spec1val,spec2val;
	VARIANT composition,v;
	CVariant comp,V,V1;
	BSTR propName;
	BOOL isVapor;
	BSTR basis;
    FlashType type;
    FlashPhaseType phaseType;
    wstring error;
    vector<double> X;
    composition.vt=VT_EMPTY;
    //this routine also checks whether the context material is set, no explicit check required
	if (!GetFlashSpec(specification1,specification2,solutionType,type,phaseType,error)) 
	 {SetError(error.c_str(),L"ICapeThermoEquilibriumRoutine",L"CalcEquilibrium");
	  return ECapeInvalidArgumentHR;
	 }
	if (type==TP)
	 {//get overall T,P,X all at once
	  mustSetT=mustSetP=FALSE;
	  hr=contextMaterial->GetOverallTPFraction(&spec1val,&spec2val,&composition);
	  if (FAILED(hr))
	   {error=L"GetOverallTPFraction failed on context material: ";
	    error+=CO_Error(contextMaterial,hr);
	    SetError(error.c_str(),L"ICapeThermoEquilibriumRoutine",L"CalcEquilibrium");
	    return ECapeUnknownHR;
	   }
	  comp.Set(composition,TRUE); //do not destroy composition
	 } 
	else
	 {//get overall X
	  hr=contextMaterial->GetOverallProp(fraction,mole,&composition);
	  if (FAILED(hr))
	   {error=L"Failed to get overall composition from context material: ";
	    error+=CO_Error(contextMaterial,hr);
	    SetError(error.c_str(),L"ICapeThermoEquilibriumRoutine",L"CalcEquilibrium");
	    return ECapeUnknownHR;
	   }
	  comp.Set(composition,TRUE); //do not destroy composition
	  //get first constraint value
	  switch (type)
	   {default:
	     SetError(L"Internal error: unknown/unimplemented flash type",L"ICapeThermoEquilibriumRoutine",L"CalcEquilibrium");
	     return ECapeUnknownHR;
	    case TVF:
        case TVFm:
         //get temperature
         propName=temperature;
         mustSetT=FALSE;
         break;
        case PVF:
        case PVFm:
        case PH:
        case PS:
         //get pressure
         propName=pressure;
         mustSetP=FALSE;
         break;
	   }
	  v.vt=VT_EMPTY;
	  hr=contextMaterial->GetOverallProp(propName,NULL,&v);
	  if (FAILED(hr))
	   {error=L"Failed to get overall ";
	    error+=propName;
	    error+=L" from context material: ";
	    error+=CO_Error(contextMaterial,hr);
	    SetError(error.c_str(),L"ICapeThermoEquilibriumRoutine",L"CalcEquilibrium");
	    return ECapeUnknownHR;
	   }
	  V.Set(v,TRUE); //do not destroy v
	  if (!V.CheckArray(VT_R8,error,1))
	   {wstring s;
	    s=L"Invalid values for ";
	    s+=propName;
	    s+=L" from context material: ";
	    s+=error;
	    SetError(s.c_str(),L"ICapeThermoEquilibriumRoutine",L"CalcEquilibrium");
	    return ECapeUnknownHR;
	   }
	  spec1val=V.GetDoubleAt(0);
	  //get second constraint value
	  switch (type)
	   {default:
	     SetError(L"Internal error: unknown/unimplemented flash type",L"ICapeThermoEquilibriumRoutine",L"CalcEquilibrium");
	     return ECapeUnknownHR;
	    case TVF:
	    case PVF:
	     //get molar vapor fraction
	     propName=phaseFraction;
	     basis=mole;
	     isVapor=TRUE;
	     break;
        case TVFm:
        case PVFm:
         //get mass vapor fraction
	     propName=phaseFraction;
	     basis=mass;
	     isVapor=TRUE;
         break;
        case PH:
         //get enthalpy
	     propName=enthalpy;
	     basis=mole;
	     isVapor=FALSE;
         break;
        case PS:
         //get entropy
	     propName=entropy;
	     basis=mole;
	     isVapor=FALSE;
         break;
	   }
	  v.vt=VT_EMPTY;
	  if (isVapor) hr=contextMaterial->GetSinglePhaseProp(propName,gas,basis,&v);
	  else hr=contextMaterial->GetOverallProp(propName,basis,&v);
	  if (FAILED(hr))
	   {error=L"Failed to get ";
	    error+=(isVapor)?L"vapor":L"overall";
	    error+=L' ';
	    error+=propName;
	    error+=L" from context material: ";
	    error+=CO_Error(contextMaterial,hr);
	    SetError(error.c_str(),L"ICapeThermoEquilibriumRoutine",L"CalcEquilibrium");
	    return ECapeUnknownHR;
	   }
	  V.Set(v,TRUE); //do not destroy v
	  if (!V.CheckArray(VT_R8,error,1))
	   {wstring s;
	    s=L"Invalid values for ";
	    s+=propName;
	    s+=L" from context material: ";
	    s+=error;
	    SetError(s.c_str(),L"ICapeThermoEquilibriumRoutine",L"CalcEquilibrium");
	    return ECapeUnknownHR;
	   }
	  spec2val=V.GetDoubleAt(0);
	 }
	//check composition
	if (!comp.CheckArray(VT_R8,error,(int)contextMaterialCompoundIndices.size()))
	 {error=L"Invalid values for overall composition from context material: "+error;
	  SetError(error.c_str(),L"ICapeThermoEquilibriumRoutine",L"CalcEquilibrium");
	  return ECapeUnknownHR;
	 }
	//get composition data
	X.resize(contextMaterialCompoundIndices.size());
	for (i=0;i<(int)contextMaterialCompoundIndices.size();i++) X[i]=comp.GetDoubleAt(i);
	//run the flash
	int phaseCount;
	Phase *phases;
	double *phaseFractions;
	double **phaseCompositions;
	double T,P;
	if (!pack->Flash((int)contextMaterialCompoundIndices.size(),VECPTR(contextMaterialCompoundIndices),
	            VECPTR(X),type,phaseType,spec1val,spec2val,phaseCount,phases,phaseFractions,phaseCompositions,T,P))
	 {//flash failed, set error
	  error=L"Flash failed: ";
	  error+=CA2CT(pack->LastError());
	  SetError(error.c_str(),L"ICapeThermoEquilibriumRoutine",L"CalcEquilibrium");
	  return ECapeComputationHR;
	 }
	//process results
	//set list of present phases
	V.MakeArray(phaseCount,VT_BSTR);
	V1.MakeArray(phaseCount,VT_I4); 
	for (j=0;j<phaseCount;j++)
	 {V.SetStringAt(j,(phases[j]==Vapor)?gas:liquid);
	  V1.SetLongAt(j,CAPE_ATEQUILIBRIUM); //this is the status of the phase
	 }
	hr=contextMaterial->SetPresentPhases(V.Value(),V1.Value());
	if (FAILED(hr))
	 {error=L"Failed to set present phases at context material: ";
	  error+=CO_Error(contextMaterial,hr);
	  SetError(error.c_str(),L"ICapeThermoEquilibriumRoutine",L"CalcEquilibrium");
	  return ECapeUnknownHR;
     }	  
	// set T, P, phase fraction and composition for all present phases
	// set overall T and P if not part of the specifications
	for (j=0;j<phaseCount;j++)
	 {BSTR phaseName=(phases[j]==Vapor)?gas:liquid;
	  V.MakeArray((int)contextMaterialCompoundIndices.size(),VT_R8);
	  for (i=0;i<(int)contextMaterialCompoundIndices.size();i++) V.SetDoubleAt(i,phaseCompositions[j][i]);
	  hr=contextMaterial->SetSinglePhaseProp(fraction,phaseName,mole,V.Value());
	  if (FAILED(hr))
	   {error=L"Failed to set ";
	    error+=phaseName;
	    error+=L" composition on material object: ";
	    error+=CO_Error(contextMaterial,hr);
        SetError(error.c_str(),L"ICapeThermoEquilibriumRoutine",L"CalcEquilibrium");
	    return ECapeUnknownHR;
	   }
	  V.MakeArray(1,VT_R8);
	  V.SetDoubleAt(0,phaseFractions[j]);
	  hr=contextMaterial->SetSinglePhaseProp(phaseFraction,phaseName,mole,V.Value());
	  if (FAILED(hr))
	   {error=L"Failed to set ";
	    error+=phaseName;
	    error+=L" fraction on material object: ";
	    error+=CO_Error(contextMaterial,hr);
        SetError(error.c_str(),L"ICapeThermoEquilibriumRoutine",L"CalcEquilibrium");
	    return ECapeUnknownHR;
	   }
	  //set phase pressure
	  V.SetDoubleAt(0,P);
	  hr=contextMaterial->SetSinglePhaseProp(pressure,phaseName,NULL,V.Value());
	  if (FAILED(hr))
	   {error=L"Failed to set ";
	    error+=phaseName;
	    error+=L" pressure on material object: ";
	    error+=CO_Error(contextMaterial,hr);
        SetError(error.c_str(),L"ICapeThermoEquilibriumRoutine",L"CalcEquilibrium");
	    return ECapeUnknownHR;
	   }
	  //set phase temperature
	  V.SetDoubleAt(0,T);
	  hr=contextMaterial->SetSinglePhaseProp(temperature,phaseName,NULL,V.Value());
	  if (FAILED(hr))
	   {error=L"Failed to set ";
	    error+=phaseName;
	    error+=L" temperature on material object: ";
	    error+=CO_Error(contextMaterial,hr);
        SetError(error.c_str(),L"ICapeThermoEquilibriumRoutine",L"CalcEquilibrium");
	    return ECapeUnknownHR;
	   }
	 }
	if (mustSetP)
	 {//set overall P
	  V.SetDoubleAt(0,P); //V is still 1 element long
	  hr=contextMaterial->SetOverallProp(pressure,NULL,V.Value());
	  if (FAILED(hr))
	   {error=L"Failed to set overall pressure on material object: ";
	    error+=CO_Error(contextMaterial,hr);
        SetError(error.c_str(),L"ICapeThermoEquilibriumRoutine",L"CalcEquilibrium");
	    return ECapeUnknownHR;
	   }
	 }
	if (mustSetT)
	 {//set overall T
	  V.SetDoubleAt(0,T); //V is still 1 element long
	  hr=contextMaterial->SetOverallProp(temperature,NULL,V.Value());
	  if (FAILED(hr))
	   {error=L"Failed to set overall temperature on material object: ";
	    error+=CO_Error(contextMaterial,hr);
        SetError(error.c_str(),L"ICapeThermoEquilibriumRoutine",L"CalcEquilibrium");
	    return ECapeUnknownHR;
	   }
	 }
	return NOERROR;
}

//! ICapeThermoEquilibriumRoutine::CheckEquilibriumSpec
/*!
  Check if the property package supports the specified equilibrium calculation. Note that the present phases
  on the material object determine which phases are allowed at equilibrium.
  \param specification1 [in] First equilibrium specification
  \param specification2 [in] Second equilibrium specification
  \param solutionType [in] Solution type for flashes containing a phase fraction; could be "unspecified", "Normal" or "Retrograde"
  \param isSupported [out, retval] Set to VARIANT_TRUE in case the equilibrium calculation is supported, or VARIANT_FALSE otherwise.
*/

STDMETHODIMP CPropertyPackage::CheckEquilibriumSpec(VARIANT specification1, VARIANT specification2, BSTR solutionType, VARIANT_BOOL * isSupported)
{	if (!isSupported) return E_POINTER;
    FlashType type;
    FlashPhaseType phaseType;
    wstring error;
	if (GetFlashSpec(specification1,specification2,solutionType,type,phaseType,error)) *isSupported=VARIANT_TRUE;
	else *isSupported=VARIANT_FALSE; //in case of data we do not process well, we simply respond: not supported
	return NOERROR;
}

//! ICapeThermoMaterialContext::SetMaterial
/*!
  Set a reference to the Material Object that is to be used in the property and equilibrium calculation
  routines.
  \param material [in] Reference to the context material object
*/

STDMETHODIMP CPropertyPackage::SetMaterial(LPDISPATCH material)
{	if (!material)  
     {//we will clear the context material and not return an error, 
      // even though this action should be performed by UnsetMaterial() instead of SetMaterial(NULL)
      contextMaterial=NULL;
      return NOERROR;
     }
    wstring error;
    contextMaterial=material;
    if (!contextMaterial)
     {SetError(L"Failed to get ICapeThermoMaterial interface from material object",L"ICapeThermoMaterialContext",L"SetMaterial");
      return ECapeInvalidArgumentHR;
     }
    //get the compounds on the context material object
    // We know that the compound list is constant in between two calls to SetMaterial. Hence, we can obtain the
    // list of compounds now. Production implementations may want to postpone getting the compound list until it
    // is actually required (as to avoid obtaining a compound list when it is not required) and merely invalidate
    // the stored list of compounds at this point
    ICapeThermoCompoundsPtr compoundInterface(material); //smart pointer, we do not need to release this
    if (!compoundInterface)
     {SetError(L"Failed to get ICapeThermoCompounds interface from material object",L"ICapeThermoMaterialContext",L"SetMaterial");
      contextMaterial=NULL;
      return ECapeInvalidArgumentHR;
     }
    //alas, we only have a method that gets all lists of compound information, while we only need the IDs
    VARIANT compIds,formulae,names,boilTemps,molwts,casnos;
    compIds.vt=VT_EMPTY;
    formulae.vt=VT_EMPTY;
    names.vt=VT_EMPTY;
    boilTemps.vt=VT_EMPTY;
    molwts.vt=VT_EMPTY;
    casnos.vt=VT_EMPTY;
    HRESULT hr=compoundInterface->GetCompoundList(&compIds,&formulae,&names,&boilTemps,&molwts,&casnos);
    if (FAILED(hr))
     {error=L"Failed to get list of compounds from material object: ";
      error+=CO_Error(compoundInterface,hr);
      SetError(error.c_str(),L"ICapeThermoMaterialContext",L"SetMaterial");
      contextMaterial=NULL;
      return ECapeUnknownHR;
     }
    //clear what we are not interested in
    VariantClear(&formulae);
    VariantClear(&names);
    VariantClear(&boilTemps);
    VariantClear(&molwts);
    VariantClear(&casnos);
    //get the list of compounds
    BOOL res=GetCompounds(compIds,contextMaterialCompoundIndices,FALSE,error);
    VariantClear(&compIds); //now all data is freed
    if (!res)
     {SetError(error.c_str(),L"ICapeThermoMaterialContext",L"SetMaterial");
      contextMaterial=NULL;
      return ECapeUnknownHR;
     }
	return NOERROR;
}

//! ICapeThermoMaterialContext::SetMaterial
/*!
  Remove the reference to the context material object
*/

STDMETHODIMP CPropertyPackage::UnsetMaterial()
{   contextMaterial=NULL;
	return NOERROR;
}

//! ICapeThermoPhases::GetNumPhases
/*!
  Retrieve the number of phases supported by this property package
  \param num [out, retval] Receives the number of phases in the property package
*/

STDMETHODIMP CPropertyPackage::GetNumPhases(long * num)
{	if (!num) return E_POINTER;
    *num=2; //gas and liquid
	return NOERROR;
}

//! ICapeThermoPhases::GetPhaseInfo
/*!
  Get string property of a specified phase
  \param phaseLabel [in] Phase for which to obtain the property
  \param phaseAttribute [in] Property to obtain
  \param value [out, retval] Receives the requested property
*/

STDMETHODIMP CPropertyPackage::GetPhaseInfo(BSTR phaseLabel, BSTR phaseAttribute, VARIANT * value)
{	//get attributes of a phase; the following are predefined attributes:
    // StateOfAggregation, KeyCompoundId, ExcludedCompoundId, DensityDescription, UserDescription, TypeOfSolid
    //for other attributes, we will simply return an empty string. 
    //we only support StateOfAggregation, UserDescription
    value->vt=VT_EMPTY;
    if (CBSTR::Same(phaseLabel,gas))
     {//gas phase
      if (CBSTR::Same(phaseAttribute,L"StateOfAggregation"))
       {value->vt=VT_BSTR; 
        value->bstrVal=SysAllocString(L"Vapor");
       }
      else if (CBSTR::Same(phaseAttribute,L"UserDescription"))
       {value->vt=VT_BSTR; 
        value->bstrVal=SysAllocString(L"Ideal gas phase");
       }
     }
    else if (CBSTR::Same(phaseLabel,liquid))
     {//liquid phase
      if (CBSTR::Same(phaseAttribute,L"StateOfAggregation"))
       {value->vt=VT_BSTR; 
        value->bstrVal=SysAllocString(L"Liquid");
       }
      else if (CBSTR::Same(phaseAttribute,L"UserDescription"))
       {value->vt=VT_BSTR; 
        value->bstrVal=SysAllocString(L"Ideal liquid phase");
       }
     }
    else
     {//undefined phase
      SetError(L"Undefined phase",L"ICapeThermoPhases",L"GetPhaseInfo");
      return ECapeInvalidArgumentHR;
     }
	return NOERROR;
}

//! ICapeThermoPhases::GetPhaseList
/*!
  Get the list of phases supported by the property package
  
  In version 1.0 we were resticted to phase IDs that started with Vapor, Liquid or Solid. In version 
  1.1 however, there is a difference between a phase label and its state of aggregation. We are therefore
  free to choose what we name a phase. To demonstrate this, we name the vapor phase: gas.
  
  \param phaseLabels [in, out] Receives the labels (IDs) of the supported phases
  \param stateOfAggregation [in, out] Receives the aggregation state of the supported phases. Valid values include Vapor, Liquid and Solid
  \param keyCompoundId [in, out] Receives the compound IDs of the key compounds of the supported phases, if any (e.g. Water for an aqueous phase)
*/

STDMETHODIMP CPropertyPackage::GetPhaseList(VARIANT * phaseLabels, VARIANT * stateOfAggregation, VARIANT * keyCompoundId)
{	if ((!phaseLabels)||(!stateOfAggregation)||(!keyCompoundId)) return E_POINTER;
    //these are [in, out]
    VariantClear(phaseLabels);
    VariantClear(stateOfAggregation);
    VariantClear(keyCompoundId);
    //production implementations should cache these lists
    CVariant labels,aggStates,keyComps;
    labels.MakeArray(2,VT_BSTR);
    aggStates.MakeArray(2,VT_BSTR);
    keyComps.MakeArray(2,VT_BSTR); //as we have no data for it, we do not fill this in. This will contain NULL values therefore, which is equivalent to empty strings
    //phase 1: gas phase
    labels.SetStringAt(0,gas);
    aggStates.AllocStringAt(0,L"Vapor"); //this is the aggregation state, which must be Vapor or Liquid (or Solid, ...)
    //phase 2: liquid phase
    labels.SetStringAt(1,liquid);
    aggStates.AllocStringAt(1,L"Liquid"); 
    //set results
    *phaseLabels=labels.ReturnValue();
    *stateOfAggregation=aggStates.ReturnValue();
    *keyCompoundId=keyComps.ReturnValue();
	return NOERROR;
}

//! ICapeThermoPropertyRoutine::CalcAndGetLnPhi
/*!
  Calculate and return values of log fugacity coefficient and/or a selection of its derivatives for a specified phase at specified conditions
  \param phaseLabel [in] Phase for which to calculate the requested properties
  \param temperature [in] Temperature at which to calculate the requested properties [K]
  \param pressure [in] Pressure at which to calculate the requested properties [Pa]
  \param moleNumbers [in] Composition at which to calculate the requested properties; these should be interpreted as mole fractions [mol/mol]
  \param fFlags [in] Bit-field specifying which property calculations are requested. Could include CAPE_LOG_FUGACITY_COEFFICIENTS, CAPE_T_DERIVATIVE, CAPE_P_DERIVATIVE and/or CAPE_MOLE_NUMBERS_DERIVATIVES
  \param lnPhi [in, out] Recieves the log fugacity coefficients
  \param lnPhiDT [in, out] Recieves the temperature derivatives of the log fugacity coefficients [1/K]
  \param lnPhiDP [in, out] Recieves the pressure derivatives of the log fugacity coefficients [1/Pa]
  \param lnPhiDn [in, out] Recieves the mole number derivatives of the log fugacity coefficients [1/mol]
*/

STDMETHODIMP CPropertyPackage::CalcAndGetLnPhi(BSTR phaseLabel, double temperature, double pressure, VARIANT moleNumbers, int fFlags, VARIANT * lnPhi, VARIANT * lnPhiDT, VARIANT * lnPhiDP, VARIANT * lnPhiDn)
{	//we check the arguments as we need them for this routine
    int i,j;
	if (!fFlags) return NOERROR; //nothing to do
    vector<SinglePhaseProperty> props;
    vector<VARIANT*> resultVars;
    if (fFlags&CAPE_LOG_FUGACITY_COEFFICIENTS)
     {if (!lnPhi) return E_POINTER;
      resultVars.push_back(lnPhi);
      props.push_back(LogFugacityCoefficient);
     }
    if (fFlags&CAPE_T_DERIVATIVE)
     {if (!lnPhiDT) return E_POINTER;
      resultVars.push_back(lnPhiDT);
      props.push_back(LogFugacityCoefficientDT);
     }
    if (fFlags&CAPE_P_DERIVATIVE)
     {if (!lnPhiDP) return E_POINTER;
      resultVars.push_back(lnPhiDP);
      props.push_back(LogFugacityCoefficientDP);
     }
    if (fFlags&CAPE_MOLE_NUMBERS_DERIVATIVES)
     {if (!lnPhiDn) return E_POINTER;
      resultVars.push_back(lnPhiDn);
      props.push_back(LogFugacityCoefficientDn);
     }
    INITPP(L"CalcAndGetLnPhi",L"ICapeThermoPropertyRoutine"); 
    //check context material
    if (!contextMaterial)
     {SetError(L"Context material object not set",L"ICapeThermoPropertyRoutine",L"CalcAndGetLnPhi");
      operation=L"SetMaterial";
      return ECapeBadInvOrderHR;
     }
    //check the composition.
    // Note, despite its name, the moleNumbers must be interpreted as composition; for 
    // lnPhi, lnPhiDT and lnPhiDP this does not matter, but for lnPhiDn this means that 
    // the results are to be returned for a total of 1 mole of substance.
    wstring error;
    CVariant composition(moleNumbers,FALSE); //we do not own this value
    if (!composition.CheckArray(VT_R8,error,(int)contextMaterialCompoundIndices.size()))
     {SetError(error.c_str(),L"ICapeThermoPropertyRoutine",L"CalcAndGetLnPhi");
      return ECapeInvalidArgumentHR;
     }
    vector<double> X;
    X.resize(contextMaterialCompoundIndices.size());
    for (i=0;i<(int)contextMaterialCompoundIndices.size();i++) X[i]=composition.GetDoubleAt(i);
    //check the phase
    Phase phase;
    if (CBSTR::Same(phaseLabel,gas)) phase=Vapor;
    else if (CBSTR::Same(phaseLabel,liquid)) phase=Liquid;
    else 
     {error=L"Invalid/undefined phase: ";
      error+=(phaseLabel)?phaseLabel:L"<NULL>";
      SetError(error.c_str(),L"ICapeThermoPropertyRoutine",L"CalcAndGetLnPhi");
      return ECapeInvalidArgumentHR;
     }
    //perform the calculations
    int *valueCount;
    double **values;
    if (!pack->GetSinglePhaseProperties((int)contextMaterialCompoundIndices.size(),VECPTR(contextMaterialCompoundIndices),
                                  phase,temperature,pressure,VECPTR(X),(int)props.size(),VECPTR(props),valueCount,values))
     {error=L"Property calculation failed: ";
      error+=CA2CT(pack->LastError());
      return ECapeComputationHR;
     }
    //assign values to return codes
    CVariant var;    
	for (j=0;j<(int)props.size();j++)
     {VariantClear(resultVars[j]); //[in, out]
      var.MakeArray(valueCount[j],VT_R8);
      for (i=0;i<valueCount[j];i++) var.SetDoubleAt(i,values[j][i]);
      *resultVars[j]=var.ReturnValue();
     }
	return NOERROR;
}

//! ICapeThermoPropertyRoutine::CalcSinglePhaseProp
/*!
  Calculate the requested single phase properties for the requested phase. The calculation conditions
  are available from the context material. The calculated properties must be set at the context material.
  \param props [in] List of properties to be calculated
  \param phaseLabel [in] Phase for which to calculate the requested properties
*/

STDMETHODIMP CPropertyPackage::CalcSinglePhaseProp(VARIANT props, BSTR phaseLabel)
{	INITPP(L"CalcSinglePhaseProp",L"ICapeThermoPropertyRoutine"); 
    int i,j;
    HRESULT hr;
    wstring error;
    //check the phase
    Phase phase;
    if (CBSTR::Same(phaseLabel,gas)) phase=Vapor;
    else if (CBSTR::Same(phaseLabel,liquid)) phase=Liquid;
    else
     {error=L"Invalid/unsupported phase: ";
      error+=(phaseLabel)?phaseLabel:L"<NULL>";
      SetError(error.c_str(),L"ICapeThermoPropertyRoutine",L"CalcSinglePhaseProp");
      return ECapeInvalidArgumentHR;
     }
    //check the list of properties:
    CVariant propList(props,FALSE); //we do not own this value
    if (!propList.CheckArray(VT_BSTR,error))
     {error=L"Invalid list of properties: "+error;
      SetError(error.c_str(),L"ICapeThermoPropertyRoutine",L"CalcSinglePhaseProp");
      return ECapeInvalidArgumentHR;
     }
    //look up the properties (production implementations should use a hash table)
    vector<SinglePhaseProperty> calcprops;
    vector<CBSTR> propNames;
    calcprops.resize(propList.GetCount());
    propNames.resize(propList.GetCount());
    for (i=0;i<propList.GetCount();i++)
     {propNames[i].SetFromBSTR(propList.GetBSTRAt(i)); //propNames[i] now owns the BSTR value; this construction prevents re-allocation of the BSTR value, which would be the csae from propNames[i]=propList.GetStringAt(i)
	  for (j=0;j<SinglePhasePropertyCount;j++)
	   if (propNames[i].Same(SinglePhasePropertyNames[j]))
	    {calcprops[i]=(SinglePhaseProperty)j;
	     break;
	    }
	  if (j==SinglePhasePropertyCount)
	   {//property not found
	    error=L"Invalid/unsupported single phase property: ";
	    error+=(propNames[i])?propNames[i]:L"<NULL>";
        SetError(error.c_str(),L"ICapeThermoPropertyRoutine",L"CalcSinglePhaseProp");
        return ECapeInvalidArgumentHR;
       }
     }
    //check the context material
    if (!contextMaterial)
     {SetError(L"Context material has not been set",L"ICapeThermoPropertyRoutine",L"CalcSinglePhaseProp");
      operation=L"SetMaterial";
      return ECapeBadInvOrderHR;
     }
    //get T,P,composition from the context material
    double T,P;
    VARIANT composition;
    composition.vt=VT_EMPTY;
    hr=contextMaterial->GetTPFraction(phaseLabel,&T,&P,&composition);
    if (FAILED(hr))
     {error=L"Failed to get calculation conditions from context material: ";
      error+=CO_Error(contextMaterial,hr);
      SetError(error.c_str(),L"ICapeThermoPropertyRoutine",L"CalcSinglePhaseProp");
      return ECapeUnknownHR;
     }
    //check composition
    CVariant comp(composition,TRUE); //don't free composition after this
    if (!comp.CheckArray(VT_R8,error,(int)contextMaterialCompoundIndices.size()))
     {error=L"Invalid composition from context material: "+error;
      error+=CO_Error(contextMaterial,hr);
      SetError(error.c_str(),L"ICapeThermoPropertyRoutine",L"CalcSinglePhaseProp");
      return ECapeUnknownHR;
     }
    //make into array
    vector<double> X;
    X.resize(contextMaterialCompoundIndices.size());
    for (i=0;i<(int)contextMaterialCompoundIndices.size();i++) X[i]=comp.GetDoubleAt(i);
    //calc properties
    int *valueCount;
    double **values;
    if (!pack->GetSinglePhaseProperties((int)contextMaterialCompoundIndices.size(),VECPTR(contextMaterialCompoundIndices),
                                       phase,T,P,VECPTR(X),(int)calcprops.size(),VECPTR(calcprops),valueCount,values))
     {error=L"Property calculations failed: ";
      error+=CA2CT(pack->LastError());
      SetError(error.c_str(),L"ICapeThermoPropertyRoutine",L"CalcSinglePhaseProp");
      return ECapeComputationHR;
     }
    //set the values back on the MO
    for (i=0;i<(int)calcprops.size();i++)
     {CVariant vals; //production implementations could re-use pre-allocated arrays (i.e. for scalars, one for the size of number of components, ...)
      vals.MakeArray(valueCount[i],VT_R8);  // ... rather than re-allocating for each property (at each call to CalcSinglePhaseProp)
      for (j=0;j<valueCount[i];j++) vals.SetDoubleAt(j,values[i][j]);
      BSTR basis=(SinglePhasePropertyMoleBasis[calcprops[i]])?mole:NULL;
      hr=contextMaterial->SetSinglePhaseProp(propNames[i],phaseLabel,basis,vals.Value());
      if (FAILED(hr))
       {error=L"Failed to set ";
        error+=propNames[i];
        error+=L" at context material: ";
        error+=CO_Error(contextMaterial,hr);
        SetError(error.c_str(),L"ICapeThermoPropertyRoutine",L"CalcSinglePhaseProp");
        return ECapeUnknownHR;
       }
     }
	return NOERROR;
}

//! ICapeThermoPropertyRoutine::CalcTwoPhaseProp
/*!
  Calculate the requested two-phase properties for the requested phase. The calculation conditions
  are available from the context material. The calculated properties must be set at the context material.
  \param props [in] List of properties to be calculated
  \param phaseLabels [in] Phase pair (list of two phases) for which to calculate the requested properties
*/

STDMETHODIMP CPropertyPackage::CalcTwoPhaseProp(VARIANT props, VARIANT phaseLabels)
{	INITPP(L"CalcTwoPhaseProp",L"ICapeThermoPropertyRoutine"); 
    int i,j,k;
    HRESULT hr;
    wstring error;
    Phase phase1,phase2;
    //check the phases
    CVariant phaseList(phaseLabels,FALSE); //we do not own this value
    if (!phaseList.CheckArray(VT_BSTR,error,2))
     {error=L"Invalid phase labels: "+error;
      SetError(error.c_str(),L"ICapeThermoPropertyRoutine",L"CalcTwoPhaseProp");
      return ECapeInvalidArgumentHR;
     }
    CBSTR phaseName; 
	//phase 1
    phaseName.SetFromBSTR(phaseList.GetBSTRAt(0));
    if (phaseName.Same(gas)) phase1=Vapor;
    else if (phaseName.Same(liquid)) phase1=Liquid;
    else
     {error=L"Invalid/unsupported phase: ";
      error+=(phaseName)?phaseName:L"<NULL>";
      SetError(error.c_str(),L"ICapeThermoPropertyRoutine",L"CalcTwoPhaseProp");
      return ECapeInvalidArgumentHR;
     }
    //phase 2
    phaseName.SetFromBSTR(phaseList.GetBSTRAt(1));
    if (phaseName.Same(gas)) phase2=Vapor;
    else if (phaseName.Same(liquid)) phase2=Liquid;
    else
     {error=L"Invalid/unsupported phase: ";
      error+=(phaseName)?phaseName:L"<NULL>";
      SetError(error.c_str(),L"ICapeThermoPropertyRoutine",L"CalcTwoPhaseProp");
      return ECapeInvalidArgumentHR;
     }
    //check the list of properties:
    CVariant propList(props,FALSE); //we do not own this value
    if (!propList.CheckArray(VT_BSTR,error))
     {error=L"Invalid list of properties: "+error;
      SetError(error.c_str(),L"ICapeThermoPropertyRoutine",L"CalcTwoPhaseProp");
      return ECapeInvalidArgumentHR;
     }
    //look up the properties (production implementations should use a hash table)
    vector<TwoPhaseProperty> calcprops;
    vector<CBSTR> propNames;
    calcprops.resize(propList.GetCount());
    propNames.resize(propList.GetCount());
    for (i=0;i<propList.GetCount();i++)
     {propNames[i].SetFromBSTR(propList.GetBSTRAt(i)); //propNames[i] now owns the BSTR value; this construction prevents re-allocation of the BSTR value, which would be the csae from propNames[i]=propList.GetStringAt(i)
	  for (j=0;j<TwoPhasePropertyCount;j++)
	   if (propNames[i].Same(TwoPhasePropertyNames[j]))
	    {calcprops[i]=(TwoPhaseProperty)j;
	     break;
	    }
	  if (j==TwoPhasePropertyCount)
	   {//property not found
	    error=L"Invalid/unsupported two-phase property: ";
	    error+=(propNames[i])?propNames[i]:L"<NULL>";
        SetError(error.c_str(),L"ICapeThermoPropertyRoutine",L"CalcTwoPhaseProp");
        return ECapeInvalidArgumentHR;
       }
     }
    //check the context material
    if (!contextMaterial)
     {SetError(L"Context material has not been set",L"ICapeThermoPropertyRoutine",L"CalcTwoPhaseProp");
      operation=L"SetMaterial";
      return ECapeBadInvOrderHR;
     }
    //get T,P,composition from the context material, for each phase
    double T1,P1,T2,P2;
    vector<double> X1,X2;
    for (k=0;k<2;k++)
     {double *T=(k)?&T2:&T1;
      double *P=(k)?&P2:&P1;
      vector<double> *X=(k)?&X2:&X1;
      phaseName.SetFromBSTR(phaseList.GetBSTRAt(k)); //production implementations should prevent obtaining the phase name more than once (this is the second time)
      VARIANT composition;
      composition.vt=VT_EMPTY;
      hr=contextMaterial->GetTPFraction(phaseName,T,P,&composition);
      if (FAILED(hr))
       {error=L"Failed to get calculation conditions from context material: ";
        error+=CO_Error(contextMaterial,hr);
        SetError(error.c_str(),L"ICapeThermoPropertyRoutine",L"CalcTwoPhaseProp");
        return ECapeUnknownHR;
       }
      //check composition
      CVariant comp(composition,TRUE); //don't free composition after this
      if (!comp.CheckArray(VT_R8,error,(int)contextMaterialCompoundIndices.size()))
       {error=L"Invalid composition from context material: "+error;
        error+=CO_Error(contextMaterial,hr);
        SetError(error.c_str(),L"ICapeThermoPropertyRoutine",L"CalcTwoPhaseProp");
        return ECapeUnknownHR;
       }
      //make into array
      X->resize(contextMaterialCompoundIndices.size());
      for (i=0;i<(int)contextMaterialCompoundIndices.size();i++) (*X)[i]=comp.GetDoubleAt(i);
     }
    //calc properties
    int *valueCount;
    double **values;
    if (!pack->GetTwoPhaseProperties((int)contextMaterialCompoundIndices.size(),VECPTR(contextMaterialCompoundIndices),
                                       phase1,phase2,T1,T2,P1,P2,VECPTR(X1),VECPTR(X2),(int)calcprops.size(),VECPTR(calcprops),
                                       valueCount,values))
     {error=L"Property calculations failed: ";
      error+=CA2CT(pack->LastError());
      SetError(error.c_str(),L"ICapeThermoPropertyRoutine",L"CalcTwoPhaseProp");
      return ECapeComputationHR;
     }
    //set the values back on the MO
    for (i=0;i<(int)calcprops.size();i++)
     {CVariant vals; //production implementations could re-use pre-allocated arrays (i.e. for scalars, one for the size of number of components, ...)
      vals.MakeArray(valueCount[i],VT_R8);  // ... rather than re-allocating for each property (at each call to CalcTwoPhaseProp)
      for (j=0;j<valueCount[i];j++) vals.SetDoubleAt(j,values[i][j]);
      hr=contextMaterial->SetTwoPhaseProp(propNames[i],phaseLabels,NULL,vals.Value());
      if (FAILED(hr))
       {error=L"Failed to set ";
        error+=propNames[i];
        error+=L" at context material: ";
        error+=CO_Error(contextMaterial,hr);
        SetError(error.c_str(),L"ICapeThermoPropertyRoutine",L"CalcTwoPhaseProp");
        return ECapeUnknownHR;
       }
     }
	return NOERROR;
}

//! ICapeThermoPropertyRoutine::CalcSinglePhaseProp
/*!
  Check if a single phase property can be calculated. Conditions at the context material should not be 
  inspected for this, hence only the property and phase identifiers should be checked.
  \param property [in] Property to check
  \param phaseLabel [in] Phase for which to check the property
  \param valid [out, retval] Receives VARIANT_TRUE if the property calculation is supported, VARIANT_FALSE otherwise
*/

STDMETHODIMP CPropertyPackage::CheckSinglePhasePropSpec(BSTR property, BSTR phaseLabel, VARIANT_BOOL * valid)
{	if (!valid) return E_POINTER;
	INITPP(L"CheckSinglePhasePropSpec",L"ICapeThermoPropertyRoutine"); 
	//check property (production implementations should use a hash table lookup)
	*valid=VARIANT_TRUE; //set to false if no support
	int i;
	for (i=0;i<SinglePhasePropertyCount;i++)
	 if (CBSTR::Same(property,SinglePhasePropertyNames[i]))
	  break;
	if (i==SinglePhasePropertyCount)
	 {//not found
	  *valid=VARIANT_FALSE;
	 }
	else
	 {//check phase 
	  if (!CBSTR::Same(phaseLabel,liquid))
	   {if (CBSTR::Same(phaseLabel,gas))
	     {//gas, we do not support activity and its derivatives
	      switch(i)
	       {case Activity:
	        case ActivityDT:
	        case ActivityDP:
	        case ActivityDX:
	        case ActivityDn:
	          *valid=VARIANT_FALSE;
	          break;
	       }
	     }
	    else 
	     {//invalid phase (this is an error, but we will just return not supported
	      *valid=VARIANT_FALSE;
	     }
	   }
	 }
	return NOERROR;
}

//! ICapeThermoPropertyRoutine::CheckTwoPhasePropSpec
/*!
  Check if a two-phase property can be calculated. Conditions at the context material should not be 
  inspected for this, hence only the property and phase pair should be checked.
  \param property [in] Property to check
  \param phaseLabels [in] Phase pair (list of two phases) for which to check the property
  \param valid [out, retval] Receives VARIANT_TRUE if the property calculation is supported, VARIANT_FALSE otherwise
*/

STDMETHODIMP CPropertyPackage::CheckTwoPhasePropSpec(BSTR property, VARIANT phaseLabels, VARIANT_BOOL * valid)
{	if (!valid) return E_POINTER;
	INITPP(L"CheckTwoPhasePropSpec",L"ICapeThermoPropertyRoutine"); 
	//check property (production implementations should use a hash table lookup)
	*valid=VARIANT_TRUE; //set to false if no support
	int i;
	for (i=0;i<TwoPhasePropertyCount;i++)
	 if (CBSTR::Same(property,TwoPhasePropertyNames[i]))
	  break;
	if (i==TwoPhasePropertyCount)
	 {//not found
	  *valid=VARIANT_FALSE;
	 }
	else
	 {//check phases; both vapor and liquid should be present
	  CVariant phases(phaseLabels,FALSE); //we do not own this value
	  wstring error;
	  if (!phases.CheckArray(VT_BSTR,error,2))
	   {//this is an error, but we just return not supported
	    *valid=VARIANT_FALSE;
	   }
	  else
	   {//check that they are gas and liquid, or liquid and gas (these are the supported combinations)
	    CBSTR phase1,phase2;
	    phase1=phases.GetStringAt(0);
	    phase2=phases.GetStringAt(1);
	    if (!(  ((phase1.Same(gas))&&(phase2.Same(liquid))) ||
	            ((phase2.Same(gas))&&(phase1.Same(liquid)))
	           )) *valid=VARIANT_FALSE;
	   }
	 }
	return NOERROR;
}

//! ICapeThermoPropertyRoutine::GetSinglePhasePropList
/*!
  Get a list of all single phase properties that can be calculated by this property package
  \param props [out, retval] Receives the list of supported single-phase properties
*/

STDMETHODIMP CPropertyPackage::GetSinglePhasePropList(VARIANT * props)
{	if (!props) return E_POINTER;
    //production implementations should cache and copy this list
    int i;
    CVariant list;
    list.MakeArray(SinglePhasePropertyCount,VT_BSTR);
    for (i=0;i<SinglePhasePropertyCount;i++) list.AllocStringAt(i,SinglePhasePropertyNames[i]);
    *props=list.ReturnValue();
	return NOERROR;
}

//! ICapeThermoPropertyRoutine::GetTwoPhasePropList
/*!
  Get a list of all two-phase properties that can be calculated by this property package
  \param props [out, retval] Receives the list of supported two-phase properties
*/

STDMETHODIMP CPropertyPackage::GetTwoPhasePropList(VARIANT * props)
{	if (!props) return E_POINTER;
    //production implementations should cache and copy this list
    int i;
    CVariant list;
    list.MakeArray(TwoPhasePropertyCount,VT_BSTR);
    for (i=0;i<TwoPhasePropertyCount;i++) list.AllocStringAt(i,TwoPhasePropertyNames[i]);
    *props=list.ReturnValue();
	return NOERROR;
}

//! ICapeThermoUniversalConstant::GetUniversalConstant
/*!
  Get value of a universal constant
  \param constantId [in] Constant ID for which to get the value
  \param constantValue [out, retval] Receives the value of the constant 
*/

STDMETHODIMP CPropertyPackage::GetUniversalConstant(BSTR constantId, VARIANT * constantValue)
{	if (!constantValue) return E_POINTER;
    wstring error;
    double value;
    if (CBSTR::Same(constantId,L"avogadroConstant")) value=6.02214199e23;
    else if (CBSTR::Same(constantId,L"boltzmannConstant")) value=1.3806503e-23;
    else if (CBSTR::Same(constantId,L"molarGasConstant")) value=8.314472;
    else if (CBSTR::Same(constantId,L"speedOfLightInVacuum")) value=299792458;
    else if (CBSTR::Same(constantId,L"standardAccelerationOfGravity")) value=9.80665;
    else
     {error=L"Unsupported univeral constant: ";
      error+=(constantId)?constantId:L"<NULL>";
      SetError(error.c_str(),L"ICapeThermoUniversalConstant",L"GetUniversalConstant");
      return ECapeInvalidArgumentHR;
     }
    constantValue->vt=VT_R8;
    constantValue->dblVal=value;
    return NOERROR;
}

//! ICapeThermoUniversalConstant::GetUniversalConstant
/*!
  Get the list of all supported universal constants in this property package
  \param constantIds [out, retval] Receives the list of supported universal constants
*/

STDMETHODIMP CPropertyPackage::GetUniversalConstantList(VARIANT * constantIds)
{	if (!constantIds) return E_POINTER;
    CVariant list;
    list.MakeArray(5,VT_BSTR);
    list.AllocStringAt(0,L"avogadroConstant");
    list.AllocStringAt(1,L"boltzmannConstant");
    list.AllocStringAt(2,L"molarGasConstant");
    list.AllocStringAt(3,L"speedOfLightInVacuum");
    list.AllocStringAt(4,L"standardAccelerationOfGravity");
	*constantIds=list.ReturnValue();
	return NOERROR;
}

//! ICapeUtilities::get_parameters
/*!
  Returns an ICapeCollection of parameters. This object has no parameters.
  \param parameters  [out,retval] Will receive the parameter collection
*/

STDMETHODIMP CPropertyPackage::get_parameters(LPDISPATCH * parameters)
{	if (!parameters) return E_POINTER;
	 SetError(L"No parameters are exposed by this object",L"ICapeUtilities",L"get_parameters");
	 return ECapeNoImplHR;
}

//! ICapeUtilities::put_simulationContext
/*!
  Provides a Simulation Context object, which can be used to access PME facilities such as message logging. We don't use it.
  \param context [in] Simulation context object
*/

STDMETHODIMP CPropertyPackage::put_simulationContext(LPDISPATCH context)
{	return NOERROR;
}

//! ICapeUtilities::Initialize
/*!
  Called upon object initialization (after InitNew or Load if we would implement it). This object does not depend
  on PMEs calling it on the proper time. Instead, we use the InitPP macro before each function that requires it.  
*/

STDMETHODIMP CPropertyPackage::Initialize()
{	return NOERROR;
}

//! ICapeUtilities::Terminate
/*!
  Called to clean up. 
*/

STDMETHODIMP CPropertyPackage::Terminate()
{	INITPP(L"Terminate",L"ICapeUtilities");
	terminated=true;
	UnsetMaterial(); //we must drop all references to external objects (if we would keep track of the simulation context, now is the time to release it)
    if (pack) {delete pack;pack=NULL;}
    if (ppfilename.size())
     {//delete the temporary file
      DeleteFileA(ppfilename.c_str());
      ppfilename.clear();
     }
 	return NOERROR;
}

//! ICapeUtilities::Edit
/*!
  Called to edit the object. Support for Edit is optional. If Edit is supported on an object, generally persistence must 
  also be implemented to save the changes made during editing. This object allows for editing and persistence.
  and persistence  
*/

STDMETHODIMP CPropertyPackage::Edit()
{	INITPP(L"Edit",L"ICapeUtilities");
    pack->Edit();
	return NOERROR;
}

//! IPersistStream::IsDirty
/*!
Check whether we need to be saved
\return S_OK if we need to be saved, or S_FALSE
*/

STDMETHODIMP CPropertyPackage::IsDirty()
{//we should return true if we need to be saved.
 // to make sure, we always return true
 return S_OK;
}

//! IPersistStream::Load
/*!
Restore from persistence

It is highly recommended to implement persistence on Property Package objects; this facilitates portability of 
documents containing the Property Package to other systems (provided the PME supports persistence on Property
Packages). In case Edit is supported on Property Packages, persistence is a must (or the changes made in 
Edit would go lost).

\param pstm [in] IStream to load from
\return S_OK for success, or S_FALSE
\sa Save()
*/

STDMETHODIMP CPropertyPackage::Load(IStream * pstm)
{   if (!pstm) return E_POINTER;
	if (ppfilename.size())
	 {SetError(L"Property package can only be loaded once",L"IPersistStream",L"Load");
	  operation=L"N/A";
	  return ECapeBadInvOrderHR;
	 }
	ppfilename=GetTempFileName();
	HANDLE hFile=CreateFileA(ppfilename.c_str(),GENERIC_WRITE,0,NULL,CREATE_ALWAYS,FILE_ATTRIBUTE_NORMAL,NULL);
	if (hFile==INVALID_HANDLE_VALUE)
	 {DeleteFileA(ppfilename.c_str());
	  ppfilename.clear();
	  SetError(L"Failed to create temp file",L"IPersistStream",L"Load");
	  return ECapePersistenceSystemErrorHR;
	 }
	ULONG read;
	DWORD written;
	int size;
	char *buf=NULL;
	if (FAILED(pstm->Read(&size,sizeof(int),&read))) 
	 {fail:
	  if (buf) delete []buf;
	  CloseHandle(hFile);
	  DeleteFileA(ppfilename.c_str());
	  ppfilename.clear();
	  return E_FAIL; 
	 }
	if (read!=sizeof(UINT)) goto fail;
	buf=new char[size];
	if (!buf)
	 {CloseHandle(hFile);
	  DeleteFileA(ppfilename.c_str());
	  ppfilename.clear();
	  return E_OUTOFMEMORY; 
	 }
	if (FAILED(pstm->Read(buf,size,&read))) goto fail;
	if (read!=size) goto fail;
	if (!WriteFile(hFile,buf,size,&written,NULL)) goto fail;
	if (written!=size) goto fail;
	CloseHandle(hFile);
	return S_OK;
}

//! IPersistStream::Save
/*!
Save to persistence

It is highly recommended to implement persistence on Property Package objects; this facilitates portability of 
documents containing the Property Package to other systems (provided the PME supports persistence on Property
Packages). In case Edit is supported on Property Packages, persistence is a must (or the changes made in 
Edit would go lost).

\param pstm [in] IStream to save to
\param fClearDirty [in] if set, we must clear the dirty flags
\return S_OK for success, or S_FALSE
\sa Load(), GetSizeMax()
*/

STDMETHODIMP CPropertyPackage::Save(IStream * pstm, BOOL fClearDirty)
{   if (!pstm) return E_POINTER;
	INITPP(L"Save",L"IPersistStream");
	ULONG written;
	DWORD read;
	//make temp file name
	string fname;
	fname=GetTempFileName();
	//save pp to temp file name
	if (!pack->Save(fname.c_str()))
	 {DeleteFileA(fname.c_str());
	  string err=pack->LastError();
	  err="Failed to save property package: "+err;
	  SetError(CA2CT(err.c_str()),L"IPersistStream",L"Save");
	  return ECapePersistenceSystemErrorHR;
	 }
	//open the file
	HANDLE hFile=CreateFileA(fname.c_str(),GENERIC_READ,0,NULL,OPEN_EXISTING,FILE_ATTRIBUTE_NORMAL,NULL);
	if (hFile==INVALID_HANDLE_VALUE)
	 {DeleteFileA(fname.c_str());
	  SetError(L"Failed to re-open saved property package info",L"IPersistStream",L"Save");
	  return ECapePersistenceSystemErrorHR;
	 }
	int size=GetFileSize(hFile,NULL);
	char *buf=new char[size];
	if (!buf)
	 {CloseHandle(hFile);
	  DeleteFileA(fname.c_str());
	  return E_OUTOFMEMORY;
	 }
	//store size
	if (FAILED(pstm->Write(&size,sizeof(int),&written))) 
	 {fail:
	  CloseHandle(hFile);
	  DeleteFileA(fname.c_str());
	  delete []buf;
	  return E_FAIL; 
	 }
	if (written!=sizeof(int)) goto fail;
	//store content
	buf=new char[size];
	if (!ReadFile(hFile,buf,size,&read,NULL)) goto fail;
	if (read!=size) goto fail;
	if (FAILED(pstm->Write(buf,size,&written))) goto fail;
	if (written!=size) goto fail;
	//clean up
	delete []buf;
	CloseHandle(hFile);
	DeleteFileA(fname.c_str());
	return S_OK;
}

//! IPersistStream::GetSizeMax
/*!
Return the maximum size required to save this object
\param pcbSize [out] receives the size, cannot be NULL
\sa Save()
*/

STDMETHODIMP CPropertyPackage::GetSizeMax(_ULARGE_INTEGER * pcbSize)
{   if (!pcbSize) return E_POINTER;
    INITPP(L"GetSizeMax",L"IPersistStream");
	//make temp file name
	string fname;
	fname=GetTempFileName();
	//save pp to temp file name
	if (!pack->Save(fname.c_str()))
	 {DeleteFileA(fname.c_str());
	  string err=pack->LastError();
	  err="Failed to save property package: "+err;
	  SetError(CA2CT(err.c_str()),L"IPersistStream",L"GetSizeMax");
	  return ECapePersistenceSystemErrorHR;
	 }
	//open the file
	HANDLE hFile=CreateFileA(fname.c_str(),GENERIC_READ,0,NULL,OPEN_EXISTING,FILE_ATTRIBUTE_NORMAL,NULL);
	if (hFile==INVALID_HANDLE_VALUE)
	 {DeleteFileA(fname.c_str());
	  SetError(L"Failed to re-open saved property package info",L"IPersistStream",L"GetSizeMax");
	  return ECapePersistenceSystemErrorHR;
	 }
	int size=GetFileSize(hFile,NULL);
	CloseHandle(hFile);
	DeleteFileA(fname.c_str());
	pcbSize->QuadPart=size;
	return NOERROR;
}

//! IPersistStream::GetClassID
/*!
Return the CLSID of this object
\param pClassID [out] receives the CLSID, cannot be NULL
*/

STDMETHODIMP CPropertyPackage::GetClassID(CLSID *pClassID)
{   if (!pClassID) return E_POINTER;
	*pClassID=CLSID_PropertyPackage;
	return NOERROR;
}

//! GetTempFileName
/*!
  Returns a temporary file name for property package content storage
  \return File name
*/

string CPropertyPackage::GetTempFileName()
{//no error check, we presume these functions succeed
 char *buf=new char[MAX_PATH];
 char *buf1=new char[MAX_PATH];
 GetTempPathA(MAX_PATH,buf);
 GetTempFileNameA(buf,"PP",0,buf1);
 delete []buf;
 string res=buf1;
 delete []buf1;
 return res;
}

//! GetCompounds
/*!
  Get the list of compounds from a VARIANT containing the compound IDs
  
  This implementation is meant to be demonstrative; for actual 
  implementations in production software, a cheaper solution should 
  be aimed for (e.g. compare component list to the one in the last call, 
  use a hash table for looking up the components, store the list 
  of compounds from the underlying property package, avoid case 
  insensitive string comparisons by using lower or upper case
  versions of the compound IDs....)

  This function does not do a check on a proper list content (e.g. 
  no check for components that appear more than once); the underlying 
  thermodynamic routines in IdealThermoModule will check the component
  list as it is passed to property or equilibrium calculations.

  \param compoundList VARIANT containing an array of compound IDs
  \param compoundIndices Receives the list of compound indices
  \param allowEmptyList If TRUE, an empty compoundList means all compounds in the property package
  \param error Receives an error message in case of failure
  \return TRUE if ok, FALSE in case of failure
*/

BOOL CPropertyPackage::GetCompounds(VARIANT compoundList,vector<int> &compoundIndices,BOOL allowEmptyList,wstring &error)
{int i,j,count;
 //check the return value
 CVariant comps(compoundList,FALSE); //we do not own this list
 if (!comps.CheckArray(VT_BSTR,error))
  {error=L"Invalid list of compounds: "+error;
   return FALSE;
  }
 //get component count in property package
 if (!pack->GetCompoundCount(&count))
  {error=L"Failed to get component count in property package: ";
   error+=CA2CT(pack->LastError());
   return FALSE;
  } 
 //check empty  
 if (comps.GetCount()==0)
  {if (allowEmptyList)
    {//this means all compounds are in the list
     compoundIndices.resize(count);
     for (i=0;i<count;i++) compoundIndices[i]=i;
     return TRUE;
    }
   //empty list not ok
   error=L"List of compounds is empty";
   return FALSE;
  }
 //re-alloc compIndices
 compoundIndices.resize(comps.GetCount());
 //loop over components
 for (i=0;i<comps.GetCount();i++)
  {CBSTR compID=comps.GetStringAt(i);
   if (!compID) 
    {error=L"Invalid list of compounds: contains at least one empty string";
     return FALSE;
    }
   string compId=CT2CA(compID);
   //compare against the component names in the property package (ignore character case)
   for (j=0;j<count;j++)
    {const char *compName=pack->GetCompoundStringConstant(j,Name);
     if (!compName)
      {error=L"Failed to get component name in property package: ";
       error+=CA2CT(pack->LastError());
       return FALSE;
      }
     if (lstrcmpiA(compName,compId.c_str())==0)
      {//found it
       compoundIndices[i]=j;
       break;
      }
    }
   if (j==count)
    {//compound not found
     error=L"Invalid list of compounds: compound \"";
     error+=CA2CT(compId.c_str());
     error+=L"\" does not exist";
     return FALSE;
    }
  }
 //all OK
 return TRUE;
}

//! GetFlashSpec
/*!
  Check a flash specification and return what flash should be calculated
  
  The flash specifications contain 3 or 4 strings: property, basis, phase and compound
  
  A compound specification is optional, and would be present only for flash specifications
  of vector properties such as fugacity, activity, fraction.
  
  A basis is only required for properties to which a basis conversion applies and composition 
  is not available. So for overall properties this is not required, as the overall composition 
  is known and the material object will provide basis conversions. Property and phase are
  always required; overall is a valid phase specification in this context (and the most
  commonly used).
  
  A solution type only applies to flashes in which a phase fraction is specified, in which 
  case a "Normal" or "Retrograde" flash result can be requested. Otherwise it should be 
  "Unspecified"; we will treat no specification (Null) equal to "Unspecified", although 
  "Unspecified" would be the proper argument in this case. Furthermore, we do not support
  retrograde flashes.
  
  \param specification1 specification of first flash constraint
  \param specification2 specification of second flash constraint
  \param solutionType specification of requested solution for flash containing vapor fraction specification
  \param type Receives the flash type
  \param phaseType Receives the allowed phases specification for the flash
  \param error Receives an error message in case of failure
  \return TRUE if ok, FALSE in case of failure
*/

BOOL CPropertyPackage::GetFlashSpec(VARIANT specification1, VARIANT specification2, BSTR solutionType,FlashType &type,FlashPhaseType &phaseType,wstring &error)
{bool haveT,haveP,haveVFm,haveVF,haveH,haveS;
 haveT=haveP=haveVFm=haveVF=haveH=haveS=false;
 for (int flashSpec=0;flashSpec<2;flashSpec++)
  {CVariant spec;
   spec.Set((flashSpec)?specification2:specification1,FALSE); //we do not own this value
   if (!spec.CheckArray(VT_BSTR,error))
    {wstring s;
     s=L"Invalid flash specification ";
     s+=(L'1'+flashSpec);
     s+=L": ";
     s+=error;
     error=s;
     return FALSE;
    }
   CBSTR prop=spec.GetStringAt(0);
   CBSTR phase=spec.GetStringAt(2);
   CBSTR basis=spec.GetStringAt(1);
   if ((spec.GetCount()!=3)&&(spec.GetCount()!=4))
    {error=L"Invalid flash specification ";
     error+=(L'1'+flashSpec);
     error+=L": expected array of 3 or 4 elements";
     return FALSE;
    }
   if (spec.GetCount()==4)
    {//check that compound is unspecified
     CBSTR comp=spec.GetStringAt(3);
     if (comp.Length()) goto unsupported;
    }
   if (prop.Same(temperature))
    {if (basis.Length()) 
      {error=L"Invalid flash specification ";
       error+=(L'1'+flashSpec);
       error+=L": no basis expected for temperature";
       return FALSE;
      }
     if (!phase.Same(L"overall"))
      {error=L"Invalid flash specification ";
       error+=(L'1'+flashSpec);
       error+=L": expected overall phase for temperature";
       return FALSE;
      }
     if (haveT)
      {error=L"Temperature is specified more than once";
       return FALSE;
      }
     haveT=true;
    }   
   else if (prop.Same(pressure))
    {if (basis.Length()) 
      {error=L"Invalid flash specification ";
       error+=(L'1'+flashSpec);
       error+=L": no basis expected for pressure";
       return FALSE;
      }
     if (!phase.Same(L"overall"))
      {error=L"Invalid flash specification ";
       error+=(L'1'+flashSpec);
       error+=L": expected overall phase for pressure";
       return FALSE;
      }
     if (haveP)
      {error=L"Pressure is specified more than once";
       return FALSE;
      }
     haveP=true;
    }   
   else if (prop.Same(phaseFraction))
    {bool isMass;
     if (basis.Same(L"mole")) isMass=false;
     else if (basis.Same(L"mass")) isMass=true;
     else
      {error=L"Invalid flash specification ";
       error+=(L'1'+flashSpec);
       error+=L": expected mole or mass basis for phase fraction";
       return FALSE;
      }
     if (!phase.Same(gas))
      {error=L"Invalid flash specification ";
       error+=(L'1'+flashSpec);
       error+=L": only the gas phase is supported for phase fraction flashes";
       return FALSE;
      }
     if ((haveVF)||(haveVFm))
      {error=L"Vapor fraction is specified more than once";
       return FALSE;
      }
     if (isMass) haveVFm=true; else haveVF=true;
    }   
   else if (prop.Same(enthalpy))
    {//basis should be NULL, but we will accept mole and mass as well
     if (basis.Length())
      if ((!basis.Same(L"mole"))&&(!basis.Same(L"mass")))
       {error=L"Invalid flash specification ";
        error+=(L'1'+flashSpec);
        error+=L": invalid basis for enthalpy";
        return FALSE;
       }
     if (!phase.Same(L"overall"))
      {error=L"Invalid flash specification ";
       error+=(L'1'+flashSpec);
       error+=L": only overall is supported for enthalpy";
       return FALSE;
      }
     if (haveH)
      {error=L"Enthalpy is specified more than once";
       return FALSE;
      }
     haveH=true;
    }   
   else if (prop.Same(entropy))
    {if (basis.Length())
      if ((!basis.Same(L"mole"))&&(!basis.Same(L"mass")))
       {error=L"Invalid flash specification ";
        error+=(L'1'+flashSpec);
        error+=L": invalid basis for entropy";
        return FALSE;
       }
     if (!phase.Same(L"overall"))
      {error=L"Invalid flash specification ";
       error+=(L'1'+flashSpec);
       error+=L": only overall is supported for entropy";
       return FALSE;
      }
     if (haveS)
      {error=L"Entropy is specified more than once";
       return FALSE;
      }
     haveS=true;
    }   
   else 
    {unsupported:
     error=L"Invalid or unsupported flash specification ";
     error+=(L'1'+flashSpec);
     return FALSE;
    }
  }
 //now we have two out of {T, P, VF, VFm, H, S}; let's see if the combination is supported
 if (haveT)
  {if (haveP) type=TP;
   else if (haveVF) type=TVF;
   else if (haveVFm) type=TVFm;
   else 
    {unsupportedCombination:
     error=L"Unsupported combination of flash specifications";
     return FALSE;
    }
  }
 else if (haveP)
  {if (haveVF) type=PVF;
   else if (haveVFm) type=PVFm;
   else if (haveH) type=PH;
   else if (haveS) type=PS;
   else goto unsupportedCombination;
  }
 else goto unsupportedCombination;
 //we have a supported combination, check solution type
 if (solutionType)
  if (!CBSTR::Same(solutionType,L"unspecified"))
   {//could be normal or retrograde
    if (CBSTR::Same(solutionType,L"normal"))
     {//ok in case of VF flashes
      if ((!haveVF)&&(!haveVFm))
       {error=L"The normal flash solution type is only allowed in case of a phase fraction specification";
        return FALSE;
       }
     }
    else if (CBSTR::Same(solutionType,L"retrograde"))
     {//valid in case of VF flashes
      if ((!haveVF)&&(!haveVFm))
       {error=L"The retrograde flash solution type is only allowed in case of a phase fraction specification";
        return FALSE;
       }
      // ... but we do not support it
      error=L"Retrograde flashes are not supported";
      return FALSE;
     }
    else
     {//invalid
      error=L"Invalid solution type specification";
      return FALSE;
     }
   }
 //check which phases are present on the context MO
 if (!contextMaterial)
  {error=L"Context material is not set";
   return FALSE;
  }
 VARIANT phaseLabels,phaseStatus;
 phaseLabels.vt=VT_EMPTY;
 phaseStatus.vt=VT_EMPTY;
 HRESULT hr=contextMaterial->GetPresentPhases(&phaseLabels,&phaseStatus);
 if (FAILED(hr))
  {error=L"Failed to get list of present phases from context material: ";
   error+=CO_Error(contextMaterial,hr);
   return FALSE;
  }
 VariantClear(&phaseStatus); //we do not care about this information; it might contain CAPE_ESTIMATES, meaning it could serve as an initial guess for equilibrium calculations
 CVariant phases(phaseLabels,TRUE); //do not delete phaseLabels after this
 if (!phases.CheckArray(VT_BSTR,error))
  {error=L"Invalid list of phases from context material: "+error;
   return FALSE;
  }
 int phasePresence=0;
 for (int i=0;i<phases.GetCount();i++)
  {CBSTR phaseName=phases.GetStringAt(i);
   if (phaseName.Same(gas)) phasePresence|=VaporOnly; //we use this as a bit field now
   else if (phaseName.Same(liquid)) phasePresence|=LiquidOnly; // ... so that both phases leads to 3 = VaporLiquid
   else
    {error=L"Invalid phase in present phase list from material object: ";
     error+=(phaseName)?phaseName:L"<NULL>";
     return FALSE;
    }
  }
 if (phasePresence==0)
  {//no valid phases present
   error=L"No phases are present on the context material";
   return FALSE;
  }    
 phaseType=(FlashPhaseType)phasePresence;
 //all ok
 return TRUE;
}