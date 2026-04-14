// PropertyPackageManager.h : Declaration of the CPropertyPackageManager

#pragma once
#include "resource.h"       // main symbols
#include "CAPEOPENBaseObject.h"
#include "IdealThermo_CPP_PPM11.h"
#include "PropertyPackage.h"
#include "Variant.h"
#include <CPPExports.h>     // exports from the IdealThermoModule.dll

//! CThermoSystem class
/*!
	CAPE-OPEN version 1.1 PropertyPackageManager class.
	
	Enumerates property packages and creates property package instances.

	\sa CAPEOPENBaseObject
*/

class ATL_NO_VTABLE CPropertyPackageManager :
	public CAPEOPENBaseObject,
	public CComCoClass<CPropertyPackageManager, &CLSID_PropertyPackageManager>,
	public IDispatchImpl<ICapeThermoPropertyPackageManager, &__uuidof(ICapeThermoPropertyPackageManager), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>,
	public IDispatchImpl<ICapeUtilities, &__uuidof(ICapeUtilities), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>
{
public:

//! Constructor
/*!
  Constructor, creates a CPropertyPackageManager class 
*/

	CPropertyPackageManager() : CAPEOPENBaseObject(false,L"CPP Ideal Property Package Manager",L"CO-LaN Example Ideal Property Package Manager CPP implementation")
	{
	}

//! Registry entries
/*!
  This is how CAPE-OPEN PMEs find this CAPE-OPEN PMC. Details are in IdealThermo_CPP_PPM11.rgs
*/

	DECLARE_REGISTRY_RESOURCEID(IDR_PROPERTYPACKAGEMANAGER) 

//! COM MAP
/*!
  ATL macro for exposed COM interfaces. BASEMAP is a macro to include the interfaces implemented by the CAPEOPENBaseObject
  
  \sa CAPEOPENBaseObject
*/

	BEGIN_COM_MAP(CPropertyPackageManager)
		COM_INTERFACE_ENTRY2(IDispatch, ICapeThermoPropertyPackageManager)
		COM_INTERFACE_ENTRY(ICapeThermoPropertyPackageManager)
		COM_INTERFACE_ENTRY(ICapeUtilities)
		BASEMAP
	END_COM_MAP()


	DECLARE_PROTECT_FINAL_CONSTRUCT()

//! ICapeThermoPropertyPackageManager::GetPropertyPackageList
/*!
  List the available property packages
  \param PackageNames [out,retval] Receives a string array packed in a Variant containing the names of the property packages installed on this system
  \sa GetPropertyPackage()
*/

	STDMETHOD(GetPropertyPackageList)(VARIANT * PackageNames)
	{	if (!PackageNames) return E_POINTER;
		int i;
	    PropertyPackEnumerator ppEnum;
		int count=ppEnum.Count();
		CVariant result;
		result.MakeArray(count,VT_BSTR);
	    for (i=0;i<count;i++)
	     {string name=ppEnum.PackageName(i);
	      result.AllocStringAt(i,CA2CT(name.c_str()));
	     }
	    *PackageNames=result.ReturnValue();
		return NOERROR;
	}
	
//! ICapeThermoPropertyPackageManager::GetPropertyPackage
/*!
  Create a property package. For new property packages, the name must be a 
  property package that exists on the system. In case the property package is 
  loaded by the persistence mechanism later on, the name is not actually used
  during initialization of the property package object. Therefore, we do not 
  check at this point whether a property package with the given name is 
  configured at the system.
  \param PackageName [in] Name of the property package to create. Not actually used in case the property package is loaded from persistence
  \param Package [out,retval] Will receive the created property package
  \sa GetPropertyPackageList()
*/
	
	STDMETHOD(GetPropertyPackage)(BSTR PackageName, LPDISPATCH * Package)
	{	if ((!PackageName)||(!Package)) return E_POINTER;
	    CComObject<CPropertyPackage> *p;
	    CComObject<CPropertyPackage>::CreateInstance(&p); //create the instance with zero references
	    p->name=PackageName;
	    p->QueryInterface(IID_IDispatch,(LPVOID*)Package); //now we have one reference, caller must release
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

OBJECT_ENTRY_AUTO(__uuidof(PropertyPackageManager), CPropertyPackageManager)
