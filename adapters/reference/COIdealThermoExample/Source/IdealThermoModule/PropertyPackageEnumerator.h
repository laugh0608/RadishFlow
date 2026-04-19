#pragma once
#include "IdealThermoModule.h"

//! PropertyPackageEnumerator class
/*!
	This object allows obtaining the available property packages on the system

	From C++ this class can be accessed via the PropertyPackEnumerator exported wrapper class
	
	\sa PropertyPackEnumerator
  
*/


class PropertyPackageEnumerator
{private:

 vector<string> PPnames;
 
 public:
 
//! Constructor
/*!
  Constructor, creates a PropertyPackageEnumerator class and lists the available property packages
*/
 
 PropertyPackageEnumerator()
 {string path;
  path=GetUserDataPath();
  ListFiles(path.c_str(),"propertypackage",PPnames);
 }
 
//! Count
/*!
  Get the number of available property packages 
  \return Number of available property packages 
*/

 int Count() {return (int)PPnames.size();}
 
//! PackageName
/*!
  Get the name of a property package
  \param index Index of the property package for which to return the name, must be between 0 and Count-1, inclusive (no error checks are made)
  \return Name of the property package
*/

 const char *PackageName(int index) {return PPnames[index].c_str();}
  
};

