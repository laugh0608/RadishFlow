#pragma once
#include "MaterialObjectWrapper.h"

//! MaterialObject11Wrapper class
/*!

  This is the version 1.1 implementation of the functionality outlined
  in the virtual MaterialObjectWrapper class.
  
  \sa MaterialObjectWrapper, MaterialObject10Wrapper
  
*/

class MaterialObject11Wrapper : public MaterialObjectWrapper
{   protected:
    friend class Material;

	ICapeThermoMaterial *mat; /*!< reference to the actual underlying version 1.1 Material Object, which is implemented by the simulation environment */
	ICapeThermoPropertyRoutine *iPropRoutine; /*!< reference to the actual underlying version 1.1 Material Object, which is implemented by the simulation environment */
	ICapeThermoEquilibriumRoutine *iEqRoutine; /*!< reference to the actual underlying version 1.1 Material Object, which is implemented by the simulation environment */
	ICapeThermoCompounds *iCompounds; /*!< reference to the actual underlying version 1.1 Material Object, which is implemented by the simulation environment */
	ICapeThermoPhases *iPhases; /*!< reference to the actual underlying version 1.1 Material Object, which is implemented by the simulation environment */

	//! Constructor.
    /*!
      Sets a reference on the material object
    */
    
    MaterialObject11Wrapper(ICapeThermoMaterial *mat)
    {this->mat=mat;
     mat->AddRef(); //will release at the destructor
     iPropRoutine=NULL; //QI when first required
     iEqRoutine=NULL; //QI when first required
     iCompounds=NULL; //QI when first required
     iPhases=NULL; //QI when first required
    }
    
	//! Destructor.
    /*!
      Releases the material object
    */
    
    ~MaterialObject11Wrapper()
    {mat->Release();
     if (iPropRoutine) iPropRoutine->Release();
     if (iEqRoutine) iEqRoutine->Release();
     if (iCompounds) iCompounds->Release();
     if (iPhases) iPhases->Release();
    }

	//! Get a duplicate material
    /*!
      Get a duplicate material. Material objects connected to feed ports
      we cannot alter. This means we cannot perform calculations on such 
      materials. We can however perform calculations on duplicates thereof.
      \param error receives textual error message upon failure
      \return a MaterialObjectWrapper in case of success, NULL in case of failure
    */
    
    virtual MaterialObjectWrapper *Duplicate(wstring &error)
    {IDispatch *disp;
     HRESULT hr;
     hr=mat->CreateMaterial(&disp); //creates a new material without copied content
     if (FAILED(hr))
      {error=L"Failed to create duplicate material object: ";
       error+=CO_Error(mat,hr);
       return NULL;
      }
     ICapeThermoMaterial *dupMat;
     hr=disp->QueryInterface(IID_ICapeThermoMaterial,(LPVOID*)&dupMat);
     disp->Release();
     if (FAILED(hr))
      {error=L"Duplicate material object does not expose ICapeThermoMaterial";
       return NULL;
      }
     //copy the content
     disp=mat;
     hr=dupMat->CopyFromMaterial(&disp);
     if (FAILED(hr))
      {error=L"Failed to copy content to duplicate material object: ";
       error+=CO_Error(dupMat,hr);
       dupMat->Release();
       return NULL;
      }
     MaterialObjectWrapper *res=new MaterialObject11Wrapper(dupMat);
     dupMat->Release(); //MaterialObjectWrapper added its own reference
     return res;
    }
    
	//! Return list of compound IDs
    /*!
      Get the list of compound IDs on this material object. 
      \param list the list of compound IDs
      \param error error description in case of failure
      \return true in case of success
    */

