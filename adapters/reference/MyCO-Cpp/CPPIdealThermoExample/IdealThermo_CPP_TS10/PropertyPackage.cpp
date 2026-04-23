// PropertyPackage.cpp : Implementation of CPropertyPackage

#include "stdafx.h"
#include "PropertyPackage.h"
#include "Variant.h"
#include "Helpers.h"
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

//! ICapePropertyPackage::GetPhaseList
/*!
  Returns a list of all supported phase identifiers. Vapor phases must start with "Vapor",
  liquid phases must start with "Liquid" and solid phases must start with "Solid".
  \param phases [out,retval] Will receive the list of phases
*/

STDMETHODIMP CPropertyPackage::GetPhaseList(VARIANT * phases)
{	if (!phases) return E_POINTER;
	INITPP(L"GetPhaseList",L"ICapeThermoPropertyPackage");
	//we support Vapor and Liquid, but we support property calculations for Overall and VaporLiquid as well
	CVariant res;
	res.MakeArray(4,VT_BSTR);
	res.AllocStringAt(0,L"Vapor");
	res.AllocStringAt(1,L"Liquid");
	res.AllocStringAt(2,L"Overall");
	res.AllocStringAt(3,L"VaporLiquid");
	*phases=res.ReturnValue();
	return NOERROR;
}

//! ICapePropertyPackage::GetComponentList
/*!
  Returns a list of all supported components (compounds) and some of their properties. 
  Note that all arguments are [in, out] so we must clear them before we use them. For
  any data that we do not have for a compound, we can return UNDEFINED.
  \param compIds [in, out] Will receive the list of compound IDs
  \param formulae [in, out] Will receive the list of chemical formulae
  \param names [in, out] Will receive the list of compound names (we use the same as compound IDs)
  \param boilTemps [in, out] Will receive the list of normal boiling point temperatures [K]
  \param molwt [in, out] Will receive the list of relative molecular weights
  \param casno [in, out] Will receive the list of CAS registry numbers
*/

STDMETHODIMP CPropertyPackage::GetComponentList(VARIANT * compIds, VARIANT * formulae, VARIANT * names, VARIANT * boilTemps, VARIANT * molwt, VARIANT * casno)
{	if ((!compIds)||(!formulae)||(!names)||(!boilTemps)||(!molwt)||(!casno)) return E_POINTER;
    INITPP(L"GetComponentList",L"ICapeThermoPropertyPackage");
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
     {SetError(CA2CT(pack->LastError()),L"ICapeThermoPropertyPackage",L"GetComponentList");
      return ECapeUnknownHR;
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
        SetError(CA2CT(pack->LastError()),L"ICapeThermoPropertyPackage",L"GetComponentList");
        return ECapeUnknownHR;
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


//! ICapePropertyPackage::GetUniversalConstant
/*!
  Returns values of universals constants
  \param materialObject [in] Material object reference, unused
  \param props [in] Requested value identifiers
  \param propVals [out, retval] Will receive the values, as array of Variants
*/

STDMETHODIMP CPropertyPackage::GetUniversalConstant(LPDISPATCH materialObject, VARIANT props, VARIANT * propVals)
{	if (!propVals) return E_POINTER;
    //check input
    CVariant properties(props,FALSE);
    wstring error;
    if (!properties.CheckArray(VT_BSTR,error))
     {error=L"Invalid properties array: "+error;
      SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"GetUniversalConstant");
      return ECapeInvalidArgumentHR;
     }
    //create array of real results
    CVariant res;
    res.MakeArray(properties.GetCount(),VT_VARIANT);
    for (int i=0;i<properties.GetCount();i++)
     {CBSTR b=properties.GetStringAt(i);
      if (!b)
       {SetError(L"Invalid properties array: contains at least one empty element",L"ICapeThermoPropertyPackage",L"GetUniversalConstant");
        return ECapeInvalidArgumentHR;
       }
      VARIANT doubleVal;
      doubleVal.vt=VT_R8;
      if (CBSTR::Same(b,L"avogadroConstant"))
       {doubleVal.dblVal=6.02214199e23;
       }
      else if (CBSTR::Same(b,L"boltzmannConstant"))
       {doubleVal.dblVal=1.3806503e-23;
       }
      else if (CBSTR::Same(b,L"molarGasConstant"))
       {doubleVal.dblVal=8.314472;
       }
      else if (CBSTR::Same(b,L"speedOfLightInVacuum"))
       {doubleVal.dblVal=299792458;
       }
      else if (CBSTR::Same(b,L"standardAccelerationOfGravity"))
       {doubleVal.dblVal=9.80665;
       }
      else
       {error=L"Unsupported univeral constant: ";
        error+=b;
        SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"GetUniversalConstant");
        return ECapeInvalidArgumentHR;
       }
      res.SetVariantAt(i,doubleVal);
     }
    //return result
    *propVals=res.ReturnValue();
    return NOERROR;
}

//! ICapePropertyPackage::GetComponentConstant
/*!
  Returns constant values for compounds on the Material Object
  \param materialObject [in] Material object reference, used to pass the list of compounds
  \param props [in] Requested constant property identifiers
  \param propVals [out, retval] Will receive the values, as array of Variants, for all compounds and all requested properties
*/

