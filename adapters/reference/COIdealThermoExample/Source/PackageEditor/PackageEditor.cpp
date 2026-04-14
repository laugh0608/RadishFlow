#include <Windows.h>
#include <CPPExports.h>     // exports from the IdealThermoModule.dll

//! Entry point
/*!
  Entry point for application. All we do is call the package editor in IdealThermoModule.dll
  \param hInstance Instance handle
  \param hPrevInstance Handle of any previous instance, if any
  \param lpCmdLine Command line
  \param nCmdShow Show command
*/

int _stdcall WinMain(HINSTANCE hInstance,HINSTANCE hPrevInstance,LPSTR lpCmdLine,int nCmdShow)
{EditThermoSystem(); //yep - that's all
 return 0;
}

/*! \mainpage Package Editor
*
*(C) CO-LaN 2011: <a href="http://www.colan.org/">http://www.colan.org/</a>
*Implemented by AmsterCHEM 2011: <a href="http://www.amsterchem.com/">http://www.amsterchem.com/</a>
*
*This project (PackageEditor) implements a utility 
*to edit the collection of existing property packages.
*
*Editing the collection of property package can be done 
*by invoking Edit on the property package manager or
*thermo system as well, but not all CAPE-OPEN PMEs support
*such an action. Therefore, an application that does this
*is recommended.
*
*All it does is make a single call into the IdealThermoModule DLL.
*
*This implementation is intended for illustrative purposes only. Use
*this example as you please. Under no circumstance can CO-LaN or 
*AmsterCHEM be held liable for consequential or any other damages
*resulting from this code.
*
*/
