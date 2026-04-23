#include "StdAfx.h"
#include "CPPExports.h"
#include "PropertyPackage.h"
#include "PropertyPackageEnumerator.h"
#include "ThermoSystemEditor.h"

//! Constructor
/*!
  Constructor, creates a PropertyPackageEnumerator class and lists the available property packages
*/
 
PropertyPackEnumerator::PropertyPackEnumerator()
 {ppEnum=new PropertyPackageEnumerator();
 }

//! Destructor
/*!
  Destructor, cleans up
*/
 
PropertyPackEnumerator::~PropertyPackEnumerator()
 {delete ppEnum;
 }
 
//! Count
/*!
  Get the number of available property packages 
  \return Number of available property packages 
*/

int PropertyPackEnumerator::Count() {return ppEnum->Count();}
 
//! PackageName
/*!
  Get the name of a property package
  \param index Index of the property package for which to return the name, must be between 0 and Count-1, inclusive (no error checks are made)
  \return Name of the property package
*/

const char *PropertyPackEnumerator::PackageName(int index) {return ppEnum->PackageName(index);}

//! Constructor
/*!
  Constructor, creates a PropertyPackage class
  \sa PropertyPackage
*/
 
PropertyPack::PropertyPack() 
  {pp=new PropertyPackage();
  }
 
//! Destructor
/*!
  Destructor, cleans up
  \sa PropertyPackage
*/
 
PropertyPack::~PropertyPack() 
  {delete pp;
  }


//! Return the last error
 /*!
  Returns the error message of the last function of the 
  PropertyPackage instance that returned a failure.
*/
 
const char *PropertyPack::LastError() {return pp->LastError();}

//! Load the PropertyPackage content from a file
/*!
  Load the configuration of the PropertyPackage from 
  a named file. Should be called only once, at
  the start of the life time of a PropertyPackage.
  The property package
  \param pathName Location of the data file to load from
  \return True for success, false for error
  \sa Save(), LoadFromPPFile(), LastError()
*/
 
bool PropertyPack::Load(const char *pathName) {return pp->Load(pathName);}
 
 //! Save the PropertyPackage content to a file
/*!
  Save the configuration of the PropertyPackage to
  a named file. 
  \param pathName Location of the data file to save to
  \return True for success, false for error
  \sa Save(), LastError()
*/

bool PropertyPack::Save(const char *pathName) {return pp->Save(pathName);}
 
//! Load the PropertyPackage content from a file
/*!
  Load the configuration of the PropertyPackage from 
  a configuration name from the default location and
  with default extension.
  \param ppName Name of the Property Package configuration to load
  \return True for success, false for error
  \sa Save(), Load(), LastError()
*/

bool PropertyPack::LoadFromPPFile(const char *ppName) {return pp->LoadFromPPFile(ppName);}
 

//! Get number of compounds
/*!
  Details of the compounds can be obtained by GetCompoundStringConstant and GetCompoundRealConstant
  \param compoundCount Receives the number of compounds
  \return true for success, false for error
  \sa GetCompoundStringConstant(), GetCompoundRealConstant(), LastError()
*/

bool PropertyPack::GetCompoundCount(int *compoundCount) {return pp->GetCompoundCount(compoundCount);}
 
//! Get compound string constant
/*!
  Get string constant for a compound.
  \param compIndex Index of the compound. Must be between zero and number of compounds-1, inclusive.
  \param constID ID of the string constant to be obtained
  \return String constant if ok, NULL in case of failure
  \sa GetCompoundCount(), GetCompoundRealConstant(), LastError(), StringConstant
*/
 
const char *PropertyPack::GetCompoundStringConstant(int compIndex,StringConstant constID) {return pp->GetCompoundStringConstant(compIndex,constID);}
 
//! Get compound real constant
/*!
  Get real constant for a compound.
  \param compIndex Index of the compound. Must be between zero and number of compounds-1, inclusive.
  \param constID ID of the real constant to be obtained
  \param value Receives the value of the requested compound constant
  \return True if ok
  \sa GetCompoundCount(), GetCompoundStringConstant(), LastError(), RealConstant
*/
 
