#pragma once
#include "MaterialObjectWrapper.h"
#include "Helpers.h"

//! MaterialObject10Wrapper class
/*!

  This is the version 1.0 implementation of the functionality outlined
  in the virtual MaterialObjectWrapper class.
  
  \sa MaterialObjectWrapper, MaterialObject11Wrapper
  
*/

class MaterialObject10Wrapper : public MaterialObjectWrapper
{   protected:
    friend class Material;

	ICapeThermoMaterialObject *mat; /*!< reference to the actual underlying version 1.0 Material Object, which is implemented by the simulation environment */

	//! Constructor.
    /*!
      Sets a reference on the material object
    */
    
    MaterialObject10Wrapper(ICapeThermoMaterialObject *mat)
    {this->mat=mat;
     mat->AddRef(); //will release at the destructor
    }
    
	//! Destructor.
    /*!
      Releases the material object
    */
    
    ~MaterialObject10Wrapper()
    {mat->Release();
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
    {IDispatch *dup;
     HRESULT hr;
     hr=mat->Duplicate(&dup); //creates a new material with copied content
     if (FAILED(hr))
      {error=L"Failed to duplicate material object: ";
       error+=CO_Error(mat,hr);
       return NULL;
      }
     ICapeThermoMaterialObject *dupMat;
     hr=dup->QueryInterface(IID_ICapeThermoMaterialObject,(LPVOID*)&dupMat);
     dup->Release();
     if (FAILED(hr))
      {error=L"Duplicate material object does not expose ICapeThermoMaterialObject";
       return NULL;
      }
     MaterialObjectWrapper *res=new MaterialObject10Wrapper(dupMat);
     dupMat->Release(); //the new MaterialObjectWrapper added a reference
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
     VARIANT v;
     v.vt=VT_EMPTY;
     hr=mat->get_ComponentIds(&v);
     if (FAILED(hr))
      {error=L"Failed to get list of compounds from material object: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //check the list
     list.Set(v,TRUE); //list must be destroyed when done
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
     hr=mat->GetPropList(&v);
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
     hr=mat->GetProp(CBSTR(propName),CBSTR(L"overall"),compIds,NULL,CBSTR(basis),&v);
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
     VARIANT v;
     v.vt=VT_EMPTY;
     hr=mat->get_PhaseIds(&v);
     if (FAILED(hr))
      {error=L"Failed to get list of present phases from material object: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //check the list
     list.Set(v,TRUE); //must be destroyed when done
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
     CVariant propList,phaseList;
     //make a list of properties
     propList.MakeArray(1,VT_BSTR);
     propList.AllocStringAt(0,propName);
     //make a list of phases
     phaseList.MakeArray(1,VT_BSTR);
     phaseList.AllocStringAt(0,phaseName);
     hr=mat->CalcProp(propList,phaseList,CBSTR(L"mixture"));
     if (FAILED(hr))
      {error=L"Failed to calculate property \"";
       error+=propName;
       error+=L"\" for phase \"";
       error+=phaseName;
       error+=L"\": ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //all ok
     return true;
    }

	//! Get value of a single-phase property
    /*!
      Obtain value(s) a property of a given phase; it is assumed that no mixture or pure properties are 
      requested via this function
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
     hr=mat->GetProp(CBSTR(propName),CBSTR(phaseName),compIds,CBSTR(calcType),CBSTR(basis),&v);
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
     VARIANT empty,v;
     CVariant scalar;
     v.vt=empty.vt=VT_EMPTY;
     //set composition
     CBSTR overall(L"overall");
     CBSTR mole(L"mole");
     hr=mat->SetProp(CBSTR(L"fraction"),overall,empty,NULL,mole,composition);
     if (FAILED(hr))
      {error=L"Failed to set overall composition on material object: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //set pressure
     scalar.MakeArray(1,VT_R8);
     scalar.SetDoubleAt(0,P);
     hr=mat->SetProp(CBSTR(L"pressure"),overall,empty,NULL,NULL,scalar);
     if (FAILED(hr))
      {error=L"Failed to set pressure on material object: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //set enthalpy
     scalar.SetDoubleAt(0,H);
     hr=mat->SetProp(CBSTR(L"enthalpy"),overall,empty,CBSTR(L"mixture"),mole,scalar);
     if (FAILED(hr))
      {error=L"Failed to set overall enthalpy on material object: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //perform PH flash
     hr=mat->CalcEquilibrium(CBSTR(L"PH"),empty);
     if (FAILED(hr))
      {error=L"PH flash calculation failed: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //get temperature
     hr=mat->GetProp(CBSTR(L"temperature"),overall,empty,NULL,NULL,&v);
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
     VARIANT empty,v;
     CVariant scalar;
     v.vt=empty.vt=VT_EMPTY;
     //set composition
     CBSTR overall(L"overall");
     CBSTR mole(L"mole");
     hr=mat->SetProp(CBSTR(L"fraction"),overall,empty,NULL,mole,composition);
     if (FAILED(hr))
      {error=L"Failed to set overall composition on material object: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //set flow
     scalar.MakeArray(1,VT_R8);
     scalar.SetDoubleAt(0,flow);
     hr=mat->SetProp(CBSTR(L"totalFlow"),overall,empty,NULL,mole,scalar);
     if (FAILED(hr))
      {error=L"Failed to set total flow on material object: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //set temperature
     scalar.SetDoubleAt(0,T);
     hr=mat->SetProp(CBSTR(L"temperature"),overall,empty,NULL,NULL,scalar);
     if (FAILED(hr))
      {error=L"Failed to set temperature on material object: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //set pressure
     scalar.SetDoubleAt(0,P);
     hr=mat->SetProp(CBSTR(L"pressure"),overall,empty,NULL,NULL,scalar);
     if (FAILED(hr))
      {error=L"Failed to set pressure on material object: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //perform TP flash
     hr=mat->CalcEquilibrium(CBSTR(L"TP"),empty);
     if (FAILED(hr))
      {error=L"TP flash calculation failed: ";
       error+=CO_Error(mat,hr);
       return false;
      }
     //all ok
     return true;
    }

};