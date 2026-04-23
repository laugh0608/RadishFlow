#include "StdAfx.h"
#include "Lock.h"
#include "IdealThermoModule.h"
#include "PropertyPackage.h"
#include <Oleauto.h>
#include "ThermoSystemEditor.h"

/*!
   Calling convention for functions exported to VB; these functions also appear in the def file.
*/

#define VBEXPORT __declspec( dllexport ) __stdcall 

/*!VB6 does not operate well on booleans that have a value of 1 for true, instead -1 is used.
   This causes funny things like res=True and Not res=True as well if not done properly
   as Not is a bitwise operator
   \param expr True or False
   \returns VARIANT_TRUE or VARIANT_FALSE
*/

#define VBBOOL(expr) ((expr)?VARIANT_TRUE:VARIANT_FALSE)

//type defs
vector<PropertyPackage*> ppMap; /*!< mapping of handle (is index) to PropertyPackage */

//support functions

//! Convert C string to BSTR
/*!
  Convert a const char * to a BSTR; caller must release the BSTR value
  \param str String to convert
  \return BSTR value; must be released by caller
*/

BSTR BSTRFromString(const char *str)
{BSTR b;
 int sz=lstrlen(str);
 if (sz)
  {b=SysAllocStringLen(NULL,sz);
   if (b) MultiByteToWideChar(CP_ACP,0,str,-1,b,sz+1);
  }
 else b=NULL;
 return b;
}

//! Create a VARIANT from a string
/*!
  Convert a const char * to a VARIANT; caller must free the VARIANT value
  \param str String to convert
  \return VARIANT value; must be freed by caller
*/

VARIANT VariantFromString(const char *str)
{VARIANT res;
 res.vt=VT_BSTR;
 res.bstrVal=BSTRFromString(str);
 return res;
}

//! Create a VARIANT from a vector of strings
/*!
  Convert a vector of strings to a VARIANT; caller must free the VARIANT value
  \param v String vector to convert
  \return VARIANT value; must be freed by caller
*/

VARIANT VariantFromStringArray(vector<string> &v)
{VARIANT res;
 LONG i;
 SAFEARRAYBOUND ba;
 ba.cElements=(ULONG)v.size();
 ba.lLbound=0;
 res.parray=SafeArrayCreate(VT_BSTR,1,&ba);
 res.vt=VT_ARRAY|VT_BSTR;
 for (i=0;i<(LONG)v.size();i++)
  {BSTR b=BSTRFromString(v[i].c_str());
   SafeArrayPutElement(res.parray,&i,b);
   SysFreeString(b);
  }
 return res;
}

//! Create a VARIANT from an array of double values
/*!
  Convert a vector of double values to a VARIANT; caller must free the VARIANT value
  \param count Number of values in array
  \param vals Array to convert
  \return VARIANT value; must be freed by caller
*/

VARIANT VariantDoubleArray(int count,double *vals)
{VARIANT res;
 LONG i;
 SAFEARRAYBOUND ba;
 ba.cElements=count;
 ba.lLbound=0;
 res.parray=SafeArrayCreate(VT_R8,1,&ba);
 res.vt=VT_ARRAY|VT_R8;
 for (i=0;i<count;i++) SafeArrayPutElement(res.parray,&i,vals+i);
 return res;
}

//! Get a Property Package from a handle created with PPCreatePropertyPackage
/*!
  Get a PropertyPackage * from a handle
  \param handle Handle of the Property Package, returned from PPCreatePropertyPackage
  \return PropertyPackage pointer, or NULL in case of invalid handle
  \sa PPCreatePropertyPackage()
*/

PropertyPackage *GetPropertyPackage(int handle)
{PropertyPackage *res=NULL;
 theLock.Lock();
 if ((handle>=1)&&(handle<(int)ppMap.size())) res=ppMap[handle];
 theLock.Unlock();
 return res;
}

//! Enumerate property package configurations
/*!
  Enumerate property package configurations present on the system (for the current user)
  \param packages Receives a list of package names
  \sa PropertyPackageEnumerator
*/