STDMETHODIMP CPropertyPackage::GetComponentConstant(LPDISPATCH materialObject, VARIANT props, VARIANT * propVals)
{	if ((!materialObject)||(!propVals)) return E_POINTER;
	INITPP(L"GetComponentConstant",L"ICapeThermoPropertyPackage");
	//get the list of components on the material object
	ICapeThermoMaterialObjectPtr MO(materialObject); //smart pointer, no need to release
	if (!MO)
	 {SetError(L"Failed to get ICapeThermoMaterialObject from material object",L"ICapeThermoPropertyPackage",L"GetComponentConstant");
      return ECapeInvalidArgumentHR;	 
	 }
	if (!GetCompoundsFromMaterial(MO,L"GetComponentConstant",L"ICapeThermoPropertyPackage")) {return ECapeUnknownHR;} //error has been set
	//check the list of properties
	CVariant propList(props,FALSE);
	wstring error;
	if (!propList.CheckArray(VT_BSTR,error))
	 {error=L"Invalid list of properties: "+error;
	  SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"GetComponentConstant");
      return ECapeUnknownHR;	 
	 }
	//make the result list:  
	CVariant results;
	results.MakeArray((int)compIndices.size()*propList.GetCount(),VT_VARIANT);
	int index=0; //index into the result array
	CVariant v; //store the result here for a property for a compound 
	//loop over the properties
	// (production implementations should interpret all property names once)
	for (int prop=0;prop<propList.GetCount();prop++)
	 {CBSTR propName=propList.GetStringAt(prop);
	  for (int comp=0;comp<(int)compIndices.size();comp++)
	   {int compIndex=compIndices[comp];
	    double realVal;
	    const char *stringVal;
	    if (!propName)
	     {SetError(L"Invalid list of properties: contains at least one empty string",L"ICapeThermoPropertyPackage",L"GetComponentConstant");
          return ECapeInvalidArgumentHR;	 
	     }
	    if (propName.Same(L"molecularWeight"))
	     {if (!pack->GetCompoundRealConstant(compIndex,MolecularWeight,realVal))
	       {failedGetValue:
	        error=L"Failed to get \"";
	        error+=propName;
	        error+=L"\": ";
	        error+=CA2CT(pack->LastError());
	        SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"GetComponentConstant");
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
	      SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"GetComponentConstant");
	      return ECapeInvalidArgumentHR;	 	     
	     }
	    results.SetVariantAt(index,v.Value()); 
	    index++;
	   }
	 }
	*propVals=results.ReturnValue();
	return NOERROR;
}

//! ICapePropertyPackage::CalcProp
/*!
  Calculate properties for phases.
  \param materialObject [in] Material object reference, used for calculation conditions and to store results
  \param props [in] Requested properties
  \param phases [in] Phases to calculate the properties for
  \param calcType [in] "Mixture" or "Pure", depending on the type of requested calculation
*/

