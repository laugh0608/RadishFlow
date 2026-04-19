// PropertyPackage.h : Declaration of the CPropertyPackage

#pragma once
#include "resource.h"       // main symbols
#include "IdealThermo_CPP_PPM11.h"
#include "CAPEOPENBaseObject.h"
#include <CPPExports.h>
#include "BSTR.h"


//! Property Package Class
/*!

This class impements the version 1.1 Property Package. It is not 
created directly by clients, so no registration for this class is 
required. Instead, clients get access to this class through 
the CPropertyPackageManager class

\sa CPropertyPackageManager

*/

class ATL_NO_VTABLE CPropertyPackage :
	public CAPEOPENBaseObject,
	public CComCoClass<CPropertyPackage, &CLSID_PropertyPackage>,
	public IDispatchImpl<ICapeThermoCompounds, &__uuidof(ICapeThermoCompounds), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>,
	public IDispatchImpl<ICapeThermoEquilibriumRoutine, &__uuidof(ICapeThermoEquilibriumRoutine), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>,
	public IDispatchImpl<ICapeThermoMaterialContext, &__uuidof(ICapeThermoMaterialContext), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>,
	public IDispatchImpl<ICapeThermoPhases, &__uuidof(ICapeThermoPhases), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>,
	public IDispatchImpl<ICapeThermoPropertyRoutine, &__uuidof(ICapeThermoPropertyRoutine), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>,
	public IDispatchImpl<ICapeThermoUniversalConstant, &__uuidof(ICapeThermoUniversalConstant), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>,
	public IDispatchImpl<ICapeUtilities, &__uuidof(ICapeUtilities), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>,
	public IPersistStream
{
public:

//! Constructor
/*!
  Called upon creation of the CPropertyPackage COM object
*/

	CPropertyPackage()  : CAPEOPENBaseObject(true,L"Property Package",L"CO-LaN Example Ideal Property Package v1.1 CPP implementation") //name will be overriden during creation
	{terminated=false;
	 pack=NULL;
	 //pre-allocate some BSTR values that we frequently use
	 // (allocated and freed by CBSTR wrapper class)
	 mole=L"mole";
	 mass=L"mass";
	 temperature=L"temperature";
	 pressure=L"pressure";
	 mixture=L"mixture";
	 pure=L"pure";
	 fraction=L"fraction";
	 phaseFraction=L"phaseFraction";
	 enthalpy=L"enthalpy";
	 entropy=L"entropy";
	 volume=L"volume";
	 gas=L"Gas";
	 liquid=L"Liquid";
	 empty.vt=VT_EMPTY;
	}
	
//! Destructor
/*!
  Called upon destruction of the CPropertyPackage COM object
*/

	~CPropertyPackage()
	{Terminate(); //make sure (in case the PME failed to do so)
	}

//! Registry entries
/*!
  This object is not registered; it cannot be created by PMEs (only indirectly via ThermoSystem
  \sa CPropertyPackageManager
*/

	DECLARE_NO_REGISTRY()

//! COM MAP
/*!
  ATL macro for exposed COM interfaces. BASEMAP is a macro to include the interfaces implemented by the CAPEOPENBaseObject
  
  \sa CAPEOPENBaseObject
*/

	BEGIN_COM_MAP(CPropertyPackage)
		COM_INTERFACE_ENTRY2(IDispatch, ICapeThermoCompounds)
		COM_INTERFACE_ENTRY(ICapeThermoCompounds)
		COM_INTERFACE_ENTRY(ICapeThermoEquilibriumRoutine)
		COM_INTERFACE_ENTRY(ICapeThermoMaterialContext)
		COM_INTERFACE_ENTRY(ICapeThermoPhases)
		COM_INTERFACE_ENTRY(ICapeThermoPropertyRoutine)
		COM_INTERFACE_ENTRY(ICapeThermoUniversalConstant)
		COM_INTERFACE_ENTRY(ICapeUtilities)
		COM_INTERFACE_ENTRY(IPersistStream)
		BASEMAP
	END_COM_MAP()

	DECLARE_PROTECT_FINAL_CONSTRUCT()

	// ICapeThermoCompounds Methods

	STDMETHOD(GetCompoundConstant)(VARIANT props, VARIANT compIds, VARIANT * propVals);
	STDMETHOD(GetCompoundList)(VARIANT * compIds, VARIANT * formulae, VARIANT * names, VARIANT * boilTemps, VARIANT * molwts, VARIANT * casnos);
	STDMETHOD(GetConstPropList)(VARIANT * props);
	STDMETHOD(GetNumCompounds)(long * num);
	STDMETHOD(GetPDependentProperty)(VARIANT props, double pressure, VARIANT compIds, VARIANT * propVals);
	STDMETHOD(GetPDependentPropList)(VARIANT * props);
	STDMETHOD(GetTDependentProperty)(VARIANT props, double temperature, VARIANT compIds, VARIANT * propVals);
	STDMETHOD(GetTDependentPropList)(VARIANT * props);

	// ICapeThermoEquilibriumRoutine Methods

	STDMETHOD(CalcEquilibrium)(VARIANT specification1, VARIANT specification2, BSTR solutionType);
	STDMETHOD(CheckEquilibriumSpec)(VARIANT specification1, VARIANT specification2, BSTR solutionType, VARIANT_BOOL * isSupported);

	// ICapeThermoMaterialContext Methods

	STDMETHOD(SetMaterial)(LPDISPATCH material);
	STDMETHOD(UnsetMaterial)();

	// ICapeThermoPhases Methods

	STDMETHOD(GetNumPhases)(long * num);
	STDMETHOD(GetPhaseInfo)(BSTR phaseLabel, BSTR phaseAttribute, VARIANT * value);
	STDMETHOD(GetPhaseList)(VARIANT * phaseLabels, VARIANT * stateOfAggregation, VARIANT * keyCompoundId);

	// ICapeThermoPropertyRoutine Methods

	STDMETHOD(CalcAndGetLnPhi)(BSTR phaseLabel, double temperature, double pressure, VARIANT moleNumbers, int fFlags, VARIANT * lnPhi, VARIANT * lnPhiDT, VARIANT * lnPhiDP, VARIANT * lnPhiDn);
	STDMETHOD(CalcSinglePhaseProp)(VARIANT props, BSTR phaseLabel);
	STDMETHOD(CalcTwoPhaseProp)(VARIANT props, VARIANT phaseLabels);
	STDMETHOD(CheckSinglePhasePropSpec)(BSTR property, BSTR phaseLabel, VARIANT_BOOL * valid);
	STDMETHOD(CheckTwoPhasePropSpec)(BSTR property, VARIANT phaseLabels, VARIANT_BOOL * valid);
	STDMETHOD(GetSinglePhasePropList)(VARIANT * props);
	STDMETHOD(GetTwoPhasePropList)(VARIANT * props);

	// ICapeThermoUniversalConstant Methods

	STDMETHOD(GetUniversalConstant)(BSTR constantId, VARIANT * constantValue);
	STDMETHOD(GetUniversalConstantList)(VARIANT * constantIds);

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

	bool terminated; /*!< set to true if Terminate has been called */
	PropertyPack *pack; /*!< the actual package doing the work, from IdealThermoModule.dll */
	string ppfilename; /*!< file name to load the PP from; used in case loaded from persistence */
	CBSTR mole; /*!< BSTR values for "mole" */
	CBSTR mass; /*!< BSTR values for "mass" */
	CBSTR temperature; /*!< BSTR values for "temperature" */
	CBSTR pressure; /*!< BSTR values for "pressure" */
	CBSTR mixture; /*!< BSTR values for "mixture" */
	CBSTR pure; /*!< BSTR values for "pure" */
	CBSTR fraction; /*!< BSTR values for "fraction" */
	CBSTR phaseFraction; /*!< BSTR values for "phaseFraction" */
	CBSTR enthalpy; /*!< BSTR values for "enthalpy" */
	CBSTR entropy; /*!< BSTR values for "entropy" */
	CBSTR volume; /*!< BSTR values for "volume" */
	CBSTR gas; /*!< BSTR values for "gas" */
	CBSTR liquid; /*!< BSTR values for "liquid" */
	VARIANT empty; /*!< VARIANT value that we often use */
	ICapeThermoMaterialPtr contextMaterial; /*!< Smart COM pointer to context material object */
	vector<int> contextMaterialCompoundIndices; /*!< Compound indices on the context material */

	//helper functions
	string GetTempFileName();
	BOOL GetCompounds(VARIANT compoundList,std::vector<int> &compoundIndices,BOOL allowEmptyList,std::wstring &error);
	BOOL GetFlashSpec(VARIANT specification1, VARIANT specification2, BSTR solutionType,FlashType &type,FlashPhaseType &phaseType,std::wstring &error);

};

OBJECT_ENTRY_AUTO(__uuidof(PropertyPackage), CPropertyPackage)
