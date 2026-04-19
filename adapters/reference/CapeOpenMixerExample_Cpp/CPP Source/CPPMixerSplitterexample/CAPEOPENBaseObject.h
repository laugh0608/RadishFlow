#pragma once


//to be included in the COM map of derived objects:

#define BASEMAP \
 COM_INTERFACE_ENTRY(ECapeRoot) \
 COM_INTERFACE_ENTRY(ECapeUnknown) \
 COM_INTERFACE_ENTRY(ECapeUser) \
 COM_INTERFACE_ENTRY(ICapeIdentification)

//! Base class for any derived CAPE-OPEN object
/*!
  Base class for any derived CAPE-OPEN object. 
  Implements basic error handling and identification
  which every CAPE-OPEN object should implement.
  All CAPE-OPEN objects derive from this class
*/

class CAPEOPENBaseObject :
  public CComObjectRootEx<CComSingleThreadModel>,
  public IDispatchImpl<ECapeRoot, &__uuidof(ECapeRoot), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>,
  public IDispatchImpl<ECapeUnknown, &__uuidof(ECapeUnknown), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>,
  public IDispatchImpl<ECapeUser, &__uuidof(ECapeUser), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>,
  public IDispatchImpl<ICapeIdentification, &__uuidof(ICapeIdentification), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>
	