void VBEXPORT GetPackages(VARIANT *packages)
{//get list of packages
 string path=GetUserDataPath();
 vector<string> PPnames;
 ListFiles(path.c_str(),"propertypackage",PPnames);
 VariantClear(packages);
 *packages=VariantFromStringArray(PPnames);
}

//! Edit a Property Package
/*!
  Show the edito dialog for a Property Package instance
*/

void VBEXPORT EditPackages()
{//edit thermo system
 ThermoSystemEditor editor;
 editor.Edit();
}

//! Create a Property Package
/*!
  Create a PropertyPackage, return a handle for further calls
  
  Must be matched to a call by PPDeletePropertyPackage
  
  \return handle to created PropertyPackage
  \sa PPDeletePropertyPackage()  
*/

int VBEXPORT PPCreatePropertyPackage()
{//create a property package, return handle 
 PropertyPackage *p=new PropertyPackage;
 theLock.Lock();
 //find an empty slot
 if (ppMap.size()<1) 
  {ppMap.resize(1);
   ppMap[0]=NULL; //this slot will remain unused, 0 is not a valid handle value
  }
 int handle;
 for (handle=1;handle<(int)ppMap.size();handle++) if (!ppMap[handle]) break; //found an empty spot
 if (handle==(int)ppMap.size()) ppMap.resize(handle+1); //grow map
 ppMap[handle]=p;
 theLock.Unlock();
 return handle;
}

//! Delete a Property Package
/*!
  Delete a PropertyPackages created by PPCreatePropertyPackage
  
  Must be matched to a call by PPDeletePropertyPackage
  
  \param handle handle to a PropertyPackage
  \sa PPCreatePropertyPackage()  
*/

void VBEXPORT PPDeletePropertyPackage(int handle)
{theLock.Lock();
 //remove map
 if ((handle>=1)&&(handle<(int)ppMap.size()))
  if (ppMap[handle]) 
   {delete ppMap[handle];
    ppMap[handle]=NULL;
   }
 theLock.Unlock();
}

//! Get error string
/*!
  Get an error string of the last operation that failed
  
  \param handle Handle to a PropertyPackage for which the operation failed
  \return Error string
*/

VARIANT VBEXPORT PPGetLastError(int handle)
{PropertyPackage *pp=GetPropertyPackage(handle); 
 const char *str;
 if (pp) str=pp->LastError();
 else str="Invalid property package handle";
 return VariantFromString(str);
}

//! Load a Property Package from file
/*!
  Load a Property Package from a specified file
  
  \param handle Handle to a PropertyPackage on which to operate
  \param path Path to load the content of the PropertyPackage from
  \return True in case of success
  \sa PPGetLastError()
*/

VARIANT_BOOL VBEXPORT PPLoad(int handle,LPCSTR path)
{PropertyPackage *pp=GetPropertyPackage(handle); 
 if (!pp) return VARIANT_FALSE;
 return VBBOOL(pp->Load(path));
}

//! Save a Property Package from file
/*!
  Save a Property Package to a specified file
  
  \param handle Handle to a PropertyPackage on which to operate
  \param path Path to save the content of the PropertyPackage to
  \return True in case of success
  \sa PPGetLastError()
*/

VARIANT_BOOL VBEXPORT PPSave(int handle,LPCSTR path)
{PropertyPackage *pp=GetPropertyPackage(handle); 
 if (!pp) return VARIANT_FALSE;
 return VBBOOL(pp->Save(path));
}

//! Load a named Property Package 
/*!
  Load a Property Package from a property package definition file, i.e. one
  of the names returned by GetPackages
  
  \param handle Handle to a PropertyPackage on which to operate
  \param ppName Name of the property package configuration on the system
  \return True in case of success
  \sa GetPackages(), PPGetLastError()
*/

VARIANT_BOOL VBEXPORT PPLoadFromPPFile(int handle,LPCSTR ppName)
{PropertyPackage *pp=GetPropertyPackage(handle); 
 if (!pp) return VARIANT_FALSE;
 return VBBOOL(pp->LoadFromPPFile(ppName));
}