    bool GetCompoundIDs(CVariant &list,wstring &error)
    {HRESULT hr;
     VARIANT compIds,formulae,names,boilTemps,molwts,casnos;
     compIds.vt=formulae.vt=names.vt=boilTemps.vt=molwts.vt=casnos.vt=VT_EMPTY;
     //get compound interface
     if (!iCompounds) 
      {hr=mat->QueryInterface(IID_ICapeThermoCompounds,(LPVOID*)&iCompounds);
       if (FAILED(hr))
        {error=L"Material object does not expose ICapeThermoCompounds";
         return false;
        }
      }
     hr=iCompounds->GetCompoundList(&compIds,&formulae,&names,&boilTemps,&molwts,&casnos);
     if (FAILED(hr))
      {error=L"Failed to get list of compounds from material object: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //clear the ones we do not use
     VariantClear(&formulae);
     VariantClear(&names);
     VariantClear(&boilTemps);
     VariantClear(&molwts);
     VariantClear(&casnos);
     //check the list we do use
     list.Set(compIds,TRUE); //list must be destroyed when done
     if (!list.CheckArray(VT_BSTR,error))
      {error=L"Invalid list of compound IDs from material object: "+error;
       return false;
      }
     //all ok
     return true;
    }
    
	//! Return list of single phase properties
    /*!
      Get the list of single phase properties supported by the material object
      \param list the list of properties
      \param error error description in case of failure
      \return true in case of success
    */
    
    bool GetSinglePhasePropList(CVariant &list,wstring &error)
    {HRESULT hr;
     VARIANT v;
     v.vt=VT_EMPTY;
     //get property routine
     if (!iPropRoutine) 
      {hr=mat->QueryInterface(IID_ICapeThermoPropertyRoutine,(LPVOID*)&iPropRoutine);
       if (FAILED(hr))
        {error=L"Material object does not expose ICapeThermoPropertyRoutine";
         return false;
        }
      }
     hr=iPropRoutine->GetSinglePhasePropList(&v);
     if (FAILED(hr))
      {error=L"Failed to get list of properties from material object: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //check the list
     list.Set(v,TRUE); //list must be destroyed when done
     if (!list.CheckArray(VT_BSTR,error))
      {error=L"Invalid list of properties from material object: "+error;
       return false;
      }
     //all ok
     return true;
    }
    
	//! Get value of an overall property
    /*!
      Get a value of an overall property; it is assumed that no mixture or pure properties are 
      requested via this function
      \param propName property identifier
      \param basis property basis, can be NULL
      \param value receive the property value(s)
      \param error error description in case of failure
      \return true in case of success
    */

    bool GetOverallProperty(const OLECHAR *propName,const OLECHAR *basis,CVariant &value,wstring &error)
    {HRESULT hr;
     VARIANT v,compIds;
     ATLASSERT(propName!=NULL);
     v.vt=VT_EMPTY;
     compIds.vt=VT_EMPTY;
     hr=mat->GetOverallProp(CBSTR(propName),CBSTR(basis),&v);
     if (FAILED(hr))
      {error=L"Failed to get overall property \"";
       error+=propName;
       error+=L"\" from material object: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //check result
     value.Set(v,TRUE); //must be destroyed
     if (!value.CheckArray(VT_R8,error))
      {wstring s; 
       s=L"Invalid property value for overall property \"";
       s+=propName;
       s+=L"\" from material object: ";
       s+=error;
       error=s;
       return false;
      }
     //all ok
     return true;
    }
    
	//! Get list of present phases
    /*!
      Get the list of phases currently present on the material object
      requested via this function
      \param list receives the list of present phases
      \param error error description in case of failure
      \return true in case of success
    */
    
    bool GetListOfPresentPhases(CVariant &list,wstring &error)
    {HRESULT hr;
     VARIANT phases,status;
     phases.vt=VT_EMPTY;
     status.vt=VT_EMPTY;
     hr=mat->GetPresentPhases(&phases,&status);
     if (FAILED(hr))
      {error=L"Failed to get list of present phases from material object: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //ignore status
     VariantClear(&status);
     //check the list
     list.Set(phases,TRUE); //must be destroyed when done
     if (!list.CheckArray(VT_BSTR,error))
      {error=L"Invalid list of present phases from material object: "+error;
       return false;
      }
     //all ok
     return true;
    }

	//! Calculate a single phase property
    /*!
      Calculate a single phase property for a given phase. It is assumed that only mixture properties will
      be calculated. Unit operation implementations that calculate multiple properties at the same 
      conditions should modify this function to allow for multiple property calculations in a single call.
      \param propName name of the property to calculate
      \param phaseName phase for which to calculate the property
      \param error error description in case of failure
      \return true in case of success
    */
    
    bool CalcSinglePhaseProperty(const OLECHAR *propName,const OLECHAR *phaseName,wstring &error)
    {HRESULT hr;
     CVariant propList;
     //get IPropertyRoutine interface
     if (!iPropRoutine) 
      {hr=mat->QueryInterface(IID_ICapeThermoPropertyRoutine,(LPVOID*)&iPropRoutine);
       if (FAILED(hr))
        {error=L"Material object does not expose ICapeThermoPropertyRoutine";
         return false;
        }
      }
     //make a list of properties
     propList.MakeArray(1,VT_BSTR);
     propList.AllocStringAt(0,propName);
     hr=iPropRoutine->CalcSinglePhaseProp(propList,CBSTR(phaseName));
     if (FAILED(hr))
      {error=L"Failed to calculate property \"";
       error+=propName;
       error+=L"\" for phase \"";
       error+=phaseName;
       error+=L"\": ";
       error+=CO_Error(iPropRoutine,hr);
       return false;
      }
     //all ok
     return true;
    }

	//! Get value of a single-phase property
    /*!
      Obtain value(s) a property of a given phase
      \param propName property identifier
      \param phaseName phase for which to get the property
      \param calcType: mixture for mixture properties or NULL for fraction or phaseFraction. Ignored for version 1.1 thermo.
      \param basis property basis, can be NULL
      \param value receive the property value(s)
      \param error error description in case of failure
      \return true in case of success
    */
    
    bool GetSinglePhaseProperty(const OLECHAR *propName,const OLECHAR *phaseName,const OLECHAR *calcType,const OLECHAR *basis,CVariant &value,wstring &error)
    {HRESULT hr;
     VARIANT v,compIds;
     ATLASSERT(propName!=NULL);
     v.vt=VT_EMPTY;
     compIds.vt=VT_EMPTY;
     hr=mat->GetSinglePhaseProp(CBSTR(propName),CBSTR(phaseName),CBSTR(basis),&v);
     if (FAILED(hr))
      {error=L"Failed to get property \"";
       error+=propName;
       error+=L"\" for phase \"";
       error+=phaseName;
       error+=L"\" from material object: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //check result
     value.Set(v,TRUE); //must be destroyed
     if (!value.CheckArray(VT_R8,error))
      {wstring s; 
       s=L"Invalid property value for property \"";
       s+=propName;
       s+=L"\" for phase \"";
       s+=phaseName;
       s+=L"\" from material object: ";
       s+=error;
       error=s;
       return false;
      }
     //all ok
     return true;
    }

	//! Get temperature from a PH flash at given P, H and composition
    /*!
      Calculate and return the temperature corresponding to a mixture at 
      given composition, enthalpy and pressure
      \param composition overall composition [mol/mol]
      \param P pressure [Pa]
      \param H enthalpy [J/mol]
      \param T receives temperature [K]
      \param error error description in case of failure
      \return true in case of success
    */
    
    bool GetTemperatureFromPHFlash(CVariant &composition,double P,double H,double &T,wstring &error)
    {HRESULT hr;
     int i;
     VARIANT empty,v,phaseList,aggState,keyComps;
     CVariant scalar;
     v.vt=empty.vt=VT_EMPTY;
     //set composition
     CBSTR mole(L"mole");
     hr=mat->SetOverallProp(CBSTR(L"fraction"),mole,composition);
     if (FAILED(hr))
      {error=L"Failed to set overall composition on material object: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //set pressure
     scalar.MakeArray(1,VT_R8);
     scalar.SetDoubleAt(0,P);
     hr=mat->SetOverallProp(CBSTR(L"pressure"),NULL,scalar);
     if (FAILED(hr))
      {error=L"Failed to set pressure on material object: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //set enthalpy
     scalar.SetDoubleAt(0,H);
     hr=mat->SetOverallProp(CBSTR(L"enthalpy"),mole,scalar);
     if (FAILED(hr))
      {error=L"Failed to set overall enthalpy on material object: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //we are going to perform a flash that will allow all possible phases as result. 
     // We need to get a list of all possible phases. Get ICapeThermo Phases interface
     if (!iPhases) 
      {hr=mat->QueryInterface(IID_ICapeThermoPhases,(LPVOID*)&iPhases);
       if (FAILED(hr))
        {error=L"Material object does not expose ICapeThermoPhases";
         return false;
        }
      }
     //get the total phase list 
     phaseList.vt=aggState.vt=keyComps.vt=VT_EMPTY;
     hr=iPhases->GetPhaseList(&phaseList,&aggState,&keyComps);
     if (FAILED(hr))
      {error=L"Failed to get list of possible phases from material object: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //ignore aggregation states and key compounds
     VariantClear(&aggState);
     VariantClear(&keyComps);
     //check phase list
     CVariant phaseLabels(phaseList,TRUE); //must be destroyed when done
     if (!phaseLabels.CheckArray(VT_BSTR,error))
      {error=L"Invalid list of possible phases from material object: "+error;
       return false;
      }
     //set present phases on MO
     CVariant phaseStatus;
     phaseStatus.MakeArray(phaseLabels.GetCount(),VT_I4);
     for (i=0;i<phaseLabels.GetCount();i++) phaseStatus.SetLongAt(i,CAPE_UNKNOWNPHASESTATUS); //we do not have an initial guess
     hr=mat->SetPresentPhases(phaseLabels,phaseStatus);
     if (FAILED(hr))
      {error=L"Failed to set list of present phases on material object: "+error;
       return false;
      }
     //the flash is performed by the ICapeThermoEquilibriumRoutine interface
     if (!iEqRoutine) 
      {hr=mat->QueryInterface(IID_ICapeThermoEquilibriumRoutine,(LPVOID*)&iEqRoutine);
       if (FAILED(hr))
        {error=L"Material object does not expose ICapeThermoEquilibriumRoutine";
         return false;
        }
      }
     //generate flash specifications
     CVariant flashSpec1,flashSpec2;
     CBSTR overall(L"overall");
     //flash specification 1: overall enthalpy
     flashSpec1.MakeArray(3,VT_BSTR);
     flashSpec1.AllocStringAt(0,L"enthalpy");
     flashSpec1.SetStringAt(1,NULL);
     flashSpec1.SetStringAt(2,overall);
     //flash specification 2: overall pressure
     flashSpec2.MakeArray(3,VT_BSTR);
     flashSpec2.AllocStringAt(0,L"pressure");
     flashSpec2.SetStringAt(1,NULL);
     flashSpec2.SetStringAt(2,overall);
     //perform PH flash
     hr=iEqRoutine->CalcEquilibrium(flashSpec1,flashSpec2,CBSTR(L"unspecified"));
     if (FAILED(hr))
      {error=L"PH flash calculation failed: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //get temperature
     hr=mat->GetOverallProp(CBSTR(L"temperature"),NULL,&v);
     if (FAILED(hr))
      {error=L"Failed to obtain temperature after PH flash: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //check value
     scalar.Set(v,TRUE); //must be deleted when done
     if (!scalar.CheckArray(VT_R8,error))
      {error=L"Invalid values for temprature from material object: "+error;
       return false;
      }
     //all ok
     T=scalar.GetDoubleAt(0);
     return true;
    }

	//! Specify a material object using composition, T and P
    /*!
      Specify a material object using total flow, composition, T and P. Perform a TP flash to complete the specification.
      \param composition overall composition [mol/mol]
      \param flow total flow [mol/s]
      \param P pressure [Pa]
      \param T temperature [K]
      \param error error description in case of failure
      \return true in case of success
    */
    
    bool SetFromFlowTPX(CVariant &composition,double flow,double T,double P,wstring &error)
    {HRESULT hr;
     int i;
     VARIANT empty,v,phaseList,aggState,keyComps;
     CVariant scalar;
     v.vt=empty.vt=VT_EMPTY;
     //set composition
     CBSTR mole(L"mole");
     hr=mat->SetOverallProp(CBSTR(L"fraction"),mole,composition);
     if (FAILED(hr))
      {error=L"Failed to set overall composition on material object: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //set total flow
     scalar.MakeArray(1,VT_R8);
     scalar.SetDoubleAt(0,flow);
     hr=mat->SetOverallProp(CBSTR(L"totalFlow"),mole,scalar);
     if (FAILED(hr))
      {error=L"Failed to set total flow on material object: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //set temperature
     scalar.SetDoubleAt(0,T);
     hr=mat->SetOverallProp(CBSTR(L"temperature"),NULL,scalar);
     if (FAILED(hr))
      {error=L"Failed to set temperature on material object: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //set pressure
     scalar.SetDoubleAt(0,P);
     hr=mat->SetOverallProp(CBSTR(L"pressure"),NULL,scalar);
     if (FAILED(hr))
      {error=L"Failed to set pressure on material object: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //we are going to perform a flash that will allow all possible phases as result. 
     // We need to get a list of all possible phases. Get ICapeThermo Phases interface
     if (!iPhases) 
      {hr=mat->QueryInterface(IID_ICapeThermoPhases,(LPVOID*)&iPhases);
       if (FAILED(hr))
        {error=L"Material object does not expose ICapeThermoPhases";
         return false;
        }
      }
     //get the total phase list 
     phaseList.vt=aggState.vt=keyComps.vt=VT_EMPTY;
     hr=iPhases->GetPhaseList(&phaseList,&aggState,&keyComps);
     if (FAILED(hr))
      {error=L"Failed to get list of possible phases from material object: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //ignore aggregation states and key compounds
     VariantClear(&aggState);
     VariantClear(&keyComps);
     //check phase list
     CVariant phaseLabels(phaseList,TRUE); //must be destroyed when done
     if (!phaseLabels.CheckArray(VT_BSTR,error))
      {error=L"Invalid list of possible phases from material object: "+error;
       return false;
      }
     //set present phases on MO
     CVariant phaseStatus;
     phaseStatus.MakeArray(phaseLabels.GetCount(),VT_I4);
     for (i=0;i<phaseLabels.GetCount();i++) phaseStatus.SetLongAt(i,CAPE_UNKNOWNPHASESTATUS); //we do not have an initial guess
     hr=mat->SetPresentPhases(phaseLabels,phaseStatus);
     if (FAILED(hr))
      {error=L"Failed to set list of present phases on material object: "+error;
       return false;
      }
     //the flash is performed by the ICapeThermoEquilibriumRoutine interface
     if (!iEqRoutine) 
      {hr=mat->QueryInterface(IID_ICapeThermoEquilibriumRoutine,(LPVOID*)&iEqRoutine);
       if (FAILED(hr))
        {error=L"Material object does not expose ICapeThermoEquilibriumRoutine";
         return false;
        }
      }
     //generate flash specifications
     CVariant flashSpec1,flashSpec2;
     CBSTR overall(L"overall");
     //flash specification 1: overall temperature
     flashSpec1.MakeArray(3,VT_BSTR);
     flashSpec1.AllocStringAt(0,L"temperature");
     flashSpec1.SetStringAt(1,NULL);
     flashSpec1.SetStringAt(2,overall);
     //flash specification 2: overall pressure
     flashSpec2.MakeArray(3,VT_BSTR);
     flashSpec2.AllocStringAt(0,L"pressure");
     flashSpec2.SetStringAt(1,NULL);
     flashSpec2.SetStringAt(2,overall);
     //perform TP flash
     hr=iEqRoutine->CalcEquilibrium(flashSpec1,flashSpec2,CBSTR(L"unspecified"));
     if (FAILED(hr))
      {error=L"TP flash calculation failed: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //all ok
     return true;
    }

};