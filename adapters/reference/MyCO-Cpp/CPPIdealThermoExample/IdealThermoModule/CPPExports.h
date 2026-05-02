#pragma once
#include "Properties.h"
#include "ImportExport.h"

//forward declarations
class PropertyPackageEnumerator;
class PropertyPackage;

//! PropertyPackEnumerator class
/*!
  This class exposes the available property packages in such 
  manner that is ok to expose from the DLL. External C++ client
  can use this class.
  
  \sa PropertyPackageEnumerator
  
*/

class IMPORTEXPORT PropertyPackEnumerator
{private:
 PropertyPackageEnumerator *ppEnum;
 public:
 PropertyPackEnumerator();
 ~PropertyPackEnumerator();
 int Count();
 const char *PackageName(int index);
};


//! PropertyPack class
/*!
  This is a wrapper class that access the PropertyPackage in a
  manner that is ok to expose from the DLL. External C++ client
  can use this class.
  
  \sa PropertyPackage
  
*/

class IMPORTEXPORT PropertyPack
{//a wrapper version of PropertyPackage with exported class definition
 private:
 PropertyPackage *pp; /*!< the actual property package */
 public:
 PropertyPack();
 ~PropertyPack();
 const char *LastError();
 bool Load(const char *pathName);
 bool Save(const char *pathName);
 bool LoadFromPPFile(const char *ppName);
 bool GetCompoundCount(int *compoundCount);
 const char *GetCompoundStringConstant(int compIndex,StringConstant constID);
 bool GetCompoundRealConstant(int compIndex,RealConstant constID,double &value);
 bool GetTemperatureDependentProperty(int compIndex,TDependentProperty propID,double T,double &value);
 bool GetSinglePhaseProperties(int nComp,const int *compIndices,Phase phaseID,double T,double P,const double *X,int nProp,SinglePhaseProperty *propIDs,int *&valueCount,double **&values);
 bool GetTwoPhaseProperties(int nComp,const int *compIndices,Phase phaseID1,Phase phaseID2,double T1,double T2,double P1,double P2,const double *X1,const double *X2,int nProp,TwoPhaseProperty *propIDs,int *&valueCount,double **&values);
 bool Flash(int nComp,const int *compIndices,const double *X,FlashType type,double spec1,double spec2,int &phaseCount,Phase *&phases,double *&phaseFractions,double **&phaseCompositions,double &T, double &P);
 bool Flash(int nComp,const int *compIndices,const double *X,FlashType type,FlashPhaseType phaseType,double spec1,double spec2,int &phaseCount,Phase *&phases,double *&phaseFractions,double **&phaseCompositions,double &T, double &P);
 bool Edit();
};

void IMPORTEXPORT EditThermoSystem();