//! Edit a PropertyPackage
/*!
  Show the Edit dialog for a Property Package  
  \param handle Handle to a PropertyPackage on which to operate
*/

void VBEXPORT PPEdit(int handle)
{PropertyPackage *pp=GetPropertyPackage(handle); 
 if (pp) pp->Edit();
}

//! Get the number of compounds
/*!
  Get the number of compounds in a PropertyPackage
  \param handle Handle to a PropertyPackage on which to operate
  \param count Receives the number of compounds
  \return True in case of success
  \sa PPGetCompoundStringConstant(), PPGetCompoundRealConstant(), PPGetLastError()
*/

VARIANT_BOOL VBEXPORT PPGetCompoundCount(int handle,int *count)
{PropertyPackage *pp=GetPropertyPackage(handle); 
 if (!pp) return VARIANT_FALSE;
 return VBBOOL(pp->GetCompoundCount(count));
}

//! Get compound string constant
/*!
  Get string constant for a compound.
  \param handle Handle to a PropertyPackage on which to operate
  \param compIndex Index of the compound. Must be between zero and number of compounds-1, inclusive.
  \param constID ID of the string constant to be obtained
  \return String constant if ok, Empty in case of failure
  \sa PPGetCompoundCount(), PPGetCompoundRealConstant(), PPGetLastError(), StringConstant
*/

VARIANT VBEXPORT PPGetCompoundStringConstant(int handle,int compIndex,int constID)
{PropertyPackage *pp=GetPropertyPackage(handle); 
 VARIANT res;
 res.vt=VT_EMPTY;
 if (pp) 
  {const char *str=pp->GetCompoundStringConstant(compIndex,(StringConstant)constID); 
   if (str) res=VariantFromString(str);
  }
 return res;
}

//! Get compound real constant
/*!
  Get real constant for a compound.
  \param handle Handle to a PropertyPackage on which to operate
  \param compIndex Index of the compound. Must be between zero and number of compounds-1, inclusive.
  \param constID ID of the real constant to be obtained
  \param value Receives the value of the requested compound constant
  \return True if ok
  \sa PPGetCompoundCount(), PPGetCompoundStringConstant(), PPGetLastError(), RealConstant
*/

VARIANT_BOOL VBEXPORT PPGetCompoundRealConstant(int handle,int compIndex,int constID,double *value)
{PropertyPackage *pp=GetPropertyPackage(handle); 
 if (!pp) return VARIANT_FALSE;
 return VBBOOL(pp->GetCompoundRealConstant(compIndex,(RealConstant)constID,*value));
}

//! Get compound temperature dependent property value at specified temperature
/*!
  Get real constant for a compound. The real constants follow from the temperature correlations.
  \param handle Handle to a PropertyPackage on which to operate
  \param compIndex Index of the compound. Must be between zero and number of compounds-1, inclusive.
  \param propID ID of the temperature dependent property to be obtained
  \param T Temperature [K]. Must be between zero and critical temperature of the compound
  \param value Receives the value of the requested temperature dependent property.
  \return True if ok
  \sa PPGetCompoundCount(), PPGetLastError(), TDependentProperty
*/

VARIANT_BOOL VBEXPORT PPGetTemperatureDependentProperty(int handle,int compIndex,int propID,double T,double *value)
{PropertyPackage *pp=GetPropertyPackage(handle); 
 if (!pp) return VARIANT_FALSE;
 return VBBOOL(pp->GetTemperatureDependentProperty(compIndex,(TDependentProperty)propID,T,*value));
}

//! Get result of single and two-phase property calculations
/*!
  Gets result of single phase and two phase property calculations wrapped in a VARIANT structure.
  \param handle Handle to a PropertyPackage on which to operate
  \param resultIndex Index of the result. Must be between 0 and the number of calculated properties - 1 
  \return True if ok
  \sa PPCalcSinglePhaseProps(), PPCalcTwoPhaseProps(), PPGetLastError()
*/

VARIANT VBEXPORT PPGetPropertyResult(int handle,int resultIndex)
{VARIANT res;
 res.vt=VT_EMPTY;
 PropertyPackage *pp=GetPropertyPackage(handle); 
 if (pp)
  {int count;
   double *vals;
   if (pp->GetPropertyResult(resultIndex,count,vals)) res=VariantDoubleArray(count,vals);
  }
 return res;
}