STDMETHODIMP CPropertyPackage::CalcProp(LPDISPATCH materialObject, VARIANT props, VARIANT phases, BSTR calcType)
{	if (!materialObject) return E_POINTER;
	INITPP(L"CalcProp",L"ICapeThermoPropertyPackage");
	//The version 1.0 CAPE-OPEN CalcProp is rather generic. It allows for calculation of temperature
	// dependent properties, single-phase mixture properties, two-phase properties, overall properties
	// all at once. Hence, we have quite some checks to perform here. This is better arranged in version 1.1
	//We do allow for overall property calculations, but only for volume, enthalpy and entropy, and 
	// none of their derivatives. For all mixture properties, we only allow for Mixture calculation type,
	// whereas for all temperature dependent properties, we only allow for Pure calculation type. For
	// temperature dependent properties, the phase does not matter, except for that we use the same phase
	// for storing the results at the material object. For two-phase properties, the only allowed phase
	// is VaporLiquid.
	//We must also not set any properties on the Material Object until all property calculations have
	// succeeded. This is a demand that is not actually enforced by many implementations, but we will
	// do so in this implementation. To do so, we buffer all results and only set them at the point all
	// calculations are completed (buffering is at the expense of allocating structures to do so)
	//Also, as we need to do some clean-up, we postpone cleaning up before a single return point, that 
	// we jump to using a label
	//This routine is intended for demonstrative purposes. Pruction implementations should aim for a 
	// more efficient routine (e.g. pre-check the list of properties before looping over the phases, 
	// use hash tables for property lookups, avoid caching of results and corresponding memory allocations, 
	// ...)
	typedef struct //structure to cache the calculation results, which are set on the material object once all calculations have succeeded
	{ CBSTR propName;  //name of the property
	  CBSTR phase;     //phase 
	  BSTR basis;      //basis, should not be freed (refers to global string)	  
	  vector<double> values; //values to set on the material object
	} CachedPropertyResult;
	HRESULT hr;        //result code for COM calls
	HRESULT resultHR;  //return code
	CVariant propList(props,FALSE); //list of properties (props will not be freed)
	CVariant phaseList(phases,FALSE); //list of phases(phases will not be freed)
	vector<CachedPropertyResult*> cachedResults; //vector of cached calculation results
	wstring error;     //in case of failure, set error, jump to cleanup (to delete cachedResults)
	BOOL isPure;       // pure if true, mixture otherwise
	CVariant V;        // temp variable for getting calculation conditions
	int i,j;           // loop counters
	int iProp,iPhase,propID; //loop counters
	double T,P;        // temperature and pressure
	vector<double> X,X2;  // composition 
	int *calculatedValueCount; //pointer to counts of values calculated by underlying property package
    double **calculatedValues; //pointer to arrays of values calculated by underlying property package
	vector<SinglePhaseProperty> propSinglePhase; //single phase property IDs to be calculated
	vector<TwoPhaseProperty> propTwoPhase; //two-phase property IDs to be calculated
	CBSTR propName,phaseName; //name of the current property, current phase, within loop
	resultHR=NOERROR;
	//get MO
	ICapeThermoMaterialObjectPtr mat(materialObject); //smart pointer, no need to release
	if (!mat)
	 {error=L"Failed to get ICapeThermoMaterialObject from material object";
	  resultHR=ECapeInvalidArgumentHR;
	  goto cleanup;
	 }
	//get compounds on the MO
	if (!GetCompoundsFromMaterial(mat,L"CalcProp",L"ICapeThermoPropertyPackage"))
	 {error=errDesc; //was set by the above routine
	  resultHR=ECapeUnknownHR;
	  goto cleanup;
	 }
	//check calcType
	if (CBSTR::Same(calcType,L"Pure")) isPure=TRUE;
	else if (CBSTR::Same(calcType,L"Mixture")) isPure=FALSE;
	else
	 {error=L"Invalid CalcType, must be Pure or Mixture";
	  resultHR=ECapeInvalidArgumentHR;
	  goto cleanup;
	 }
	//check prop list
	if (!propList.CheckArray(VT_BSTR,error))
	 {error=L"Invalid list of properties: "+error;
	  resultHR=ECapeInvalidArgumentHR;
	  goto cleanup;
	 }
	if (propList.GetCount()==0) goto cleanup; //nothing to do, no error
	//check phase list
	if (!phaseList.CheckArray(VT_BSTR,error))
	 {error=L"Invalid list of phases: "+error;
	  resultHR=ECapeInvalidArgumentHR;
	  goto cleanup;
	 }
	if (phaseList.GetCount()==0) goto cleanup; //nothing to do, no error
	if (isPure)
	 {//Pure calculations, we only allow temperature dependent properties and do not care about the phase
	  for (iProp=0;iProp<propList.GetCount();iProp++)
	   {propName=propList.GetStringAt(iProp);
	    for (propID=0;propID<ExposedTDependentPropertyCount;propID++) //production implementations should use a hash table or equivalent
	     if (propName.Same(TDependentPropertyNames[propID])) 
	      break;
	    if (propID==ExposedTDependentPropertyCount)
	     {//not a T-dependent property, hence pure calculation not valid
	      error=L"Property \"";
	      error+=propName;
	      error+=L"\" (calcType Pure) is not supported";
	      resultHR=ECapeInvalidArgumentHR;
	      goto cleanup;
	     }
	    //get temperature, if we did not do so already
	    if (iProp==0)
	     {if (!GetPropertyFromMaterial(mat,temperature,overall,NULL,NULL,1,V,error)) 
	       {resultHR=ECapeUnknownHR;
	        goto cleanup;
	       }
	      T=V.GetDoubleAt(0);
	     }
	    //calc the property values
	    vector<double> vals;
	    vals.resize(compIndices.size());
	    for (i=0;i<(int)compIndices.size();i++)
	     {if (!pack->GetTemperatureDependentProperty(compIndices[i],(TDependentProperty)propID,T,vals[i]))
	       {string s=pack->LastError(); //error description of the GetTemperatureDependentProperty call
	        error=L"Failed to calculate ";
	        error+=propName;
	        error+=L" for compound ";
	        error+=CA2CT(pack->GetCompoundStringConstant(compIndices[i],Name));
	        error+=L": ";
	        error+=CA2CT(s.c_str());
	        resultHR=ECapeComputationHR;
	        goto cleanup;
	       }
	     }
	    //set the property for all requested phases
	    for (iPhase=0;iPhase<phaseList.GetCount();iPhase++)
	     {phaseName=phaseList.GetStringAt(iPhase);
	      //cache for setting
	      CachedPropertyResult *r=new CachedPropertyResult;
	      r->phase=phaseName;
	      r->propName=propName;
	      r->basis=(TDependentPropertyMoleBasis[propID])?mole:NULL;
	      r->values=vals;
	      cachedResults.push_back(r);
	     }
	   }
	 }
	else
	 {//mixture calculations, loop over the phases
	  for (iPhase=0;iPhase<phaseList.GetCount();iPhase++)
	   {phaseName=phaseList.GetStringAt(iPhase);
	    if ((phaseName.Same(L"Vapor"))||(phaseName.Same(L"Liquid")))
	     {//single phase property calculations
	      Phase phaseID=(phaseName.Same(L"Vapor"))?Vapor:Liquid;
	      //get temperature
	      if (!GetPropertyFromMaterial(mat,temperature,overall,NULL,NULL,1,V,error)) 
	       {resultHR=ECapeUnknownHR;
	        goto cleanup;
	       }
	      T=V.GetDoubleAt(0);
	      //get pressure 
	      if (!GetPropertyFromMaterial(mat,pressure,overall,NULL,NULL,1,V,error)) 
	       {resultHR=ECapeUnknownHR;
	        goto cleanup;
	       }
	      P=V.GetDoubleAt(0);
	      //make a list of properties to calculate:
	      propSinglePhase.resize(propList.GetCount());
	      for (i=0;i<propList.GetCount();i++)
	       {propName=propList.GetStringAt(i);
	        for (j=0;j<SinglePhasePropertyCount;j++) //production implementations should use a hash table or equivalent
	         if (propName.Same(SinglePhasePropertyNames[j]))
	          {propSinglePhase[i]=(SinglePhaseProperty)j;
	           break;
	          }
	        if (j==SinglePhasePropertyCount)
	         {error=L"Property \"";
	          error+=propName;
	          error+=L"\" (calcType Mixture, phase \"";
	          error+=phaseName;
	          error+=L"\") is not supported";
	          resultHR=ECapeInvalidArgumentHR;
	          goto cleanup;
	         }
	       }
	      //get composition
	      CVariant composition;
	      if (!GetPropertyFromMaterial(mat,fraction,phaseName,NULL,mole,(int)compIndices.size(),composition,error)) 
	       {resultHR=ECapeUnknownHR;
	        goto cleanup;
	       }
          X.resize((int)compIndices.size());
          for (j=0;j<(int)compIndices.size();j++) X[j]=composition.GetDoubleAt(j);
          //calculate all properties simultaneously
          if (!pack->GetSinglePhaseProperties((int)compIndices.size(),VECPTR(compIndices),phaseID,T,P,VECPTR(X),(int)propSinglePhase.size(),VECPTR(propSinglePhase),calculatedValueCount,calculatedValues))
           {error=L"Property calculations failed: ";
            error+=CA2CT(pack->LastError());
            resultHR=ECapeComputationHR;
            goto cleanup;
           }
	      //store values for setting
	      for (j=0;j<(int)propSinglePhase.size();j++) 
	       {CachedPropertyResult *r=new CachedPropertyResult;
	        r->phase=phaseName;
	        r->propName=SinglePhasePropertyNames[propSinglePhase[j]];
	        r->basis=(SinglePhasePropertyMoleBasis[propSinglePhase[j]])?mole:NULL;
	        r->values.resize(calculatedValueCount[j]);
	        for (i=0;i<calculatedValueCount[j];i++) r->values[i]=calculatedValues[j][i];
	        cachedResults.push_back(r);
	       }
	     }
	    else if (phaseName.Same(L"Overall"))
	     {//overall property calculations
	      // we only support enthalpy, entropy and volume
	      vector<BSTR> propNames; //do not destroy these values, they are references to class variables
	      for (iProp=0;iProp<propList.GetCount();iProp++)
	       {propName=propList.GetStringAt(iProp);
	        if (propName.Same(L"enthalpy")) 
	         {propSinglePhase.push_back(Enthalpy);
	          propNames.push_back(enthalpy);
	         }
	        else if (propName.Same(L"entropy")) 
	         {propSinglePhase.push_back(Entropy);
	          propNames.push_back(entropy);
	         }
	        else if (propName.Same(L"volume")) 
	         {propSinglePhase.push_back(Volume);
	          propNames.push_back(volume);
	         }
	        else
	         {//not supported 
	          error=L"Property \"";
	          error+=propName;
	          error+=L"\" (calcType Mixture) is not supported for the overall phase";
	          resultHR=ECapeInvalidArgumentHR;
	          goto cleanup;
	         }
	       }
	      //get temperature
	      if (!GetPropertyFromMaterial(mat,temperature,overall,NULL,NULL,1,V,error)) 
	       {resultHR=ECapeUnknownHR;
	        goto cleanup;
	       }
	      T=V.GetDoubleAt(0);
	      //get pressure 
	      if (!GetPropertyFromMaterial(mat,pressure,overall,NULL,NULL,1,V,error)) 
	       {resultHR=ECapeUnknownHR;
	        goto cleanup;
	       }
	      P=V.GetDoubleAt(0);
	      //Get the list of present phases on the MO
	      VARIANT v;
	      v.vt=VT_EMPTY;
	      hr=mat->get_PhaseIds(&v);
	      if (FAILED(hr))
	       {error=L"Failed to get list of present pahses from Material Object: ";
	        error+=CO_Error(mat,hr);
	        resultHR=ECapeUnknownHR;
	        goto cleanup;
	       }
	      V.Set(v,TRUE);
	      if (!V.CheckArray(VT_BSTR,error))
	       {error=L"Invalid list of present phases from Material Object: "+error;
	        resultHR=ECapeUnknownHR;
	        goto cleanup;
	       }
	      //loop over the present phases
	      vector<double> propVals;
	      propVals.resize(propSinglePhase.size());
	      for (i=0;i<(int)propSinglePhase.size();i++) propVals[i]=0;
	      for (i=0;i<V.GetCount();i++)
	       {phaseName=V.GetStringAt(i);
	        Phase presentPhase;
	        if (phaseName.Same(L"Vapor")) presentPhase=Vapor;
	        else if (phaseName.Same(L"Liquid")) presentPhase=Liquid;
	        else
	         {error=L"Material reports present phase \"";
	          error+=phaseName;
	          error+=L"\" which is not defined by the Property Package. Cannot perform overall property calculation";
	          resultHR=ECapeUnknownHR;
	          goto cleanup;
	         }
	        //get the phase fraction
	        CVariant phaseFrac;
	        if (!GetPropertyFromMaterial(mat,phaseFraction,phaseName,NULL,mole,1,phaseFrac,error)) 
	         {resultHR=ECapeUnknownHR;
	          goto cleanup;
	         }
	        double phaseF=phaseFrac.GetDoubleAt(0);
	        if (phaseF>0) 
	         {//get composition
	          CVariant composition;
	          if (!GetPropertyFromMaterial(mat,fraction,phaseName,NULL,mole,(int)compIndices.size(),composition,error)) 
	           {resultHR=ECapeUnknownHR;
	            goto cleanup;
	           }
	          X.resize((int)compIndices.size());
	          for (j=0;j<(int)compIndices.size();j++) X[j]=composition.GetDoubleAt(j);
	          if (!pack->GetSinglePhaseProperties((int)compIndices.size(),VECPTR(compIndices),presentPhase,T,P,VECPTR(X),(int)propSinglePhase.size(),VECPTR(propSinglePhase),calculatedValueCount,calculatedValues))
	           {error=L"Property calculations failed: ";
	            error+=CA2CT(pack->LastError());
	            resultHR=ECapeComputationHR;
	            goto cleanup;
	           }
	          //add to overall values
	          for (j=0;j<(int)propSinglePhase.size();j++) propVals[j]+=phaseF*calculatedValues[j][0];
	         }
	       }
	      //store values for setting
	      for (j=0;j<(int)propSinglePhase.size();j++) 
	       {CachedPropertyResult *r=new CachedPropertyResult;
	        r->phase=overall;
	        r->propName=propNames[j];
	        r->basis=mole;
	        r->values.resize(1);
	        r->values[0]=propVals[j];
	        cachedResults.push_back(r);
	       }
	     }
	    else if (phaseName.Same(L"VaporLiquid"))
	     {//two phase property calculations
	      //get temperature
	      if (!GetPropertyFromMaterial(mat,temperature,overall,NULL,NULL,1,V,error)) 
	       {resultHR=ECapeUnknownHR;
	        goto cleanup;
	       }
	      T=V.GetDoubleAt(0);
	      //get pressure 
	      if (!GetPropertyFromMaterial(mat,pressure,overall,NULL,NULL,1,V,error)) 
	       {resultHR=ECapeUnknownHR;
	        goto cleanup;
	       }
	      P=V.GetDoubleAt(0);
	      //make a list of properties to calculate:
	      propTwoPhase.resize(propList.GetCount());
	      for (i=0;i<propList.GetCount();i++)
	       {propName=propList.GetStringAt(i);
	        for (j=0;j<ExposedTwoPhasePropertyCount;j++) //production implementations should use a hash table or equivalent
	         if (propName.Same(TwoPhasePropertyNames[j]))
	          {propTwoPhase[i]=(TwoPhaseProperty)j;
	           break;
	          }
	        if (j==ExposedTwoPhasePropertyCount)
	         {//note that composition derivates of two-phase properties are not properly defined in 
	          // version 1.0 (as composition w.r.t. the composition of both compounds should be 
	          //taken into account)
	          error=L"Property \"";
	          error+=propName;
	          error+=L"\" (calcType Mixture, phase \"VaporLiquid\") is not supported";
	          resultHR=ECapeInvalidArgumentHR;
	          goto cleanup;
	         }
	       }
	      //get vapor composition 
	      CVariant composition;
	      if (!GetPropertyFromMaterial(mat,fraction,vapor,NULL,mole,(int)compIndices.size(),composition,error)) 
	       {resultHR=ECapeUnknownHR;
	        goto cleanup;
	       }
          X.resize((int)compIndices.size());
          for (j=0;j<(int)compIndices.size();j++) X[j]=composition.GetDoubleAt(j);
	      //get liquid composition 
	      if (!GetPropertyFromMaterial(mat,fraction,liquid,NULL,mole,(int)compIndices.size(),composition,error)) 
	       {resultHR=ECapeUnknownHR;
	        goto cleanup;
	       }
          X2.resize((int)compIndices.size());
          for (j=0;j<(int)compIndices.size();j++) X2[j]=composition.GetDoubleAt(j);
          //calculate all properties simultaneously (we have only one value for T and P for each phase in version 1.0 thermo)
          if (!pack->GetTwoPhaseProperties((int)compIndices.size(),VECPTR(compIndices),Vapor,Liquid,T,T,P,P,VECPTR(X),VECPTR(X2),(int)propTwoPhase.size(),VECPTR(propTwoPhase),calculatedValueCount,calculatedValues))
           {error=L"Property calculations failed: ";
            error+=CA2CT(pack->LastError());
            resultHR=ECapeComputationHR;
            goto cleanup;
           }
	      //store values for setting
	      for (j=0;j<(int)propTwoPhase.size();j++) 
	       {CachedPropertyResult *r=new CachedPropertyResult;
	        r->phase=phaseName;
	        r->propName=TwoPhasePropertyNames[propTwoPhase[j]];
	        r->basis=NULL;
	        r->values.resize(calculatedValueCount[j]);
	        for (i=0;i<calculatedValueCount[j];i++) r->values[i]=calculatedValues[j][i];
	        cachedResults.push_back(r);
	       }
	     }
	    else
	     {//unsupported phase
	      error=L"Unknown or unsupported phase \"";
	      error+=phaseName;
	      error+=L'"';
	      resultHR=ECapeInvalidArgumentHR;
	      goto cleanup;
	     }
	   }
	 }
	//all calculations succeeded, set the results at the material object
	for (i=0;i<(int)cachedResults.size();i++) 
	 {CachedPropertyResult *res=cachedResults[i];
	  V.MakeArray((int)res->values.size(),VT_R8);
	  for (j=0;j<(int)res->values.size();j++) V.SetDoubleAt(j,res->values[j]);
	  hr=mat->SetProp(res->propName,res->phase,empty,calcType,res->basis,V.Value());
	  if (FAILED(hr))
	   {error=L"Failed to set ";
	    error+=res->propName;
	    error+=L", phase ";
	    error+=res->phase;
	    error+=L", calcType ";
	    error+=calcType;
	    error+=L" on Material Object: ";
	    error+=CO_Error(mat,hr);
	    resultHR=ECapeUnknownHR;
	    goto cleanup;
	   }
	 }
	cleanup:
	if (FAILED(resultHR))
	 {//set the error
	  SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"CalcProp");
	 }
	for (i=0;i<(int)cachedResults.size();i++) delete cachedResults[i];
	return resultHR;
}

