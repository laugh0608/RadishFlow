#include <COBIA.h>
// Ensures the entry points are created
#define COBIA_PMC_ENTRY_POINTS
// Eensures the a default DllMain entry point is created
#define COBIA_PMC_DEFAULT_DLLMAIN
// PMC entry points are defined here:
#include <COBIA_PMC.h>

bool isPMCRegistrationForAllUsers() {
	return false;
}
