// RealParameter.h : Declaration of the CRealParameter

#pragma once
#include "resource.h"       // main symbols

#include "CPPMixerSplitterexample.h"
#include "CAPEOPENBaseObject.h"

#include <float.h>

//! Real parameter class
/*!
  CAPE-OPEN class that implements a real parameter.
  A CAPE-OPEN parameter gives a reference to a ParameterSpecification 
  object; the parameter specification of this parameter is implemented
  by the parameter itself. So this object implements a real parameter, 
  a parameter specification and a real parameter specification 
  interface (and derives from the CAPE-OPEN base object to implement
  identification and error interfaces)
  
  We ensure that the current value of the parameter is always valid.
  This way the implementation of the Validation is trivial
*/

class ATL_NO_VTABLE CRealParameter :
	public CComCoClass<CRealParameter, &CLSID_RealParameter>,
	public IDispatchImpl<ICapeParameter, &__uuidof(ICapeParameter), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>,
	public IDispatchImpl<ICapeParameterSpec, &__uuidof(ICapeParameterSpec), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>,
	public IDispatchImpl<ICapeRealParameterSpec, &__uuidof(ICapeRealParameterSpec), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>,
	public CAPEOPENBaseObject
{
public:

    double minVal; /*!< the upper limit of this parameter, can be NaN */
    double maxVal; /*!< the lower limit of this parameter, can be NaN */
    double defVal; /*!< the default value of this parameter; used to initialize as well, so cannot be NaN*/
    double value;  /*!< the current value of this parameter, must always be valid */
    CVariant dimensionality; /*!< the dimensionality of this parameter */
    CapeValidationStatus *valStatus; /*!< points to the unit operation's validation status */

	//! Helper function for creating the parameter 
    /*!
      Helper function for creating the parameter as an exposable COM object. After calling CreateParameter, the reference
      count is one; do not delete the returned object. Use Release() instead
      \param name name of the parameter
      \param description description of the parameter
      \param minVal the minimum value of the parameter
      \param maxVal the maximum value of the parameter
      \param defVal the default value of the parameter
      \param dimensionality the dimensionality of this parameter      
      \param valStatus points to the unit operation's validation status
      \sa CRealParameter()
    */
    
    static CComObject<CRealParameter> *CreateParameter(const OLECHAR *name,const OLECHAR *description, double minVal, double maxVal, double defVal,vector<double> dimensionality,CapeValidationStatus *valStatus)
    {unsigned int i;
     CComObject<CRealParameter> *p;
     CComObject<CRealParameter>::CreateInstance(&p); //create the instance with zero references
     p->AddRef(); //now it has one reference, the caller must Release this object
     p->name=name;
     p->description=description;
     p->minVal=minVal;
     p->maxVal=maxVal;
     p->defVal=p->value=defVal;
     p->dimensionality.MakeArray((int)dimensionality.size(),VT_R8);
     p->valStatus=valStatus;
     for (i=0;i<dimensionality.size();i++) p->dimensionality.SetDoubleAt(i,dimensionality[i]);
     return p;
    }

	//! Constructor.
    /*!
      Creates an parameter of which the name cannot be changed by external applications.
      Use CreateParameter instead of new
      \sa CreateParameter()
    */

	CRealParameter() : CAPEOPENBaseObject(false)
	{//everything is initialized via CreateParameter
	}

	//this object cannot be created using CoCreateInstance, so we do not need to put anything in the registry

	DECLARE_NO_REGISTRY()

	//COM map, including that of the CAPEOPENBaseObject

	BEGIN_COM_MAP(CRealParameter)
		COM_INTERFACE_ENTRY2(IDispatch, ICapeParameter)
		COM_INTERFACE_ENTRY(ICapeParameter)
		COM_INTERFACE_ENTRY(ICapeParameterSpec)
		COM_INTERFACE_ENTRY(ICapeRealParameterSpec)
		BASEMAP
	END_COM_MAP()


	DECLARE_PROTECT_FINAL_CONSTRUCT()

	// ICapeParameter Methods

	//! ICapeParameter::get_Specification
    /*!
      Return a reference to the object implementing the parameter spec. The parameter
      spec is implemented by this object, so we return a reference to ourselves
      \param spec [out, retval] receives the requested specification. Cannot be NULL.
    */

	STDMETHOD(get_Specification)(LPDISPATCH * spec)
	{	if (!spec) return E_POINTER; //not a valid pointer
	    //this object implements the parameter spec. Instead of below, we can also call QueryInterface on ourselves
	    *spec=(ICapeParameter*)this;
	    (*spec)->AddRef(); //caller must release
		return NOERROR;
	}

	//! ICapeParameter::get_value
    /*!
      Return the current value of the parameter
      \param value [out, retval] receives the value. Cannot be NULL.
    */

	STDMETHOD(get_value)(VARIANT * value)
	{	if (!value) return FALSE; //not a valid pointer
		VARIANT res;
		res.vt=VT_R8;
		res.dblVal=this->value;
		*value=res;
		return NOERROR;
	}

	//! ICapeParameter::put_value
    /*!
      Sets the current value of the parameter
      \param value [in] the value. Only accepted if of proper data type and within bounds
    */

	STDMETHOD(put_value)(VARIANT value)
	{	//convert to a number
	    VARIANT v;
	    v.vt=VT_EMPTY;
	    if (FAILED(VariantChangeType(&v,&value,0,VT_R8))) //this should not be required; simulation environments should pass a VT_R8 value to begin with
	     {SetError(L"Invalid data type. Expected a real number",L"ICapeParameter",L"put_value");
	      return ECapeUnknownHR;
	     }
	    ATLASSERT(v.vt==VT_R8);
	    //check missing
	    if (_isnan(v.dblVal))
	     {//CAPE-OPEN value for missing number
	      SetError(L"Invalid value: cannot be missing",L"ICapeParameter",L"put_value");
	      return ECapeUnknownHR;
	     }
	    //check if in range
	    if (!_isnan(minVal))
	     if (v.dblVal<minVal)
	      {SetError(L"Invalid value: below minimum value",L"ICapeParameter",L"put_value");
 	       return ECapeUnknownHR;
	      }
	    if (!_isnan(maxVal))
	     if (v.dblVal>maxVal)
	      {SetError(L"Invalid value: above maximum value",L"ICapeParameter",L"put_value");
 	       return ECapeUnknownHR;
	      }
	    //value is ok
	    this->value=v.dblVal;
	    *valStatus=CAPE_NOT_VALIDATED; //we changed the parameter, the unit needs to be re-validated
	    dirty=true; //something changed that affects saving
		return NOERROR;
	}

	//! ICapeParameter::get_ValStatus
    /*!
      Get the validation status. As the value is always valid, we return valid
      \param ValStatus [out, retval] receives the validation status. Cannot be NULL
    */

	STDMETHOD(get_ValStatus)(CapeValidationStatus * ValStatus)
	{	if (!ValStatus) return E_POINTER; //not a valid value
	    *ValStatus=CAPE_VALID; //value is always valid
		return NOERROR;
	}

	//! ICapeParameter::get_Mode
    /*!
      Get the mode. We only implement input parameters in this unit operation
      \param Mode [out, retval] receives the mode. Cannot be NULL
    */

	STDMETHOD(get_Mode)(CapeParamMode * Mode)
	{	if (!Mode) return E_POINTER; //not a valid value
	    *Mode=CAPE_INPUT; //this unit operation only has input parameters
		return NOERROR;
	}

	//! ICapeParameter::put_Mode
    /*!
      Set the mode. Not supported
      \param Mode [in] the mode
    */

	STDMETHOD(put_Mode)(CapeParamMode Mode)
	{	SetError(L"The mode of this parameter is read-only",L"ICapeParameter",L"put_Mode");
		return ECapeUnknownHR;
	}

	//! ICapeParameter::Validate
    /*!
      Validate the value of the parameter. As the value is always valid, we return OK.
      \param message [in, out] textual message in case of validation failure
      \param isOK [out, retval] validation result
    */

	STDMETHOD(Validate)(BSTR * message, VARIANT_BOOL * isOK)
	{	if ((!message)||(!isOK)) return E_POINTER; //not valid values
	    //note the [in, out] status of the message; if we were to put a new value in there, we should clear its existing value or its memory would leak
	    *isOK=VARIANT_TRUE; //we are always valid
		return NOERROR;
	}

	//! ICapeParameter::Reset
    /*!
      Resets the value of the parameter to its default value
    */

	STDMETHOD(Reset)()
	{	value=defVal;
	    *valStatus=CAPE_NOT_VALIDATED; //we changed the parameter, the unit needs to be re-validated
	    dirty=true; //something changed that affects saving
		return NOERROR;
	}

	// ICapeParameterSpec Methods

	//! ICapeParameterSpec::get_Type
    /*!
      Gets the type of this parameter. Always CAPE_REAL
      \param Type [out, retval] receives the type. Cannot be NULL.
    */

	STDMETHOD(get_Type)(CapeParamType * Type)
	{	if (!Type) return E_POINTER; //not a valid pointer
	    *Type=CAPE_REAL; 
		return NOERROR;
	}

	//! ICapeParameterSpec::get_Dimensionality
    /*!
      Gets the dimensionality of this parameter. Order of values is 
      m, kg, S, A, K, mole, cd, rad, optionally followed by a delta indicator.
      All trailing zeroes can be ommited; the returned data is initialized in 
      CreateParameter.
      \param dim [out, retval] receives the dimensionality. Cannot be NULL.
      \sa CreateParameter()
    */

	STDMETHOD(get_Dimensionality)(VARIANT * dim)
	{	if (!dim) return E_POINTER; //not a valid pointer
	    *dim=dimensionality.Copy(); //caller must free the result
		return NOERROR;
	}

	// ICapeRealParameterSpec Methods

	//! ICapeRealParameterSpec::get_DefaultValue
    /*!
      Gets the default value of this parameter
      \param DefaultValue [out, retval] receives the default value. Cannot be NULL.
    */

	STDMETHOD(get_DefaultValue)(double * DefaultValue)
	{	if (!DefaultValue) return E_POINTER; //not a valid pointer
	    *DefaultValue=defVal;
		return NOERROR;
	}

	//! ICapeRealParameterSpec::get_LowerBound
    /*!
      Gets the lower bound of this parameter. If not defined, will return NaN
      \param lBound [out, retval] receives the lower bound. Cannot be NULL.
      \sa get_UpperBound()
    */

	STDMETHOD(get_LowerBound)(double * lBound)
	{	if (!lBound) return E_POINTER; //not a valid pointer
	    *lBound=minVal;
		return NOERROR;
	}

	//! ICapeRealParameterSpec::get_UpperBound
    /*!
      Gets the upper bound of this parameter. If not defined, will return NaN
      \param uBound [out, retval] receives the upper bound. Cannot be NULL.
      \sa get_LowerBound()
    */

	STDMETHOD(get_UpperBound)(double * uBound)
	{	if (!uBound) return E_POINTER; //not a valid pointer
	    *uBound=maxVal;
		return NOERROR;
	}

	//! ICapeRealParameterSpec::Validate
    /*!
      Validate whether a given number is ok for this parameter.
      \param value [in] the value to check
      \param message [in, out] receives a textual error message of the reason for a value not being valid
      \param isOK [out, retval] receives the validation result
    */

	STDMETHOD(Validate)(double value, BSTR * message, VARIANT_BOOL * isOK)
	{	if ((!message)||(!isOK)) return E_POINTER; //invalid pointer 
	    //notice that message is [in,out]; if we set a value, we must clear the existing value or its memory will leak. Let's do that now
	    if (*message) 
	     {//it is not wise of the caller to pass a value in here... one cannot trust that all implementors do this properly
	      SysFreeString(*message);
	      *message=NULL;
	     }
	    //assume ok, unless not
	    *isOK=VARIANT_TRUE;
	    //check if NaN
	    if (_isnan(value))
	     {*message=SysAllocString(L"This parameter must have a value"); //caller must SysFreeString this value
	      *isOK=VARIANT_FALSE;
	     }
	    if (*isOK)
	     {//check min
	      if (!_isnan(minVal))
	       if (value<minVal)
	        {*message=SysAllocString(L"Value is below minimum value"); //caller must SysFreeString this value
	         *isOK=VARIANT_FALSE;
	        }
	     }
	    if (*isOK)
	     {//check max
	      if (!_isnan(maxVal))
	       if (value>maxVal)
	        {*message=SysAllocString(L"Value is above minimum value"); //caller must SysFreeString this value
	         *isOK=VARIANT_FALSE;
	        }
	     }
		return NOERROR;
	}

};

//convenience definition for variables of COM objects defined by this class
typedef CComObject<CRealParameter> RealParameterObject;

OBJECT_ENTRY_AUTO(__uuidof(RealParameter), CRealParameter)

