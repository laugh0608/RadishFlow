#include "stdafx.h"
#include "Lock.h"

CRITICAL_SECTION TheCriticalSection; /*!< CRITICAL_SECTION object to perform locking */

//! Constructor.
/*!
  Called upon construction of the LockObject class; initializes the CRITICAL_SECTION object
*/

LockObject::LockObject()
 {InitializeCriticalSection(&TheCriticalSection);
 }

//! Destructor.
/*!
  Called upon destruction of the LockObject class; deletes the CRITICAL_SECTION object
*/
 
LockObject::~LockObject()
 {DeleteCriticalSection(&TheCriticalSection);
 }
 
//! Lock.
/*!
  Call to protect access to global variables. Make sure that each Lock() matches an Unlock()
  \sa Unlock()
*/
 
void LockObject::Lock()
{EnterCriticalSection(&TheCriticalSection);
}

//! Unlock.
/*!
  Match each call to Lock() with Unlock()
  \sa Lock()
*/

void LockObject::Unlock()
{LeaveCriticalSection(&TheCriticalSection);
}

LockObject theLock; /*!< singleton instanc of the LockObject class */