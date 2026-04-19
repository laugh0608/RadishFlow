#pragma once

//! LockObject class
/*!
	We have a singleton lock object to protect multi-threaded 
	access to global variables
*/

class LockObject
{
public:
   LockObject(void); 
   ~LockObject(void);
   void Lock();
   void Unlock();

};

extern LockObject theLock; /*!< singleton instanc of the LockObject class */