//! Calculate single-phase mixture properties at specified temperature, pressure and composition
/*!
  Calculate single phase mixture properties. Use PPGetPropertyResult() to get the 
  results of the calculation if this function succeeds
  \param handle Handle to a PropertyPackage on which to operate
  \param nComp Number of compounds in the mixture
  \param compIndices Indices of the compounds in the mixture. One index for each compounds. Must be between 0 and number of compounds-1, inclusive
  \param phaseID ID of the phase for which to calculate the properties
  \param T Temperature [K]
  \param P Pressure [Pa]
  \param X Mole fractions [mol/mol], one value for each compound, assumed normalized
  \param nProp Number of properties requested
  \param propIDs IDs of the properties requested
  \return True if ok
  \sa PPGetPropertyResult(), PPGetCompoundCount(), PPGetLastError(), Phase, SinglePhaseProperty
*/

VARIANT_BOOL VBEXPORT PPCalcSinglePhaseProps(int handle,int nComp,int *compIndices,int phaseID,double T,double P,const double *X,int nProp,int *propIDs)
{int i;
 PropertyPackage *pp=GetPropertyPackage(handle); 
 if (!pp) return VARIANT_FALSE;
 SinglePhaseProperty *props;
 props=new SinglePhaseProperty[nProp];
 for (i=0;i<nProp;i++) props[i]=(SinglePhaseProperty)propIDs[i];
 int *valueCount;
 double **values;
 VARIANT_BOOL res=VBBOOL(pp->GetSinglePhaseProperties(nComp,compIndices,(Phase)phaseID,T,P,X,nProp,props,valueCount,values));
 delete []props;
 return res;
}

//! Calculate two-phase mixture properties at specified temperatures, pressures and compositions
/*!
  Calculate two-phase mixture properties. Use PPGetPropertyResult() to get the 
  results of the calculation if this function succeeds
  \param handle Handle to a PropertyPackage on which to operate
  \param nComp Number of compounds in the mixture
  \param compIndices Indices of the compounds in the mixture. One index for each compounds. Must be between 0 and number of compounds-1, inclusive
  \param phaseID1 ID of the first phase of the phase pair for which to calculate the properties
  \param phaseID2 ID of the second phase of the phase pair for which to calculate the properties
  \param T1 Temperature of the first phase [K]
  \param T2 Temperature of the second phase [K]
  \param P1 Pressure of the first phase [Pa]
  \param P2 Pressure of the second phase [Pa]
  \param X1 Mole fractions [mol/mol] of the first phase, one value for each compound, assumed normalized
  \param X2 Mole fractions [mol/mol] of the second phase, one value for each compound, assumed normalized
  \param nProp Number of properties requested
  \param propIDs IDs of the properties requested
  \return True if ok
  \sa PPGetPropertyResult(), PPGetCompoundCount(), PPGetLastError(), Phase, TwoPhaseProperty
*/

VARIANT_BOOL VBEXPORT PPCalcTwoPhaseProps(int handle,int nComp,int *compIndices,int phaseID1,int phaseID2,double T1,double T2,double P1,double P2,const double *X1,const double *X2,int nProp,int *propIDs)
{int i;
 PropertyPackage *pp=GetPropertyPackage(handle); 
 if (!pp) return VARIANT_FALSE;
 TwoPhaseProperty *props;
 props=new TwoPhaseProperty[nProp];
 for (i=0;i<nProp;i++) props[i]=(TwoPhaseProperty)propIDs[i];
 int *valueCount;
 double **values;
 VARIANT_BOOL res=VBBOOL(pp->GetTwoPhaseProperties(nComp,compIndices,(Phase)phaseID1,(Phase)phaseID2,T1,T2,P1,P2,X1,X2,nProp,props,valueCount,values));
 delete []props;
 return res;
}

