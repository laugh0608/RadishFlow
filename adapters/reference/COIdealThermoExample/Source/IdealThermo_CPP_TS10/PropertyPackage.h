// PropertyPackage.h : Declaration of the CPropertyPackage

#pragma once
#include "resource.h"       // main symbols
#include "IdealThermo_CPP_TS10.h"
#include "CAPEOPENBaseObject.h"
#include <CPPExports.h>
#include "BSTR.h"

class CVariant;

//! Property Package Class
/*!

This class impements the version 1.0 Property Package. It is not 
created directly by clients, so no registration for this class is 
required. Instead, clients get access to this class through 
the CThermoSystem class

\sa CThermoSystem

*/

class ATL_NO_VTABLE CPropertyPackage :
	public CAPEOPENBaseObject,
	public CComCoClass<CPropertyPackage, &CLSID_PropertyPackage>,
	public IPersistStream,
	public IDispatchImpl<ICapeThermoPropertyPackage, &__uuidof(ICapeThermoPropertyPackage), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>,
	public IDispatchImpl<ICapeUtilities, &__uuidof(ICapeUtilities), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>
{

public:

//! Constructor
/*!
  Called upon creation of the CPropertyPackage COM object
*/

	CPropertyPackage() : CAPEOPENBaseObject(true,L"Property Package",L"CO-LaN Example Ideal Property Package v1.0 CPP implementation") //name will be overriden during creation
	{pack=NULL;
	 terminated=false;
	 //pre-allocate some BSTR values that we frequently use
	 // (allocated and freed by CBSTR wrapper class)
	 mole=L"mole";
	 temperature=L"temperature";
	 pressure=L"pressure";
	 mixture=L"mixture";
	 pure=L"pure";
	 fraction=L"fraction";
	 phaseFraction=L"phaseFraction";
	 enthalpy=L"enthalpy";
	 entropy=L"entropy";
	 volume=L"volume";
	 vapor=L"Vapor";
	 liquid=L"Liquid";
	 overall=L"Overall";
	 empty.vt=VT_EMPTY;
	}

//! Destructor
/*!
  Called upon destruction of the CPropertyPackage COM object
*/
	
	~CPropertyPackage()
	{//in case Terminate was not called, call it
	 Terminate();
    }

//! Registry entries
/*!
  This object is not registered; it cannot be created by PMEs (only indirectly via ThermoSystem
  \sa CThermoSystem
*/

	DECLARE_NO_REGISTRY()

//! COM MAP
/*!
  ATL macro for exposed COM interfaces. BASEMAP is a macro to include the interfaces implemented by the CAPEOPENBaseObject
  
  \sa CAPEOPENBaseObject
*/

	BEGIN_COM_MAP(CPropertyPackage)
		COM_INTERFACE_ENTRY2(IDispatch, ICapeThermoPropertyPackage)
		COM_INTERFACE_ENTRY(ICapeThermoPropertyPackage)
		COM_INTERFACE_ENTRY(IPersistStream)
		COM_INTERFACE_ENTRY(ICapeUtilities)
		BASEMAP
	END_COM_MAP()

	DECLARE_PROTECT_FINAL_CONSTRUCT()

	// ICapeThermoPropertyPackage Methods

	STDMETHOD(GetPhaseList)(VARIANT * phases);
	STDMETHOD(GetComponentList)(VARIANT * compIds, VARIANT * formulae, VARIANT * name, VARIANT * boilTemps, VARIANT * molwt, VARIANT * casno);
	STDMETHOD(GetUniversalConstant)(LPDISPATCH materialObject, VARIANT props, VARIANT * propVals);
	STDMETHOD(GetComponentConstant)(LPDISPATCH materialObject, VARIANT props, VARIANT * propVals);
	STDMETHOD(CalcProp)(LPDISPATCH materialObject, VARIANT props, VARIANT phases, BSTR calcType);
	STDMETHOD(CalcEquilibrium)(LPDISPATCH materialObject, BSTR flashType, VARIANT props);
	STDMETHOD(PropCheck)(LPDISPATCH materialObject, VARIANT props, VARIANT * valid);
	STDMETHOD(ValidityCheck)(LPDISPATCH materialObject, VARIANT props, VARIANT * valid);
	STDMETHOD(GetPropList)(VARIANT * props);

	// ICapeUtilities Methods

	STDMETHOD(get_parameters)(LPDISPATCH * parameters);
	STDMETHOD(put_simulationContext)(LPDISPATCH );
	STDMETHOD(Initialize)();
	STDMETHOD(Terminate)();
	STDMETHOD(Edit)();
	
	//IPersistStream Methods
	STDMETHOD(IsDirty)();
	STDMETHOD(Load)(IStream * pstm);
	STDMETHOD(Save)(IStream * pstm, BOOL fClearDirty);
	STDMETHOD(GetSizeMax)(_ULARGE_INTEGER * pcbSize);
	STDMETHOD(GetClassID)(CLSID *pClassID);
	
	//members
	bool terminated; /*!< set to true if Terminate has been called */
	PropertyPack *pack; /*!< the actual package doing the work, from IdealThermoModule.dll */
	string ppfilename; /*!< file name to load the PP from; used in case loaded from persistence */
	vector<int> compIndices; /*!< internal buffer for component indices */
	CBSTR mole; /*!< BSTR values for "mole" */
	CBSTR temperature; /*!< BSTR values for "temperature" */
	CBSTR pressure; /*!< BSTR values for "pressure" */
	CBSTR mixture; /*!< BSTR values for "mixture" */
	CBSTR pure; /*!< BSTR values for "pure" */
	CBSTR fraction; /*!< BSTR values for "fraction" */
	CBSTR phaseFraction; /*!< BSTR values for "phaseFraction" */
	CBSTR enthalpy; /*!< BSTR values for "enthalpy" */
	CBSTR entropy; /*!< BSTR values for "entropy" */
	CBSTR volume; /*!< BSTR values for "volume" */
	CBSTR vapor; /*!< BSTR values for "vapor" */
	CBSTR liquid; /*!< BSTR values for "liquid" */
	CBSTR overall; /*!< BSTR values for "overall" */
	VARIANT empty; /*!< VARIANT value that we often use */
	
	//utility functions
	string GetTempFileName();
	BOOL GetCompoundsFromMaterial(ICapeThermoMaterialObjectPtr &materialObject,const OLECHAR *fnc,const OLECHAR *iface);
	BOOL GetPropertyFromMaterial(ICapeThermoMaterialObjectPtr &mat,BSTR propName,BSTR phaseName,BSTR calcType,BSTR basis,int expectedCount,CVariant &res,std::wstring &error);

};

OBJECT_ENTRY_AUTO(__uuidof(PropertyPackage), CPropertyPackage)
