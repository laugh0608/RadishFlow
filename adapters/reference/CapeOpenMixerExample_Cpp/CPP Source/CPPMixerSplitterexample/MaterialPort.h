// MaterialPort.h : Declaration of the CMaterialPort

#pragma once
#include "resource.h"       // main symbols

#include "CPPMixerSplitterexample.h"
#include "CAPEOPENBaseObject.h"
#include "Material.h"


//! Material port class
/*!
  CAPE-OPEN class that implements a material port.
  We have both feed and product ports. A port connects to 
  an external material object. This implementation supports
  material objects defined by CAPE-OPEN versions 1.0 and 1.1. 
  The GetMaterial() utility function returns a material object
  interface to the caller that uses the appropriate thermo
  material object version.
  
  This object implements ICapeUnitPort and derived from 
  CAPEOPENBaseObject for the identification and error
  common interfaces.
*/

class ATL_NO_VTABLE CMaterialPort :
	public CComCoClass<CMaterialPort, &CLSID_MaterialPort>,
	public IDispatchImpl<ICapeUnitPort, &__uuidof(ICapeUnitPort), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>,
	public CAPEOPENBaseObject
{
public:

	ICapeThermoMaterialObject *mat10; /*!< the material object connected to this port, if version 1.0 */
	ICapeThermoMaterial *mat11; /*!< the material object connected to this port, if version 1.1 */
	CapePortDirection direction; /*!< the direction of the port, CAPE_INLET or CAPE_OUTLET */

	//! Helper function for creating the material port 
    /*!
      Helper function for creating the material port as an exposable COM object. After calling CreateMaterialPort, the reference
      count is one; do not delete the returned object. Use Release() instead
      \param name name of the port
      \param description description of the port
      \param direction direction of the port
      \sa CMaterialPort()
    */

    static CComObject<CMaterialPort> *CreateMaterialPort(const OLECHAR *name,const OLECHAR *description,CapePortDirection direction)
    {CComObject<CMaterialPort> *p;
     CComObject<CMaterialPort>::CreateInstance(&p); //create the instance with zero references
     p->AddRef(); //now it has one reference, the caller must Release this object
     p->name=name;
     p->description=description;
     p->direction=direction;
     return p;
    }

	//! Constructor.
    /*!
      Creates an material of which the name cannot be changed by external applications.
      Use CreateMaterialPort instead of new
      \sa CreateMaterialPort()
    */

	CMaterialPort() : CAPEOPENBaseObject(false)
	{mat10=NULL;
	 mat11=NULL;
	}
	
	//! Destructor.
    /*!
      The port should already have been disconnected at Terminate; let's make sure
    */

    ~CMaterialPort()
    {if (mat10)
      {ATLASSERT(false); //should have been disconnected before
       mat10->Release();
      }
     if (mat11)
      {ATLASSERT(false); //should have been disconnected before
       mat11->Release();
      }
    }	

	//this object cannot be created using CoCreateInstance, so we do not need to put anything in the registry

	DECLARE_NO_REGISTRY()

	//COM map, including that of the CAPEOPENBaseObject

	BEGIN_COM_MAP(CMaterialPort)
		COM_INTERFACE_ENTRY2(IDispatch, ICapeUnitPort)
		COM_INTERFACE_ENTRY(ICapeUnitPort)
		BASEMAP
	END_COM_MAP()

	DECLARE_PROTECT_FINAL_CONSTRUCT()

	//! IsConnected
    /*!
      Utility function to check whether the port is connected.
      \return true if connected
      \sa Connect(), Disconnect()
    */

    bool IsConnected()
     {return ((mat10!=NULL)||(mat11!=NULL));
     }

	//! Get a MaterialObject class
    /*!
      Get an object to represent the connected material. To the caller it is transparent 
      whether the connected material is CAPE-OPEN version 1.0 or version 1.1 based.
      
      Should only be called if the port is connected (caller should verify).
      
      \return a MaterialObject class
      \sa MaterialObject, MaterialObject10, MaterialObject11
    */

    Material GetMaterial()
    {ATLASSERT(IsConnected()); //caller should verify that the port is connected before calling this function
     Material m;
     if (mat11) m.SetMaterial11(mat11);
     else m.SetMaterial10(mat10);
     return m;
    }	

	// ICapeUnitPort Methods

	//! ICapeUnitPort::get_portType
    /*!
      Return the type of this port. Port is a material port.
      \param portType [out, retval] receives the port type. Cannot be NULL.
    */

	STDMETHOD(get_portType)(CapePortType * portType)
	{	if (!portType) return E_POINTER; //not a valid pointer
	    *portType=CAPE_MATERIAL;
		return NOERROR;
	}

	//! ICapeUnitPort::get_direction
    /*!
      Return the direction of this port. 
      \param portDirection [out, retval] receives the port direction. Cannot be NULL.
      \sa CreateMaterialPort
    */

	STDMETHOD(get_direction)(CapePortDirection * portDirection)
	{	if (!portDirection) return E_POINTER; //not a valid pointer
	    *portDirection=direction;
		return NOERROR;
	}

	//! ICapeUnitPort::get_connectedObject
    /*!
      Return the object connected to this port
      \param connectedObject [out, retval] receives the object connected to this port, if any. Cannot be NULL.
    */

	STDMETHOD(get_connectedObject)(LPDISPATCH * connectedObject)
	{	if (!connectedObject) return E_POINTER; //not a valid pointer
	    if (mat10) 
	     {*connectedObject=mat10;
	      mat10->AddRef(); //caller must release
	     }
	    else if (mat11) 
	     {*connectedObject=mat11;
	      mat11->AddRef(); //caller must release
	     }
	    else *connectedObject=NULL; //not connected; 
	    //alternatively we could throw an error at this point
	    //else
	    // {SetError(L"Port is not connected",L"ICapeUnitPort",L"get_connectedObject");
		//  return ECapeUnknownHR;
	    // }
		return NOERROR;
	}

	//! ICapeUnitPort::Connect
    /*!
      Return the object connected to this port. The Connect method should check whether the object is an object
      of the type which is supported, and refuse the connection if not. This port accepts material objects 
      of CAPE-OPEN thermo 1.0 and 1.1 versions
      \param objectToConnect [in] object to connected to. Cannot be NULL.
      \sa Disconnect()
    */

	STDMETHOD(Connect)(LPDISPATCH objectToConnect)
	{	if (!objectToConnect) return E_POINTER; //not a valid pointer; use Disconnect instead
	    //disconnect whatever we have connected now
	    Disconnect();
	    //we prefer to use version 1.1 thermo, if available
	    if (SUCCEEDED(objectToConnect->QueryInterface(IID_ICapeThermoMaterial,(LPVOID*)&mat11))) return NOERROR;
	    //not available, so use version 1.0 thermo
	    if (SUCCEEDED(objectToConnect->QueryInterface(IID_ICapeThermoMaterialObject,(LPVOID*)&mat10))) return NOERROR;
	    //neither appears to be available, disallow the connection
	    SetError(L"Object is not a valid version 1.0 or version 1.1 CAPE-OPEN material object",L"ICapeUnitPort",L"Connect");
		return ECapeUnknownHR;
	}

	//! ICapeUnitPort::Disconnect
    /*!
      Release references to the connected object
      \sa Connect()
    */

	STDMETHOD(Disconnect)()
	{	if (mat10) 
	     {mat10->Release();
	      mat10=NULL;
	     }
	    if (mat11)
	     {mat11->Release();
	      mat11=NULL;
	     }
		return NOERROR;
	}

};

//convenience definition for variables of COM objects defined by this class
typedef CComObject<CMaterialPort> MaterialPortObject;

OBJECT_ENTRY_AUTO(__uuidof(MaterialPort), CMaterialPort)
