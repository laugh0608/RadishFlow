#pragma once
#include "Properties.h"

//forward declarations
class Compound; //forward declaration
class PropertyPackage; //forward declaration

//! PropertyPackage class
/*!
	This is the basic object that does the work. Its functionality corresponds to 
	a CAPE-OPEN property package. The calculations are performed using variables
	that are stored with each class. Although each property package can only be 
	called from a single thread (due to the ApartmentThreaded annotation of the 
	exported CAPE-OPEN classes), multiple property packages can exist and each 
	should store their own variables to be thread-safe. 
	
	From C++ this class can be accessed via the PropertyPack exported wrapper class
	
	From VB6, this class can be accessed via a number of exposed functions, via
	a property package handle.
	
	\sa PropertyPack
  
*/

class PropertyPackage
{public:

	//construction and destruction
	PropertyPackage();
	~PropertyPackage();
	    
    //functions
	const char *LastError();
	bool Load(const char *pathName);
	bool Save(const char *pathName);
	bool LoadFromPPFile(const char *ppName);
	
	//compounds and their properties
	bool GetCompoundCount(int *compoundCount);
	const char *GetCompoundStringConstant(int compIndex,StringConstant constID); //returns NULL in case of FAIL
	bool GetCompoundRealConstant(int compIndex,RealConstant constID,double &value); 
	bool GetTemperatureDependentProperty(int compIndex,TDependentProperty propID,double T,double &value); 
	
	//single phase mixture properties
	bool GetSinglePhaseProperties(int nComp,const int *compIndices,Phase phaseID,double T,double P,const double *X,int nProp,SinglePhaseProperty *propIDs,int *&valueCount,double **&values);

	//two-phase mixture properties
	bool GetTwoPhaseProperties(int nComp,const int *compIndices,Phase phaseID1,Phase phaseID2,double T1,double T2,double P1,double P2,const double *X1,const double *X2,int nProp,TwoPhaseProperty *propIDs,int *&valueCount,double **&values);
	
	//flash calculations
	bool Flash(int nComp,const int *compIndices,const double *X,FlashType type,FlashPhaseType phaseType,double spec1,double spec2,int &phaseCount,Phase *&phases,double *&phaseFractions,double **&phaseCompositions,double &T, double &P);
	
	//edit the package
	bool Edit();
	

private:

	//data members
	string lastError; /*!< the last error is stored as text */
    bool initialized; /*!< before first use, LoadFromPPFile or Load should be called */
    vector<Compound*> compounds; /*!< compounds in this property package */
    vector<double> values; /*!< internal buffer for return values */
    vector<double*> valuePointers; /*!< internal buffer for pointers to return values */
    vector<int> valueCounts; /*!< internal buffer for number of return values */
    vector<int> valueOffsets; /*!< internal buffer for offsets of return values */
    vector<int> flashCompounds; /*!< internal buffer storing compounds accounted for in flash */
    vector<int> flashCompoundMapping; /*!< internal buffer storing mapping of compounds in array passed to Flash()*/
    vector<double> flashComposition; /*!< internal buffer storing composition of compounds accounted for in flash*/
	bool vaporExists,liquidExists; /*!< phase existence during flash calc*/
	double vapFrac; /*!< molar vapor phase fraction during flash calc*/
	double liqFrac; /*!< molar liquid phase fraction during flash calc*/
	vector<double> vapX; /*!< molar vapor phase composition during flash calc*/
	vector<double> liqX; /*!< molar liquid phase composition during flash calc*/
	vector<Phase> existingPhases; /*!< internal buffer for returning existing phases after flash*/
	vector<double> Psat; /*!< storage of Psat during constant T flashes*/
	vector<double> Kminus1; /*!< storage of K-1 values during TP flashes*/
	FlashPhaseType flashPhaseType; /*!< storage of allowed phases specifier during flash*/
	double Hflash; /*!< storage of H during constant PH flashes*/
	double Sflash; /*!< storage of S during constant PS flashes*/
	double Pflash; /*!< storage of P during constant P flashes*/
	double Tflash; /*!< storage of T during constant T flashes*/
	double VFflash; /*!< storage of VF during constant VF flashes*/
	
	//editor can access private members:
	friend class PackageEditor;

	//generic helpers
	bool CheckTemperature(double T);
	bool CheckPressure(double P);
	bool CheckVaporPhaseFraction(double VF);
	bool CheckEnthalpy(double H);
	bool CheckEntropy(double S);

	//flash helpers
	double DewPointPressure();
	double BubblePointPressure();
	bool TPFlash(double T,double P);
	bool TVFFlash(double T,double VF,double &P);
	bool PVFFlash(double P,double VF,double &T);
	bool TVFmFlash(double T,double VF,double &P);
	bool PVFmFlash(double P,double VF,double &T);
	bool PHFlash(double P,double H,double &T);
	bool PSFlash(double P,double S,double &T);
	double MassVapFrac();
	
	//target routines for solving flashes
	friend bool TPFlashFunc(void *param,double X,double &F,string &error);
	friend bool TVFFlashFunc(void *param,double X,double &F,string &error);
	friend bool TsatFlashFunc(void *param,double X,double &F,string &error);
	friend bool TbubFlashFunc(void *param,double X,double &F,string &error);
	friend bool TdewFlashFunc(void *param,double X,double &F,string &error);
	friend bool PVFFlashFunc(void *param,double X,double &F,string &error);
	friend bool TVFmFlashFunc(void *param,double X,double &F,string &error);
	friend bool PVFmFlashFunc(void *param,double X,double &F,string &error);
	friend bool PHFlashFunc(void *param,double X,double &F,string &error);
	friend bool PSFlashFunc(void *param,double X,double &F,string &error);

public:

    //helpers for VB export routines
    BOOL GetPropertyResult(int index,int &count,double *&vals);
    BOOL GetFlashPhase(int index,Phase &phase,double &phaseFrac,int &Xcount,double *&X);
    BOOL GetFlashPhaseType(int index,Phase &phase);

};

