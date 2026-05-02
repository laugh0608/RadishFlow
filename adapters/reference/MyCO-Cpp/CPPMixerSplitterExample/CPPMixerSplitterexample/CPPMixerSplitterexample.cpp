// CPPMixerSplitterexample.cpp : Implementation of DLL Exports.


#include "stdafx.h"
#include "resource.h"
#include "CPPMixerSplitterexample.h"

//! COM Module object
/*!
  COM module object as defined by ATL. Contains the basic implementation of the COM server. See ATL documentation for more information.
*/

class CCPPMixerSplitterexampleModule : public CAtlDllModuleT< CCPPMixerSplitterexampleModule >
{
public :

	//! The GUID of the COM server DLL

	DECLARE_LIBID(LIBID_CPPMixerSplitterexampleLib)

	//! Registration file entries, as in CPPMixerSplitterexample.rgs

	DECLARE_REGISTRY_APPID_RESOURCEID(IDR_CPPMIXERSPLITTEREXAMPLE, "{D5F7818B-EB4C-4F0A-A6D7-38105E99D79C}") 

};

CCPPMixerSplitterexampleModule _AtlModule; /*!< The one and only instance of CCPPMixerSplitterexampleModule */


//! DLL Entry Point
/*!
  This function is called when the DLL is loaded, when the DLL is unloaded, when threads attach and when threads detach. See Windows platform documentation. The default ATL implementation is used.
  \param hInstance instance handle
  \param dwReason reason for calling this function (e.g. thread is detaching)
  \param lpReserved not used
*/

extern "C" BOOL WINAPI DllMain(HINSTANCE hInstance, DWORD dwReason, LPVOID lpReserved)
{
	hInstance;
    return _AtlModule.DllMain(dwReason, lpReserved); 
}


//! COM function to check whether COM objects are still active
/*!
  This function is called to determine whether the DLL can be unloaded, which is not the case if COM objects are still active. The default ATL implementation is used.
*/

STDAPI DllCanUnloadNow()
{
    return _AtlModule.DllCanUnloadNow();
}

//! Returns a class factory to create an object of the requested type
/*!
  This function returns a class factory to create an object of the requested type. The default ATL implementation is used.
  \param rclsid the GUID of the object for which a class factory is required
  \param riid the ID of the interface to the class factory that should be returned
  \param ppv receives the class factory pointer
*/

STDAPI DllGetClassObject(REFCLSID rclsid, REFIID riid, LPVOID* ppv)
{   return _AtlModule.DllGetClassObject(rclsid, riid, ppv);
}


//! Adds entries to the system registry
/*!
  This function is called in response to a COM registration request. For example, by regsvr32.exe, or by installation tools. The default
  ATL implemenation is used. This uses registry entries from rgs files that are linked as resources and mapped to the corresponding classes
  using the DECLARE_REGISTRY_RESOURCEID macro
  \sa DllUnregisterServer
*/

STDAPI DllRegisterServer()
{   // registers object, typelib and all interfaces in typelib
    /// remark: even though the classes are registered in the user part of the register, 
    //  the type library is still registered in the All Users part. Later version of 
    //  ATL support AtlSetPerUserRegistration(true); here, if we want to prevent 
    //  the requirement for admin rights, we should pass FALSE to below function, and 
    //  register the type lib ourselves.
    // we do not require any type lib to be registered, hence we pass FALSE
    HRESULT hr = _AtlModule.DllRegisterServer(FALSE);
	return hr;
}

//! Removes entries from the system registry
/*!
  This function is called in response to a COM unregistration request. For example, by regsvr32.exe /u, or by uninstallation tools. The default
  ATL implemenation is used. This uses registry entries from rgs files that are linked as resources and mapped to the corresponding classes
  using the DECLARE_REGISTRY_RESOURCEID macro
  \sa DllRegisterServer
*/

STDAPI DllUnregisterServer()
{   // we do not require any type lib to be registered, hence we pass FALSE
	HRESULT hr = _AtlModule.DllUnregisterServer(FALSE);
	return hr;
}

