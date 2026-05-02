// IdealThermo_CPP_TS10.cpp : Implementation of DLL Exports.


#include "stdafx.h"
#include "resource.h"
#include "IdealThermo_CPP_TS10.h"

/*! \mainpage CPP version 1.0 Ideal Thermo System 
*
*(C) CO-LaN 2011: <a href="http://www.colan.org/">http://www.colan.org/</a>
*Implemented by AmsterCHEM 2011: <a href="http://www.amsterchem.com/">http://www.amsterchem.com/</a>
*
*This project (IdealThermo_CPP_TS10) implements a CAPE-OPEN 
*version 1.0 Thermo System COM object. IdealThermoModule.dll
*is used for all property and equilibrium calculations.
*
*CO-LaN nor AmsterCHEM claim that this example is fit for purpose. 
*CO-LaN nor AmsterCHEM claim that this example implements
*the thermo system in an efficient way; in the contrary, 
*in order to keep the example readible, no attempt is made to 
*cache allocated variables (such as strings, arrays, ...) or to provide 
* efficient lookup methods (e.g. for items in collections, ...)
*
*This implementation is intended for illustrative purposes only. Use
*this example as you please. Under no circumstance can CO-LaN or 
*AmsterCHEM be held liable for consequential or any other damages
*resulting from this code.
*
*/


//! Module class
/*!

This class impements the ATL module, with basic functionality 
like class registration, unregistration, etc

*/

class CIdealThermo_CPP_TS10Module : public CAtlDllModuleT< CIdealThermo_CPP_TS10Module >
{
public :
	DECLARE_LIBID(LIBID_IdealThermo_CPP_TS10Lib)
	DECLARE_REGISTRY_APPID_RESOURCEID(IDR_IDEALTHERMO_CPP_TS10, "{22952EEB-F01C-421D-92DA-D1859475A159}")
};

CIdealThermo_CPP_TS10Module _AtlModule;


// DLL Entry Point
extern "C" BOOL WINAPI DllMain(HINSTANCE hInstance, DWORD dwReason, LPVOID lpReserved)
{
	hInstance;
    return _AtlModule.DllMain(dwReason, lpReserved); 
}


// Used to determine whether the DLL can be unloaded by OLE
STDAPI DllCanUnloadNow(void)
{
    return _AtlModule.DllCanUnloadNow();
}


// Returns a class factory to create an object of the requested type
STDAPI DllGetClassObject(REFCLSID rclsid, REFIID riid, LPVOID* ppv)
{
    return _AtlModule.DllGetClassObject(rclsid, riid, ppv);
}


// DllRegisterServer - Adds entries to the system registry
STDAPI DllRegisterServer(void)
{   // registers object, all interfaces in typelib
    //  we do not register the type lib, it is not needed and it requires admin rights
    //   (in VS2005, no longer in later ATL versions)
    HRESULT hr = _AtlModule.DllRegisterServer(FALSE);
	return hr;
}

// DllUnregisterServer - Removes entries from the system registry
STDAPI DllUnregisterServer(void)
{	// unregisters object, all interfaces in typelib
    //  we do not unregister the type lib, it is not needed and it requires admin rights
    //   (in VS2005, no longer in later ATL versions)
    HRESULT hr = _AtlModule.DllUnregisterServer(FALSE);
	return hr;
}