//! ICapePropertyPackage::CalcEquilibrium
/*!
  Calculate phase equilibrium
  \param materialObject [in] Material object reference, used for calculation conditions and to store results
  \param flashType [in] Type of equilibrium to calculate, e.g. "TP"
  \param props [in] List of properties to calculate for all resulting phases
*/

STDMETHODIMP CPropertyPackage::CalcEquilibrium(LPDISPATCH materialObject, BSTR flashType, VARIANT props)
{	if (!materialObject) return E_POINTER;
	INITPP(L"CalcEquilibrium",L"ICapeThermoPropertyPackage");
	HRESULT hr;
	wstring error;
	vector<double> X;
	CVariant V,propList;
	vector<SinglePhaseProperty> propertiesToCalculate;
	vector<CBSTR> propNames;
	double specVal1,specVal2;
	bool mustSetT=true;
	bool mustSetP=true;
	FlashType type;
	int i,j,k;
	//get MO
	ICapeThermoMaterialObjectPtr mat(materialObject); //smart pointer, no need to release
	if (!mat)
	 {SetError(L"Failed to get ICapeThermoMaterialObject from material object",L"ICapeThermoPropertyPackage",L"CalcEquilibrium");
	  return ECapeInvalidArgumentHR;
	 }
	//get compounds on the MO
	if (!GetCompoundsFromMaterial(mat,L"CalcEquilibrium",L"ICapeThermoPropertyPackage"))
	 {SetError(errDesc.c_str(),L"ICapeThermoPropertyPackage",L"CalcEquilibrium");
	  return ECapeUnknownHR;
	 }
	//get overall composition
	if (!GetPropertyFromMaterial(mat,fraction,overall,NULL,mole,(int)compIndices.size(),V,error))
	 {SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"CalcEquilibrium");
	  return ECapeUnknownHR;
	 }
    X.resize((int)compIndices.size());
    for (j=0;j<(int)compIndices.size();j++) X[j]=V.GetDoubleAt(j);
    //check the property list
    // it is not recommended for PMCs to pass properties to CalcEquilibrium, but rather use CalcEquilibrium with the 
    // proper arguments after a succesful CalcEquilibrium. However, as a Property Package we better provide
    // support for it. So we are performing mixture type calculations on all resulting phases for each of 
    // the properties passed. This means they need to be single-phase properties.
    propList.Set(props,TRUE);
    if (!propList.CheckArray(VT_BSTR,error))
     {error=L"Invalid list of properties: "+error;
      SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"CalcEquilibrium");
	  return ECapeInvalidArgumentHR;
     }
    //make sure that they are single-phase properties
    // (we need to calculate the properties for all resulting phases... hence, only single-phase properties are ok in this context)
    propertiesToCalculate.resize(propList.GetCount());
    propNames.resize(propList.GetCount());
    for (j=0;j<propList.GetCount();j++)
     {propNames[j]=propList.GetStringAt(j);
      for (i=0;i<SinglePhasePropertyCount;i++)
       if (propNames[j].Same(SinglePhasePropertyNames[i]))
        {propertiesToCalculate[j]=(SinglePhaseProperty)i;
         break;
        }
      if (i==SinglePhasePropertyCount)
       {error=L"Property \"";
        error+=propNames[j];
        error+=L"\" is not supported in CalcEquilibrium; only single-phase properties are supported for the list of properties to calculate";
        SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"CalcEquilibrium");
	    return ECapeInvalidArgumentHR;
       }
     }
	//check flashType and get parameters
	if ((CBSTR::Same(flashType,L"TP"))||(CBSTR::Same(flashType,L"PT")))
	 {//TP flash
	  type=TP;
	  mustSetT=false;
	  mustSetP=false;
	  //get temperature
	  if (!GetPropertyFromMaterial(mat,temperature,overall,NULL,NULL,1,V,error))
	   {SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"CalcEquilibrium");
	    return ECapeUnknownHR;
	   }
      specVal1=V.GetDoubleAt(0);
	  //get pressure
	  if (!GetPropertyFromMaterial(mat,pressure,overall,NULL,NULL,1,V,error))
	   {SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"CalcEquilibrium");
	    return ECapeUnknownHR;
	   }
      specVal2=V.GetDoubleAt(0);
	 }
	else if (CBSTR::Same(flashType,L"TVF"))
	 {//T VF flash
	  type=TVF;
	  mustSetT=false;
	  //get temperature
	  if (!GetPropertyFromMaterial(mat,temperature,overall,NULL,NULL,1,V,error))
	   {SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"CalcEquilibrium");
	    return ECapeUnknownHR;
	   }
      specVal1=V.GetDoubleAt(0);
	  //get molar vapor fraction
	  if (!GetPropertyFromMaterial(mat,phaseFraction,vapor,NULL,mole,1,V,error))
	   {SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"CalcEquilibrium");
	    return ECapeUnknownHR;
	   }
      specVal2=V.GetDoubleAt(0);
	 }
	else if (CBSTR::Same(flashType,L"PVF"))
	 {//P VF flash
	  type=PVF;
	  mustSetP=false;
	  //get pressure
	  if (!GetPropertyFromMaterial(mat,pressure,overall,NULL,NULL,1,V,error))
	   {SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"CalcEquilibrium");
	    return ECapeUnknownHR;
	   }
      specVal1=V.GetDoubleAt(0);
	  //get molar vapor fraction
	  if (!GetPropertyFromMaterial(mat,phaseFraction,vapor,NULL,mole,1,V,error))
	   {SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"CalcEquilibrium");
	    return ECapeUnknownHR;
	   }
      specVal2=V.GetDoubleAt(0);
	 }
	else if (CBSTR::Same(flashType,L"PH"))
	 {//PH flash
	  type=PH;
	  mustSetP=false;
	  //get pressure
	  if (!GetPropertyFromMaterial(mat,pressure,overall,NULL,NULL,1,V,error))
	   {SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"CalcEquilibrium");
	    return ECapeUnknownHR;
	   }
      specVal1=V.GetDoubleAt(0);
	  //get molar enthalpy
	  if (!GetPropertyFromMaterial(mat,enthalpy,overall,mixture,mole,1,V,error))
	   {SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"CalcEquilibrium");
	    return ECapeUnknownHR;
	   }
      specVal2=V.GetDoubleAt(0);
	 }
	else if (CBSTR::Same(flashType,L"PS"))
	 {//PS flash
	  type=PS;
	  mustSetP=false;
	  //get pressure
	  if (!GetPropertyFromMaterial(mat,pressure,overall,NULL,NULL,1,V,error))
	   {SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"CalcEquilibrium");
	    return ECapeUnknownHR;
	   }
      specVal1=V.GetDoubleAt(0);
	  //get molar entropy
	  if (!GetPropertyFromMaterial(mat,entropy,overall,mixture,mole,1,V,error))
	   {SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"CalcEquilibrium");
	    return ECapeUnknownHR;
	   }
      specVal2=V.GetDoubleAt(0);
	 }
	else
	 {//unknown / unsupported flash type
	  error=L"Unknown / unsupported flash type: ";
	  error+=flashType;
      SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"CalcEquilibrium");
	  return ECapeInvalidArgumentHR;
	 }
	//calc the flash
	int phaseCount;
	Phase *phases;
	double *phaseFractions;
	double **phaseCompositions;
	double T,P;
	if (!pack->Flash((int)compIndices.size(),VECPTR(compIndices),VECPTR(X),type,specVal1,specVal2,phaseCount,phases,phaseFractions,phaseCompositions,T,P))
	 {error=L"Flash failure: ";
	  error+=CA2CT(pack->LastError());
      SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"CalcEquilibrium");
	  return ECapeComputationHR;
	 }
	//set results on MO
	// set phase fraction and composition for all present phases
	// set T and P if not part of the specifications
	for (j=0;j<phaseCount;j++)
	 {BSTR phaseName=(phases[j]==Vapor)?vapor:liquid;
	  V.MakeArray((int)compIndices.size(),VT_R8);
	  for (i=0;i<(int)compIndices.size();i++) V.SetDoubleAt(i,phaseCompositions[j][i]);
	  hr=mat->SetProp(fraction,phaseName,empty,NULL,mole,V.Value());
	  if (FAILED(hr))
	   {error=L"Failed to set ";
	    error+=phaseName;
	    error+=L" composition on material object: ";
	    error+=CO_Error(mat,hr);
        SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"CalcEquilibrium");
	    return ECapeUnknownHR;
	   }
	  V.MakeArray(1,VT_R8);
	  V.SetDoubleAt(0,phaseFractions[j]);
	  hr=mat->SetProp(phaseFraction,phaseName,empty,NULL,mole,V.Value());
	  if (FAILED(hr))
	   {error=L"Failed to set ";
	    error+=phaseName;
	    error+=L" fraction on material object: ";
	    error+=CO_Error(mat,hr);
        SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"CalcEquilibrium");
	    return ECapeUnknownHR;
	   }
	 }
	if (mustSetP)
	 {V.SetDoubleAt(0,P); //V is still 1 element long
	  hr=mat->SetProp(pressure,overall,empty,NULL,NULL,V.Value());
	  if (FAILED(hr))
	   {error=L"Failed to set pressure on material object: ";
	    error+=CO_Error(mat,hr);
        SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"CalcEquilibrium");
	    return ECapeUnknownHR;
	   }
	 }
	if (mustSetT)
	 {V.SetDoubleAt(0,T); //V is still 1 element long
	  hr=mat->SetProp(temperature,overall,empty,NULL,NULL,V.Value());
	  if (FAILED(hr))
	   {error=L"Failed to set temperature on material object: ";
	    error+=CO_Error(mat,hr);
        SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"CalcEquilibrium");
	    return ECapeUnknownHR;
	   }
	 }
	if (propertiesToCalculate.size()>0)
	 {//we need to store the compositions for all phases, as the underlying property package does not keep them allocated as soon as we call GetSinglePhaseProperties
	  vector<vector<double>> phaseX;
	  phaseX.resize(phaseCount);
	  for (j=0;j<phaseCount;j++)
	   {phaseX[j].resize(compIndices.size());
	    for (i=0;i<(int)compIndices.size();i++) phaseX[j][i]=phaseCompositions[j][i];
	   }
	  //calculate the properties for the resulting phases
      for (j=0;j<phaseCount;j++)
	   {BSTR phaseName=(phases[j]==Vapor)?vapor:liquid;
	    int *valueCount;
	    double **values;
	    if (!pack->GetSinglePhaseProperties((int)compIndices.size(),VECPTR(compIndices),phases[j],T,P,VECPTR(phaseX[j]),(int)propertiesToCalculate.size(),VECPTR(propertiesToCalculate),valueCount,values))
	     {wstring s;
	      s=L"Property calculation failed for the ";
	      s+=phaseName;
	      s+=L" phase: ";
	      s+=error;
          SetError(s.c_str(),L"ICapeThermoPropertyPackage",L"CalcEquilibrium");
	      return ECapeComputationHR;
	     }
	    for (i=0;i<(int)propertiesToCalculate.size();i++)
	     {CVariant vals;
	      vals.MakeArray(valueCount[i],VT_R8);
	      for (k=0;k<valueCount[i];k++) vals.SetDoubleAt(k,values[i][k]);
	      BSTR basis=(SinglePhasePropertyMoleBasis[i])?mole:NULL; 
	      hr=mat->SetProp(propNames[i],phaseName,empty,mixture,basis,vals.Value());
	      if (FAILED(hr))
	       {error=L"Failed to set ";
	        error+=propNames[i];
	        error+=L" for the ";
	        error+=phaseName;
	        error+=L" phase: ";
	        error+=CO_Error(mat,hr);
		    SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"CalcEquilibrium");
		    return ECapeUnknownHR;
	       }
	     }
	   }
	 }
	return NOERROR;
}