bool PropertyPack::GetCompoundRealConstant(int compIndex,RealConstant constID,double &value) {return pp->GetCompoundRealConstant(compIndex,constID,value);}


//! Get compound temperature dependent property value at specified temperature
/*!
  Get real constant for a compound.
  \param compIndex Index of the compound. Must be between zero and number of compounds-1, inclusive.
  \param propID ID of the temperature dependent property to be obtained
  \param T Temperature [K]. Must be between zero and critical temperature of the compound
  \param value Receives the value of the requested temperature dependent property.
  \return True if ok
  \sa GetCompoundCount(), LastError(), TDependentProperty
*/

bool PropertyPack::GetTemperatureDependentProperty(int compIndex,TDependentProperty propID,double T,double &value) {return pp->GetTemperatureDependentProperty(compIndex,propID,T,value);}

//! Get single-phase mixture properties at specified temperature, pressure and composition
/*!
  Calculate and get single phase mixture properties. The properties are returned in arrays 
  that are allocated and stored by this DLL. The return values are only valid until the next
  call to GetSinglePhaseProperties or GetTwoPhaseProperties, so store the return values, but 
  not the pointers to them. Multiple properties can be requested in a single call. For each
  property, the values and number of values are returned
  
  \param nComp Number of compounds in the mixture
  \param compIndices Indices of the compounds in the mixture. One index for each compounds. Must be between 0 and number of compounds-1, inclusive
  \param phaseID ID of the phase for which to calculate the properties
  \param T Temperature [K]
  \param P Pressure [Pa]
  \param X Mole fractions [mol/mol], one value for each compound, assumed normalized
  \param nProp Number of properties requested
  \param propIDs IDs of the properties requested
  \param valueCount Receives the number of values for each of the properties, one value for each property
  \param values Receives the values, one double array for each property. Size of the array corresponds to valueCount for each property
  \return True if ok
  \sa GetCompoundCount(), LastError(), Phase, SinglePhaseProperty
*/

bool PropertyPack::GetSinglePhaseProperties(int nComp,const int *compIndices,Phase phaseID,double T,double P,const double *X,int nProp,SinglePhaseProperty *propIDs,int *&valueCount,double **&values) {return pp->GetSinglePhaseProperties(nComp,compIndices,phaseID,T,P,X,nProp,propIDs,valueCount,values);}
 
//! Get single-phase mixture properties at specified temperature, pressure and composition
/*!
  Calculate and get single phase mixture properties. The properties are returned in arrays 
  that are allocated and stored by this DLL. The return values are only valid until the next
  call to GetSinglePhaseProperties or GetTwoPhaseProperties, so store the return values, but 
  not the pointers to them. Multiple properties can be requested in a single call. For each
  property, the values and number of values are returned
  
  \param nComp Number of compounds in the mixture
  \param compIndices Indices of the compounds in the mixture. One index for each compounds. Must be between 0 and number of compounds-1, inclusive
  \param phaseID1 ID of the first phase for which to calculate the property
  \param phaseID2 ID of the second phase for which to calculate the property
  \param T1 Temperature of phase 1[K]
  \param T2 Temperature of phase 2[K]
  \param P1 Pressure of phase 1 [Pa]
  \param P2 Pressure of phase 2 [Pa]
  \param X1 Mole fractions for phase 1 [mol/mol], one value for each compound, assumed normalized
  \param X2 Mole fractions for phase 2 [mol/mol], one value for each compound, assumed normalized
  \param nProp Number of properties requested
  \param propIDs IDs of the properties requested
  \param valueCount Receives the number of values for each of the properties, one value for each property
  \param values Receives the values, one double array for each property. Size of the array corresponds to valueCount for each property
  \return True if ok
  \sa GetCompoundCount(), LastError(), Phase, TwoPhaseProperty
*/


bool PropertyPack::GetTwoPhaseProperties(int nComp,const int *compIndices,Phase phaseID1,Phase phaseID2,double T1,double T2,double P1,double P2,const double *X1,const double *X2,int nProp,TwoPhaseProperty *propIDs,int *&valueCount,double **&values) {return pp->GetTwoPhaseProperties(nComp,compIndices,phaseID1,phaseID2,T1,T2,P1,P2,X1,X2,nProp,propIDs,valueCount,values);}