//! Get resulting phase of a flash calculation
/*!
  Gets result for a phase that exists in equilibrium of a flash calculation
  \param handle Handle to a PropertyPackage on which to operate
  \param index Index of the phase. Must be between 0 and the number resulting phases  - 1 (phaseCount as returned by Flash())
  \param phase Receives the phase identifier, Vapor or Liquid
  \param phaseFrac Receives the phase fraction, wrapped as a VARIANT
  \param phaseComposition Receives the phase composition, wrapped as a VARIANT
  \return True if ok
  \sa PPFlash(), PPGetLastError(), PPFlashPhase()
*/

VARIANT_BOOL VBEXPORT PPFlashPhaseResult(int handle,int index,int *phase,VARIANT *phaseFrac,VARIANT *phaseComposition)
{PropertyPackage *pp=GetPropertyPackage(handle); 
 if (!pp) return VARIANT_FALSE;
 Phase phaseType;
 double phaseFraction;
 int compositionCount;
 double *compositionValues;
 if (!pp->GetFlashPhase(index,phaseType,phaseFraction,compositionCount,compositionValues)) return VARIANT_FALSE;
 //convert outputs
 *phase=phaseType;
 VariantClear(phaseFrac);
 *phaseFrac=VariantDoubleArray(1,&phaseFraction);
 VariantClear(phaseComposition);
 *phaseComposition=VariantDoubleArray(compositionCount,compositionValues);
 return VARIANT_TRUE;
}

//! Get resulting phase type of a flash calculation
/*!
  Gets result for a phase type that exists in equilibrium of a flash calculation
  \param handle Handle to a PropertyPackage on which to operate
  \param index Index of the phase. Must be between 0 and the number resulting phases  - 1 (phaseCount as returned by Flash())
  \param phase Receives the phase identifier, Vapor or Liquid
  \return True if ok
  \sa PPFlash(), PPGetLastError(), PPFlashPhaseResult()
*/

VARIANT_BOOL VBEXPORT PPFlashPhase(int handle,int index,int *phase)
{PropertyPackage *pp=GetPropertyPackage(handle); 
 if (!pp) return VARIANT_FALSE;
 Phase phaseType;
 if (!pp->GetFlashPhaseType(index,phaseType)) return VARIANT_FALSE;
 *phase=phaseType;
 return VARIANT_TRUE;
}

//! Calculate phase equilibrium
/*!
  Calculate phase equilibrium. Number of phases, pressure and temperature are returned, the 
  details of each phase can be retrieved using PPFlashPhaseResult()
  \param handle Handle to a PropertyPackage on which to operate
  \param nComp Number of compounds in the mixture
  \param compIndices Indices of the compounds in the mixture. One index for each compounds. Must be between 0 and number of compounds-1, inclusive
  \param X Overall mole fractions[mol/mol], one value for each compound, assumed normalized
  \param flashType Type of specifications passed (e.g. TP for a temperature and pressure specification)
  \param phaseType Specified allowed phases in flash. 
  \param spec1 Value of first specification (e.g. T/[K] for TP)
  \param spec2 Value of second specification (e.g. P/[Pa] for TP)
  \param phaseCount Receives the number of phases at equilibrium
  \param T Receives the temperature at equilibrium
  \param P Receives the pressure at equilibrium
  \return True if ok
  \sa PPFlashPhaseResult(), PPGetCompoundCount(), PPGetLastError(), Phase, FlashType, FlashPhaseType
*/


VARIANT_BOOL VBEXPORT PPFlash(int handle,int nComp,const int *compIndices,const double *X,int flashType,int phaseType,double spec1,double spec2,int *phaseCount,double *T,double *P)
{PropertyPackage *pp=GetPropertyPackage(handle); 
 if (!pp) return VARIANT_FALSE;
 //return values are ignored, obtain with PPFlashPhaseResult
 Phase *phases;
 double *phaseFractions;
 double **phaseCompositions;
 if (!pp->Flash(nComp,compIndices,X,(FlashType)flashType,(FlashPhaseType)phaseType,spec1,spec2,*phaseCount,phases,phaseFractions,phaseCompositions,*T,*P)) return VARIANT_FALSE;
 return VARIANT_TRUE;
}