//! ICapePropertyPackage::PropCheck
/*!
  Check to see if properties can be calculated.
  \param materialObject [in] Material object reference
  \param props [in] Properties to check support for
  \param valid [out, retval] Receives an array of boolean values, VARIANT_TRUE for supported properties
*/

STDMETHODIMP CPropertyPackage::PropCheck(LPDISPATCH materialObject, VARIANT props, VARIANT * valid)
{	if ((!materialObject)||(!valid)) return E_POINTER;
	INITPP(L"PropCheck",L"ICapeThermoPropertyPackage");
	//as we do not know about phase or calculation conditions, all we do is check if the properties are known
	// (production implementations should use a case-insensitive hash table for property lookup, or improve performance in an equivalent manner)
	//check array
	CVariant propList(props,FALSE);
	wstring error;
	if (!propList.CheckArray(VT_BSTR,error))
	 {error=L"Invalid property list: "+error;
	  SetError(error.c_str(),L"ICapeThermoPropertyPackage",L"PropCheck");
	  return ECapeInvalidArgumentHR;
	 }
	//allocate return array
	CVariant results;
	results.MakeArray(propList.GetCount(),VT_BOOL);
	//loop over all properties
	int i,j;
	for (i=0;i<propList.GetCount();i++)
	 {VARIANT_BOOL supported=VARIANT_FALSE;
	  BSTR propName=propList.GetStringAt(i);
	  if (propName) //else string is empty and we simply return not supported for this element
	   {//check single-phase property
	    for (j=0;j<SinglePhasePropertyCount;j++)
	     if (CBSTR::Same(propName,SinglePhasePropertyNames[j]))
	      {supported=VARIANT_TRUE;
	       break;
	      }
	    if (!supported)
	     {//check two-phase property
	      for (j=0;j<ExposedTwoPhasePropertyCount;j++)
	       if (CBSTR::Same(propName,TwoPhasePropertyNames[j]))
	        {supported=VARIANT_TRUE;
	         break;
	        }
	      if (!supported)
	       {//check temperature dependent property
	        for (j=0;j<ExposedTDependentPropertyCount;j++)
	         if (CBSTR::Same(propName,TDependentPropertyNames[j]))
	          {supported=VARIANT_TRUE;
	           break;
	          }
	       }
	     }
	    SysFreeString(propName);
	   }
	  //set result
	  results.SetBoolAt(i,supported);
	 }
	return NOERROR;
}