//! Calculate phase equilibrium
/*!
  Calculate vapor liquid phase equilibrium. The vaues are returned in arrays that are allocated and stored by 
  this DLL. The return values are only valid until the next call to GetSinglePhaseProperties, 
  GetTwoPhaseProperties or Flash, so store the return values, but  not the pointers to them. 
  
  \param nComp Number of compounds in the mixture
  \param compIndices Indices of the compounds in the mixture. One index for each compounds. Must be between 0 and number of compounds-1, inclusive
  \param X Overall mole fractions[mol/mol], one value for each compound, assumed normalized
  \param type Type of specifications passed (e.g. TP for a temperature and pressure specification)
  \param spec1 Value of first specification (e.g. T/[K] for TP)
  \param spec2 Value of second specification (e.g. P/[Pa] for TP)
  \param phaseCount Receives the number of phases at equilibrium
  \param phases Receives the types of the existing phases (Vapor or Liquid)
  \param phaseFractions Receives the phase fractions of the existing phases [mol/mol]
  \param phaseCompositions Receives the compositions of the existing phases [mol/mol]; one array for each phase, each array contains one mole fraction for each compound
  \param T Receives the temperature at equilibrium
  \param P Receives the pressure at equilibrium
  \return True if ok
  \sa GetCompoundCount(), LastError(), Phase, FlashType
*/

bool PropertyPack::Flash(int nComp,const int *compIndices,const double *X,FlashType type,double spec1,double spec2,int &phaseCount,Phase *&phases,double *&phaseFractions,double **&phaseCompositions,double &T, double &P) {return pp->Flash(nComp,compIndices,X,type,VaporLiquid,spec1,spec2,phaseCount,phases,phaseFractions,phaseCompositions,T,P);}

//! Calculate phase equilibrium
/*!
  Calculate phase equilibrium. The vaues are returned in arrays that are allocated and stored by 
  this DLL. The return values are only valid until the next call to GetSinglePhaseProperties, 
  GetTwoPhaseProperties or Flash, so store the return values, but  not the pointers to them. 
  
  \param nComp Number of compounds in the mixture
  \param compIndices Indices of the compounds in the mixture. One index for each compounds. Must be between 0 and number of compounds-1, inclusive
  \param X Overall mole fractions[mol/mol], one value for each compound, assumed normalized
  \param type Type of specifications passed (e.g. TP for a temperature and pressure specification)
  \param phaseType Specified allowed phases in flash. 
  \param spec1 Value of first specification (e.g. T/[K] for TP)
  \param spec2 Value of second specification (e.g. P/[Pa] for TP)
  \param phaseCount Receives the number of phases at equilibrium
  \param phases Receives the types of the existing phases (Vapor or Liquid)
  \param phaseFractions Receives the phase fractions of the existing phases [mol/mol]
  \param phaseCompositions Receives the compositions of the existing phases [mol/mol]; one array for each phase, each array contains one mole fraction for each compound
  \param T Receives the temperature at equilibrium
  \param P Receives the pressure at equilibrium
  \return True if ok
  \sa GetCompoundCount(), LastError(), Phase, FlashType
*/

bool PropertyPack::Flash(int nComp,const int *compIndices,const double *X,FlashType type,FlashPhaseType phaseType,double spec1,double spec2,int &phaseCount,Phase *&phases,double *&phaseFractions,double **&phaseCompositions,double &T, double &P) {return pp->Flash(nComp,compIndices,X,type,phaseType,spec1,spec2,phaseCount,phases,phaseFractions,phaseCompositions,T,P);}

//! Edit the property package
/*!
  Edit the property package
  \return True if the changes are accepted, False in case the user cancels
*/

bool PropertyPack::Edit() {return pp->Edit();}

//! Edit routine for collection of Property Packages
/*!
  Show the edit dialog for the Property Packages available
  on this system (for the current user). 
  
  \sa ThermoSystemEditor
  
*/

void IMPORTEXPORT EditThermoSystem()
{ThermoSystemEditor editor;
 editor.Edit();
}



