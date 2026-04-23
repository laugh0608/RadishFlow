#pragma once

#ifdef IDEALTHERMOMODULE_EXPORTS
//for the IdealThermoModule.dll, the symbol IDEALTHERMOMODULE_EXPORTS is defined
#define IMPORTEXPORT __declspec( dllexport )
#else
// ... otherwise the header files that expose the exported objects will use:
#define IMPORTEXPORT __declspec( dllimport )
#endif