//! ICapePropertyPackage::ValidityCheck
/*!
  Not implemented, see details in specification document
*/

STDMETHODIMP CPropertyPackage::ValidityCheck(LPDISPATCH materialObject, VARIANT props, VARIANT * valid)
{	if ((!materialObject)||(!valid)) return E_POINTER;
	SetError(L"Not implemented; CapeArrayThermoReliability is not defined",L"ICapeThermoPropertyPackage",L"ValidityCheck");
	return ECapeNoImplHR;
}

//! ICapePropertyPackage::GetPropList
/*!
  Get a list of all properties that can be calculated
  \param props [out, retval] Receives an array of supported properties
*/

STDMETHODIMP CPropertyPackage::GetPropList(VARIANT * props)
{	if (!props) return E_POINTER;
 	INITPP(L"GetPropList",L"ICapeThermoPropertyPackage");
 	//return a list of all properties we can calculate
 	// (production implementations should cache this list)
 	CVariant result;
 	int i,index;
 	result.MakeArray(ExposedTDependentPropertyCount+SinglePhasePropertyCount+ExposedTwoPhasePropertyCount,VT_BSTR);
 	index=0;
 	for (i=0;i<ExposedTDependentPropertyCount;i++) result.AllocStringAt(index++,TDependentPropertyNames[i]);
 	for (i=0;i<SinglePhasePropertyCount;i++) result.AllocStringAt(index++,SinglePhasePropertyNames[i]);
 	for (i=0;i<ExposedTwoPhasePropertyCount;i++) result.AllocStringAt(index++,TwoPhasePropertyNames[i]);
 	*props=result.ReturnValue();
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

//! GetCompoundsFromMaterial
/*!
Get the list of components from the material object and store in 
compIndices; in case of a failure, the error is set so that the 
caller only has to return an error code

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

\param materialObject Material object from which to get the component list (smart pointer)
\param fnc Name of calling function
\param iface Interface implementing the calling function
*/

BOOL CPropertyPackage::GetCompoundsFromMaterial(ICapeThermoMaterialObjectPtr &materialObject,const OLECHAR *fnc,const OLECHAR *iface)
{VARIANT v;
 v.vt=VT_EMPTY;
 HRESULT hr;
 wstring s;
 int i,j,count;
 //get the component list
 hr=materialObject->get_ComponentIds(&v);
 if (FAILED(hr))
  {s=L"Failed to get list of components from material object: ";
   s+=CO_Error(materialObject,hr);
   SetError(s.c_str(),iface,fnc);
   return FALSE;
  }
 //check the return value
 CVariant comps(v,TRUE); //do not delete content of v after this
 if (!comps.CheckArray(VT_BSTR,s))
  {s=L"Invalid list of compounds from material object: "+s;
   SetError(s.c_str(),iface,fnc);
   return FALSE;
  } 
 if (comps.GetCount()==0)
  {SetError(L"List of compounds from material object is empty",iface,fnc);
   return FALSE;
  }
 //get component count in property package
 if (!pack->GetCompoundCount(&count))
  {s=L"Failed to get component count in property package: ";
   s+=CA2CT(pack->LastError());
   SetError(s.c_str(),iface,fnc);
   return FALSE;
  } 
 //re-alloc compIndices
 compIndices.resize(comps.GetCount());
 //loop over components
 for (i=0;i<comps.GetCount();i++)
  {CBSTR compID=comps.GetStringAt(i);
   if (!compID) 
    {SetError(L"Invalid list of compounds from material object: contains at least one empty string",iface,fnc);
     return FALSE;
    }
   string compId=CT2CA(compID);
   //compare against the component names in the property package (ignore character case)
   for (j=0;j<count;j++)
    {const char *compName=pack->GetCompoundStringConstant(j,Name);
     if (!compName)
      {s=L"Failed to get component name in property package: ";
       s+=CA2CT(pack->LastError());
       SetError(s.c_str(),iface,fnc);
       return FALSE;
      }
     if (lstrcmpiA(compName,compId.c_str())==0)
      {//found it
       compIndices[i]=j;
       break;
      }
    }
   if (j==count)
    {//compound not found
     s=L"Invalid list of compounds from material object: compound \"";
     s+=CA2CT(compId.c_str());
     s+=L"\" does not exist";
     SetError(s.c_str(),iface,fnc);
     return FALSE;
    }
  }
 //all OK
 return TRUE;
}

//! GetPropertyFromMaterial
/*!
Get a property from the material object. The return 
value is checked, for type and count. An error message
is returned if the return value is not ok or if the 
GetProp call failed.

Empty is always assumed for the compound argument.

\param mat Material object from which to get the property (smart pointer)
\param propName Name of the property to obtain
\param phaseName Name of the phase for which to obtain the property
\param calcType Calculation type, NULL Pure or Mixture
\param basis Basis in which to obtain the property, Null mole or mass
\param expectedCount Number of values expected
\param res Receives the result
\param error Receives the error description in case of failure
\return TRUE if ok, FALSE in case of failure
*/

BOOL CPropertyPackage::GetPropertyFromMaterial(ICapeThermoMaterialObjectPtr &mat,BSTR propName,BSTR phaseName,BSTR calcType,BSTR basis,int expectedCount,CVariant &res,wstring &error)
{VARIANT v;
 v.vt=VT_EMPTY;
 HRESULT hr=mat->GetProp(propName,phaseName,empty,calcType,basis,&v);
 if (FAILED(hr))
  {error=L"Failed to get ";
   error+=propName;
   if (phaseName) if (!CBSTR::Same(phaseName,overall))
    {error+=L" for the ";
     error+=phaseName;
     error+=L" phase";
    }
   if (calcType)
    {error+=L" (calcType ";
     error+=calcType;
     error+=L')';
    }
   error+=L" from the material object: ";
   error+=CO_Error(mat,hr);
   return FALSE;
  }
 res.Set(v,TRUE);
 wstring s;
 if (!res.CheckArray(VT_R8,s,expectedCount))
  {error=L"Invalid values for ";
   error+=propName;
   if (phaseName) if (!CBSTR::Same(phaseName,overall))
    {error+=L" for the ";
     error+=phaseName;
     error+=L" phase";
    }
   if (calcType)
    {error+=L" (calcType ";
     error+=calcType;
     error+=L')';
    }
   error+=L" from the material object: ";
   error+=s;
   return FALSE;
  }
 return TRUE;
}