{
public:

	//! Constructor.
    /*!
      The base class needs to know whether the name and the description of the object 
      can be changed by external users. Optionally, a name and description initializer
      can be passed with the constructor. The name of an object should not be empty, so
      if no name is given at time of construction, one should provide a name immediately
      after construction.
      \param canRename should be true for classes of which the name and description may be changed by external applications.
      \param name the initial name, optional
      \param description the initial description, optional
    */

	CAPEOPENBaseObject(bool canRename, const OLECHAR *name=NULL, const OLECHAR *description=NULL)
	 {this->canRename=canRename; //determines whether external objects are allowed to rename this object
	  if (name) this->name=name; //if a name is not provided with the constructor, make sure to set after construction 
	  if (description) this->description=description;
	  dirty=false; //nothing has changed
	 }

	//! Destructor.
    /*!
      Destructors of base classes should be declared virtual so that 
      derived class destructors will be called in case the base class is destroyed instead
    */

	virtual ~CAPEOPENBaseObject()
	 {
	 }

	//member variables
	bool canRename;         /*!< can this object be renamed? */
	wstring name;           /*!< the name of this object */
	wstring description;	/*!< the description of this object */
	wstring errDesc;        /*!< the description of the last error, also used as the error name */
	wstring errIface;       /*!< the interface of the last error, e.g. ICapeUnit */
	wstring errScope;       /*!< the scope of the last error, e.g. Validate */
	bool dirty;             /*!< set if something has changed that needs to be saved */
	
	//! ECapeRoot::SetError
    /*!
      Sets description, interface and scope for last error. These values must be set before
      returning a CAPE-OPEN error code, and can later on be obtained by the caller of this
      object via the CAPE-OPEN error interfaces ECapeRoot, ECapeUnknown and ECapeUser
      \param desc description of the last error, cannot be empty or NULL
      \param iface interface of the last error, cannot be empty or NULL. E.g. ICapeUnitPort
      \param scope scope of the last error, cannot be empty or NULL. E.g Connect
      \sa get_name(), get_description(), get_scope(), get_interfaceName()
    */
    
    void SetError(const OLECHAR *desc,const OLECHAR *iface,const OLECHAR *scope)
    {//store the details of the error about to be thrown
     ATLASSERT(desc);
     ATLASSERT(*desc);
     ATLASSERT(iface);
     ATLASSERT(*iface);
     ATLASSERT(scope);
     ATLASSERT(*scope);
     errDesc=desc;
     errScope=scope;
     errIface=iface;
    }
	
	// ECapeRoot Methods

	//! ECapeRoot::get_name
    /*!
      Returns the name of the last error; this implementation returns the description of the last error as errors are not
      explicitly named; objects must call SetError before returning a CAPE-OPEN error code
      \param name [out, retval] receives the error name, cannot be NULL
      \sa SetError()
    */

	STDMETHOD(get_name)(BSTR * name)
	{	//return the name of the last error (we return its decription)
	 	if (!name) return E_POINTER; //invalid pointer
	    ATLASSERT(errDesc.size()!=0);
	    *name=SysAllocString(errDesc.c_str());
		return NOERROR;
	}

	// ECapeUser Methods

	//! ECapeUser::get_code
    /*!
      Returns the code of the last error; this implementation does not know about error codes, and always returns 0.
      \param code [out, retval] receives the error code, cannot be NULL
    */

	STDMETHOD(get_code)(long * code)
	{	//return the last error code
	 	if (!code) return E_POINTER; //invalid pointer
	    //we do not support error codes
	    *code=0;
		return NOERROR;
	}
	
	//! ECapeUser::get_description
    /*!
      Returns the description of the last error; objects must call SetError before returning a CAPE-OPEN error code
      \param description [out, retval] receives the error description, cannot be NULL
      \sa SetError()
    */

	STDMETHOD(get_description)(BSTR * description)
	{	//return the last error description
	    if (!description) return E_POINTER; //invalid pointer
	    ATLASSERT(errDesc.size()!=0);
	    *description=SysAllocString(errDesc.c_str());
		return NOERROR;
	}

	//! ECapeUser::get_scope
    /*!
      Returns the scope of the last error; objects must call SetError before returning a CAPE-OPEN error code
      \param scope [out, retval] receives the error scope, cannot be NULL
      \sa SetError()
    */

	STDMETHOD(get_scope)(BSTR * scope)
	{   //return the last error scope
	    if (!scope) return E_POINTER; //invalid pointer
	    ATLASSERT(errScope.size()!=0);
	    *scope=SysAllocString(errScope.c_str());
		return NOERROR;
	}

	//! ECapeUser::get_interfaceName
    /*!
      Returns the interface of the last error; objects must call SetError before returning a CAPE-OPEN error code
      \param interfaceName [out, retval] receives the error interface, cannot be NULL
      \sa SetError()
    */

	STDMETHOD(get_interfaceName)(BSTR * interfaceName)
	{   //return the last error interface
	    if (!interfaceName) return E_POINTER; //invalid pointer
	    ATLASSERT(errIface.size()!=0);
	    *interfaceName=SysAllocString(errIface.c_str());
		return NOERROR;
	}

	//! ECapeUser::get_operation
    /*!
      Returns the operation of the last error; not supported by this code, this function always returns N/A
      \param operation [out, retval] receives the error operation, cannot be NULL
    */

	STDMETHOD(get_operation)(BSTR * operation)
	{   //we do not support an error operation
	    if (!operation) return E_POINTER; //invalid pointer
	    *operation=SysAllocString(L"N/A");
		return NOERROR;
	}

	//! ECapeUser::get_moreInfo
    /*!
      Returns more information on the last error; this function always returns a reference to the CO-LaN web site
      \param moreInfo [out, retval] receives the error information, cannot be NULL
    */

	STDMETHOD(get_moreInfo)(BSTR * moreInfo)
	{	//perhaps CO-LaN wants to provide more info
	    if (!moreInfo) return E_POINTER; //invalid pointer
	    *moreInfo=SysAllocString(L"Please visit http://www.colan.org/ for more information");
		return NOERROR;
	}

	// ICapeIdentification Methods

	//! ICapeIdentification::get_ComponentName
    /*!
      Returns the name of this object. The name of items in a collection must be unique. The name 
      must not be empty. The name can be passed to the constructor, or must initialized after 
      construction of the object
      \param name [out, retval] receives this object's name, cannot be NULL
      \sa CAPEOPENBaseObject(), get_ComponentDescription(), put_ComponentName()
    */

	STDMETHOD(get_ComponentName)(BSTR * name)
	{	if (!name) return E_POINTER; //invalid pointer
		ATLASSERT(this->name.size()>0);
		*name=SysAllocString(this->name.c_str());
		return NO_ERROR;
	}

	//! ICapeIdentification::put_ComponentName
    /*!
      Sets the name of this object. The name of items in a collection must be unique. The name 
      must not be empty. Only allowed if the object can be renamed.
      \param name [IN] new object name, cannot be NULL or empty
      \sa CAPEOPENBaseObject(), get_ComponentName(), put_ComponentDescription()
    */
	
	STDMETHOD(put_ComponentName)(BSTR name)
	{	if (!canRename) 
	     {SetError(L"The name of this object is read-only",L"ICapeIdentification",L"put_ComponentName");
	      return ECapeUnknownHR;
	     }
	    if (!name)
	     {SetError(L"The name of this object cannot be empty",L"ICapeIdentification",L"put_ComponentName");
	      return ECapeUnknownHR;
	     }
	    if (!*name)
	     {SetError(L"The name of this object cannot be empty",L"ICapeIdentification",L"put_ComponentName");
	      return ECapeUnknownHR;
	     }
	    this->name=name;
	    dirty=true; //something changed that affects saving
		return NO_ERROR;
	}	

	//! ICapeIdentification::get_ComponentDescription
    /*!
      Returns the description of this object. The description can be passed to the constructor, 
      or can be initialized after construction of the object
      \param desc [out, retval] receives this object's description, cannot be NULL
      \sa CAPEOPENBaseObject(), get_ComponentName(), put_ComponentDescription()
    */
	
	STDMETHOD(get_ComponentDescription)(BSTR * desc)
	{   if (!desc) return E_POINTER; //invalid pointer
	    if (this->description.size()==0) *desc=NULL;
	    else *desc=SysAllocString(this->description.c_str());
		return NO_ERROR;
	}
	
	//! ICapeIdentification::put_ComponentDescription
    /*!
      Sets the description of this object. Only allowed if the object can be renamed.
      \param desc [IN] new object description, cannot be NULL or empty
      \sa CAPEOPENBaseObject(), get_ComponentDescription(), put_Name()
    */
	
	STDMETHOD(put_ComponentDescription)(BSTR desc)
	{	if (!canRename) 
	     {SetError(L"The description of this object is read-only",L"ICapeIdentification",L"put_ComponentDescription");
	      return ECapeUnknownHR;
	     }
	    if (!desc) this->description.clear();
	    else this->description=desc;
	    dirty=true; //something changed that affects saving
		return NO_ERROR;
	}

};
