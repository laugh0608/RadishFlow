// ThermoSystem.h : Declaration of the CThermoSystem

#pragma once
#include "resource.h"       // main symbols
#include "CAPEOPENBaseObject.h"
#include "IdealThermo_CPP_TS10.h"
#include "Variant.h"
#include "PropertyPackage.h"
#include <CPPExports.h>     // exports from the IdealThermoModule.dll

//! CThermoSystem class
/*!
	CAPE-OPEN version 1.0 ThermoSystem class.
	
	Enumerates property packages and creates property package instances.

	\sa CAPEOPENBaseObject
*/

class ATL_NO_VTABLE CThermoSystem :
	public CAPEOPENBaseObject,
	public CComCoClass<CThermoSystem, &CLSID_ThermoSystem>,
	public IDispatchImpl<ICapeThermoSystem, &__uuidof(ICapeThermoSystem), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>,
	public IDispatchImpl<ICapeUtilities, &__uuidof(ICapeUtilities), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>
{
public:

//! Constructor
/*!
  Constructor, creates a CThermoSystem class 
*/

	CThermoSystem() : CAPEOPENBaseObject(false,L"CPP Ideal Thermo System",L"CO-LaN Example Ideal Thermo System CPP implementation")
	{
	}

//! Registry entries
/*!
  This is how CAPE-OPEN PMEs find this CAPE-OPEN PMC. Details are in IdealThermo_CPP_TS10.rgs
*/

	DECLARE_REGISTRY_RESOURCEID(IDR_THERMOSYSTEM)

//! COM MAP
/*!
  ATL macro for exposed COM interfaces. BASEMAP is a macro to include the interfaces implemented by the CAPEOPENBaseObject
  
  \sa CAPEOPENBaseObject
*/

	BEGIN_COM_MAP(CThermoSystem)
		COM_INTERFACE_ENTRY2(IDispatch, ICapeThermoSystem)
		COM_INTERFACE_ENTRY(ICapeThermoSystem)
		COM_INTERFACE_ENTRY(ICapeUtilities)
		BASEMAP
	END_COM_MAP()

	DECLARE_PROTECT_FINAL_CONSTRUCT()

//! ICapeThermoSystem::GetPropertyPackages
/*!
  List the available property packages
  \param propPackageList [out,retval] Receives a string array packed in a Variant containing the names of the property packages installed on this system
  \sa ResolvePropertyPackage()
*/

	STDMETHOD(GetPropertyPackages)(VARIANT * propPackageList)
	{	if (!propPackageList) return E_POINTER;
		int i;
	    PropertyPackEnumerator ppEnum;
		int count=ppEnum.Count();
		CVariant result;
		result.MakeArray(count,VT_BSTR);
	    for (i=0;i<count;i++)
	     {string name=ppEnum.PackageName(i);
	      result.AllocStringAt(i,CA2CT(name.c_str()));
	     }
	    *propPackageList=result.ReturnValue();
		return NOERROR;
	}
	
//! ICapeThermoSystem::ResolvePropertyPackage
/*!
  Create a property package. For new property packages, the name must be a 
  property package that exists on the system. In case the property package is 
  loaded by the persistence mechanism later on, the name is not actually used
  during initialization of the property package object. Therefore, we do not 
  check at this point whether a property package with the given name is 
  configured at the system.
  \param propertyPackage [in] Name of the property package to create. Not actually used in case the property package is loaded from persistence
  \param propPackObject [out,retval] Will receive the created property package
  \sa GetPropertyPackages()
*/
	
	STDMETHOD(ResolvePropertyPackage)(BSTR propertyPackage, LPDISPATCH * propPackObject)
	{	if ((!propertyPackage)||(!propPackObject)) return E_POINTER;
	    CComObject<CPropertyPackage> *p;
	    CComObject<CPropertyPackage>::CreateInstance(&p); //create the instance with zero references
	    p->name=propertyPackage;
	    p->QueryInterface(IID_IDispatch,(LPVOID*)propPackObject); //now we have one reference, caller must release
		return NOERROR;
	}


//! ICapeUtilties::get_parameters
/*!
  Returns an ICapeCollection of parameters. This object has no parameters.
  \param parameters  [out,retval] Will receive the parameter collection
*/

	STDMETHOD(get_parameters)(LPDISPATCH * parameters)
	{if (!parameters) return E_POINTER;
	 SetError(L"No parameters are exposed by this object",L"ICapeUtilities",L"get_parameters");
	 return ECapeNoImplHR;
	}

//! ICapeUtilties::put_simulationContext
/*!
  Provides a Simulation Context object, which can be used to access PME facilities such as message logging. We don't use it.
  \param context [in] Simulation context object
*/
	
	STDMETHOD(put_simulationContext)(LPDISPATCH context)
	{return NOERROR;
	}
	
//! ICapeUtilties::Initialize
/*!
  Called upon object initialization (after InitNew or Load if we would implement it). This object does not need it
*/
	
	STDMETHOD(Initialize)()
	{return NOERROR;
	}

//! ICapeUtilties::Terminate
/*!
  Called to clean up. This object does not need it.
*/
	
	STDMETHOD(Terminate)()
	{return NOERROR;
	}

//! ICapeUtilties::Edit
/*!
  Called to edit the object. Supporting editing from within the simulation 
  environment (provided the simulation environment supports this) could be
  helpful to the user. As not all simulation environments support this, 
  it is recommended to also have an external application to edit the 
  collection of Property Packages.
  
  Note that the list of available Property Packages may have changed after 
  calling this function.
*/

	STDMETHOD(Edit)()
	{EditThermoSystem();
	 return NOERROR;
	}

};

OBJECT_ENTRY_AUTO(__uuidof(ThermoSystem), CThermoSystem)
