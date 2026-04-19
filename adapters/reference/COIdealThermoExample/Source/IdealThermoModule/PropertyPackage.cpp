#include "StdAfx.h"
#include "PropertyPackage.h"
#include "Compound.h"
#include "IdealThermoModule.h"
#include <float.h>
#include "Solver1Dim.h"
#include "PackageEditor.h"

//! VECPTR macro
/*!
  Cast a vector to a pointer
  \param vec vector for which to obtain the pointer
  \return Const pointer to element type of vector
*/
  
#define VECPTR(vec) &((vec)[0])


//! Constructor
/*!
  Called upon construction of a PropertyPackage instance
*/

PropertyPackage::PropertyPackage()
{initialized=false; //methods can only be used after Load or LoadFromPPFile is successfully called
 lastError="No error"; //set value to error in case an error has occured
}

//! Destructor
/*!
  Called upon destruction of a PropertyPackage instance
*/

PropertyPackage::~PropertyPackage()
{//clean up compounds
 int i;
 for (i=0;i<(int)compounds.size();i++) delete compounds[i]; 
}

//! Return the last error
/*!
  Returns the error message of the last function of the 
  PropertyPackage instance that returned a failure.
*/

const char *PropertyPackage::LastError() 
{//return the last error
 return lastError.c_str();
}

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

bool PropertyPackage::Load(const char *pathName)
{//load content of the PP
 if (initialized)
  {lastError="Load can only be called once";
   return false;
  }
 FILE *f;
 int errCode;
 errCode=fopen_s(&f,pathName,"rb");
 if (errCode)
  {lastError="Failed to open \"";
   lastError+=pathName;
   lastError+="\": ";
   lastError+=ErrorString(errCode);
   return false;
  }
 //read the compounds
 string compName;
 while (ReadLine(f,compName))
  {//load the compound
   Compound *c=new Compound;
   if (!c->Load(compName.c_str(),lastError)) 
    {delete c;
     fclose(f);
     return false; //error is already set
    }
   compounds.push_back(c);
  }
 fclose(f);
 //we must have at least one compound
 if (compounds.size()==0)
  {lastError="Property package must contain at least one compound";
   return false;
  }
 //compounds must be unique
 int i,j;
 for (i=0;i<(int)compounds.size();i++)
  for (j=i+1;j<(int)compounds.size();j++)
   if (lstrcmpi(compounds[i]->name.c_str(),compounds[j]->name.c_str())==0)
    {lastError="Compound \"";
     lastError+=compounds[i]->name;
     lastError+="\" is present in property package more than once; compounds must be unique";
     return false;
    }
 //all ok
 initialized=true;
 return true; 
}

//! Save the PropertyPackage content to a file
/*!
  Save the configuration of the PropertyPackage to
  a named file. 
  \param pathName Location of the data file to save to
  \return True for success, false for error
  \sa Save(), LastError()
*/

bool PropertyPackage::Save(const char *pathName)
{//save content of the PP
 if (!initialized)
  {lastError="Property package has not been initialized";
   return false;
  }
 FILE *f;
 int errCode;
 errCode=fopen_s(&f,pathName,"wb");
 if (errCode)
  {lastError="Failed to open \"";
   lastError+=pathName;
   lastError+="\": ";
   lastError+=ErrorString(errCode);
   return false;
  }
 //store compound names, one per line
 int i;
 for (i=0;i<(int)compounds.size();i++)
  fprintf_s(f,"%s\n",compounds[i]->name.c_str());
 fclose(f);
 return true;
}

//! Load the PropertyPackage content from a file
/*!
  Load the configuration of the PropertyPackage from 
  a configuration name from the default location and
  with default extension.
  \param ppName Name of the Property Package configuration to load
  \return True for success, false for error
  \sa Save(), Load(), LastError()
*/

bool PropertyPackage::LoadFromPPFile(const char *ppName)
{//call load, with the default PP folder
 string path;
 path=GetUserDataPath();
 path+='\\';
 path+=ppName;
 path+=".propertypackage";
 return Load(path.c_str());
}

//! Get number of compounds
/*!
  Details of the compounds can be obtained by GetCompoundStringConstant and GetCompoundRealConstant
  \param compoundCount returns number of compounds
  \return true for success, false for error
  \sa GetCompoundStringConstant(), GetCompoundRealConstant(), LastError()
*/

bool PropertyPackage::GetCompoundCount(int *compoundCount)
{if (!initialized)
  {lastError="Property package has not been initialized";
   return false;
  }
 *compoundCount=(int)compounds.size();
 return true;
}

//! Get compound string constant
/*!
  Get string constant for a compound.
  \param compIndex Index of the compound. Must be between zero and number of compounds-1, inclusive.
  \param constID ID of the string constant to be obtained
  \return String constant if ok, NULL in case of failure
  \sa GetCompoundCount(), GetCompoundRealConstant(), LastError(), StringConstant
*/

const char *PropertyPackage::GetCompoundStringConstant(int compIndex,StringConstant constID)
{if (!initialized)
  {lastError="Property package has not been initialized";
   return NULL;
  }
 if ((compIndex<0)||(compIndex>=(int)compounds.size()))
  {lastError="Compound index out of range";
   return NULL;
  }
 switch (constID)
  {case Name:
        return compounds[compIndex]->name.c_str();
   case CASNumber:
        return compounds[compIndex]->CAS.c_str();
   case ChemicalFormula:
        return compounds[compIndex]->formula.c_str();
  }
 //not found
 lastError="Invalid constant ID";
 return NULL;
}

//! Get compound real constant
/*!
  Get real constant for a compound.
  \param compIndex Index of the compound. Must be between zero and number of compounds-1, inclusive.
  \param constID ID of the real constant to be obtained
  \param value Receives the value of the requested compound constant
  \return True if ok
  \sa GetCompoundCount(), GetCompoundStringConstant(), LastError(), RealConstant
*/


bool PropertyPackage::GetCompoundRealConstant(int compIndex,RealConstant constID,double &value)
{if (!initialized)
  {lastError="Property package has not been initialized";
   return false;
  }
 if ((compIndex<0)||(compIndex>=(int)compounds.size()))
  {lastError="Compound index out of range";
   return false;
  }
 switch (constID)
  {case NormalBoilingPoint:
        value=compounds[compIndex]->NBP;
		break;   
   case MolecularWeight:
        value=compounds[compIndex]->MW;
		break;   
   case CriticalTemperature:
        value=compounds[compIndex]->TC;
		break;   
   case CriticalPressure:
        value=compounds[compIndex]->PC;
		break;   
   case CriticalVolume:
        value=compounds[compIndex]->VC;
		break;   
   case CriticalDensity:
        value=1.0/compounds[compIndex]->VC;
		break;   
   default:
        lastError="Invalid constant ID";
        return false;
  }
 return true; 
}

//! Get compound temperature dependent property value at specified temperature
/*!
  Get real constant for a compound. The real constants follow from the temperature correlations.
  \param compIndex Index of the compound. Must be between zero and number of compounds-1, inclusive.
  \param propID ID of the temperature dependent property to be obtained
  \param T Temperature [K]. Must be between zero and critical temperature of the compound
  \param value Receives the value of the requested temperature dependent property.
  \return True if ok
  \sa GetCompoundCount(), LastError(), TDependentProperty
*/


bool PropertyPackage::GetTemperatureDependentProperty(int compIndex,TDependentProperty propID,double T,double &value)
{if (!initialized)
  {lastError="Property package has not been initialized";
   return false;
  }
 if ((compIndex<0)||(compIndex>=(int)compounds.size()))
  {lastError="Compound index out of range";
   return false;
  }
 if (!CheckTemperature(T)) return false;
 if (T>compounds[compIndex]->TC)
  {lastError="Temperature exceeds critical temperature";
   return false;
  }
 switch (propID)
  {case HeatOfVaporization:
        value=compounds[compIndex]->HvapCorrelation->Value(T);
        break;
   case HeatOfVaporizationDT:
        value=compounds[compIndex]->HvapCorrelation->ValueDT(T);
        break;
   case IdealGasHeatCapacity:
        value=compounds[compIndex]->CpCorrelation->Value(T);
        break;
   case IdealGasHeatCapacityDT:
        value=compounds[compIndex]->CpCorrelation->ValueDT(T);
        break;
   case VaporPressure:
        value=compounds[compIndex]->pSatCorrelation->Value(T);
        break;
   case VaporPressureDT:
        value=compounds[compIndex]->pSatCorrelation->ValueDT(T);
        break;
   case LiquidDensity:
        value=compounds[compIndex]->liqDensCorrelation->Value(T);
        break;
   case LiquidDensityDT:
        value=compounds[compIndex]->liqDensCorrelation->ValueDT(T);
        break;
   default:
        lastError="Invalid property ID";
        return false;
  }
 return true; 
}

//! Get single-phase mixture properties at specified temperature, pressure and composition
/*!
  Calculate and get single phase mixture properties. The properties are returned in arrays 
  that are allocated and stored by this DLL. The return values are only valid until the next
  call to GetSinglePhaseProperties, GetTwoPhaseProperties or Flash, so store the return values, but 
  not the pointers to them. Multiple properties can be requested in a single call. For each
  property, the values and number of values are returned.
  
  For the vapor phase, volume and density follow from the ideal gas law. Enthalpy follows from
  
  h = sum (Xi * int(CPi(T),T=Tref..T))
  
  Entropy follows from
  
  s = sum (Xi * (int(CPi(T)/T,T=Tref..T) - R ln (P/Pref) - R ln Xi)
  
  Fugacity coefficient is unity, log fugacity coefficient is zero, fugacity equals X * P.
  
  For the liquid phase, an ideal activity model is used. Activity coefficient is unity,
  therefore activity equals X. Fugacity then equals X * Psat, as Poynting corrections are ignored.
  
  The liquid phase volume is ideal:
  
  v = sum ( Xi Vi) = sum (Xi / rhoi)
  
  and density follows from 1 / v.
  
  Enthalpy follows from
  
  h = sum (Xi * int(CPi(T),T=Tref..T) - hvap,i)
  
  Entropy follows from 
  
  s = sum (Xi * (int(CPi(T)/T,T=Tref..T) - R ln (Psat,i/Pref) - R ln Xi - hvap,i/T)
  
  Due to the nature of the thermodynamics, the mixture properties should not be used for T 
  larger than or equal to the critical temperature of any present compound.
  
  \param nComp Number of compounds in the mixture
  \param compIndices Indices of the compounds in the mixture. One index for each compounds. Must be between 0 and number of compounds-1, inclusive
  \param phaseID ID of the phase for which to calculate the properties
  \param T Temperature [K]
  \param P Pressure [Pa]
  \param X Mole fractions [mol/mol], one value for each compound, assumed normalized
  \param nProp Number of properties requested
  \param propIDs IDs of the properties requested
  \param ValueCount Receives the number of values for each of the properties, one value for each property
  \param Values Receives the values, one double array for each property. Size of the array corresponds to valueCount for each property
  \return True if ok
  \sa GetCompoundCount(), LastError(), Phase, SinglePhaseProperty
*/

bool PropertyPackage::GetSinglePhaseProperties(int nComp,const int *compIndices,Phase phaseID,double T,double P,const double *X,int nProp,SinglePhaseProperty *propIDs,int *&ValueCount,double **&Values)
{//this implementation is for instructive purposes only; a production implementation would use
 // stored values for combined property evaluations, e.g. evaluate PSat only once for all 
 // requested properties for which Psat is required. THIS ROUTINE DOES NOT TAKE ADVANTAGE OF 
 // SIMULTANEOUS PROPERTY CALCULATIONS!!!
 int i,j,k,index;
 if (!initialized)
  {lastError="Property package has not been initialized";
   return false;
  }
 //check the inputs
 if ((phaseID!=Vapor)&&(phaseID!=Liquid))
  {lastError="Invalid phase ID";
   return false;
  }
 for (i=0;i<nComp;i++)
  {if ((compIndices[i]<0)||(compIndices[i]>=(int)compounds.size()))
    {lastError="Compound index out of range";
     return false;
    }
   for (j=0;j<i;j++) 
    if (compIndices[i]==compIndices[j])
     {lastError="At least one compound appears in the mixture more than once";
      return false;
     }
   if (T>compounds[compIndices[i]]->TC)
    {lastError="Temperature exceeds critical temperature of one of the compounds in the mixture";
     return false;
    }
   if (_isnan(X[i]))
    {lastError="At least one value for composition is missing";
     return false;
    }
   if (!_finite(X[i]))
    {lastError="At least one value for composition is not finite";
     return false;
    }
   if (X[i]<0)
    {lastError="At least one value for composition is negative";
     return false;
    }
  }
 if (!CheckTemperature(T)) return false;
 if (!CheckPressure(P)) return false;
 //check the properties and allocate and assign the return values
 int offset;
 valueCounts.resize(nProp);
 valueOffsets.resize(nProp);
 valuePointers.resize(nProp);
 offset=0;
 for (i=0;i<nProp;i++)
  {//get the number of values for this property
   if ((propIDs[i]<0)||(propIDs[i]>=SinglePhasePropertyCount))
    {lastError="One or more invalid single-phase property IDs";
     return false;
    }
   int nVal=1;
   int dim=SinglePhasePropertyDimension[propIDs[i]];
   while (dim)
    {nVal*=nComp;
     dim--;
    }
   valueCounts[i]=nVal;
   valueOffsets[i]=offset;
   offset+=nVal;
  }
 values.resize(offset); //offset now contains total count
 for (i=0;i<nProp;i++) valuePointers[i]=VECPTR(values)+valueOffsets[i];
 ValueCount=VECPTR(valueCounts);
 Values=VECPTR(valuePointers);
 //calculate the properties
 for (i=0;i<nProp;i++) 
  {double *vals=valuePointers[i];
   switch (propIDs[i])
    {case Density:
         if (phaseID==Vapor)
          {//vapor density, ideal gas law
           *vals=P/(GAS_CONSTANT*T);
          }
         else
          {//liquid density
           // V = sum(X/rho)
           double V=0;
           for (j=0;j<nComp;j++) V+=X[j]/compounds[j]->liqDensCorrelation->Value(T);
           *vals=1.0/V;
          }
		 break;   
     case DensityDT:
         if (phaseID==Vapor)
          {//vapor density, ideal gas law
           *vals=-P/(GAS_CONSTANT*T*T);
          }
         else
          {//liquid density
           // V = sum(X/rho)
           double V=0;
           double VDT=0;
           for (j=0;j<nComp;j++) 
            {double vcomp=1.0/compounds[j]->liqDensCorrelation->Value(T);
             V+=X[j]*vcomp;
             VDT-=X[j]*compounds[j]->liqDensCorrelation->ValueDT(T)*vcomp*vcomp;
            }
           *vals=-VDT/(V*V);
          }
		 break;   
     case DensityDP:
         if (phaseID==Vapor)
          {//vapor density, ideal gas law
           *vals=1.0/(GAS_CONSTANT*T);
          }
         else
          {//liquid density, incompressible
           *vals=0;
          }
		 break;   
     case DensityDX:
         if (phaseID==Vapor)
          {//does not depend on composition
           for (j=0;j<nComp;j++) vals[j]=0;
          }
         else
          {//liquid density
           double V;
           vector<double> vComp; 
           V=0;
           vComp.resize(nComp);
           for (j=0;j<nComp;j++) 
            {vComp[j]=1.0/compounds[j]->liqDensCorrelation->Value(T);
             V+=X[j]*vComp[j];
            }
           double invV2=-1.0/(V*V);
           for (j=0;j<nComp;j++) vals[j]=invV2*vComp[j];
          }
		 break;   
     case DensityDn:
         if (phaseID==Vapor)
          {//does not depend on composition
           for (j=0;j<nComp;j++) vals[j]=0;
          }
         else
          {//liquid density
           double V;
           vector<double> vComp; 
           V=0;
           vComp.resize(nComp);
           for (j=0;j<nComp;j++) 
            {vComp[j]=1.0/compounds[j]->liqDensCorrelation->Value(T);
             V+=X[j]*vComp[j];
            }
           double invV2=-1.0/(V*V);
           for (j=0;j<nComp;j++) vals[j]=invV2*(vComp[j]-V);
          }
		 break;   
     case Volume:
         if (phaseID==Vapor)
          {//vapor density, ideal gas law
           *vals=GAS_CONSTANT*T/P;
          }
         else
          {//liquid density
           // V = sum(X/rho)
           *vals=0;
           for (j=0;j<nComp;j++) *vals+=X[j]/compounds[j]->liqDensCorrelation->Value(T);
          }
		 break;   
     case VolumeDT:
         if (phaseID==Vapor)
          {//vapor density, ideal gas law
           *vals=GAS_CONSTANT/P;
          }
         else
          {//liquid density
           // V = sum(X/rho)
           double VDT=0;
           for (j=0;j<nComp;j++) 
            {double vcomp=1.0/compounds[j]->liqDensCorrelation->Value(T);
             VDT-=X[j]*compounds[j]->liqDensCorrelation->ValueDT(T)*vcomp*vcomp;
            }
           *vals=VDT;
          }
		 break;   
     case VolumeDP:
         if (phaseID==Vapor)
          {//vapor density, ideal gas law
           *vals=-GAS_CONSTANT*T/(P*P);
          }
         else
          {//liquid density, incompressible
           *vals=0;
          }
		 break;   
     case VolumeDX:
         if (phaseID==Vapor)
          {//does not depend on composition
           for (j=0;j<nComp;j++) vals[j]=0;
          }
         else
          {//liquid volume
           for (j=0;j<nComp;j++) vals[j]=1.0/compounds[j]->liqDensCorrelation->Value(T);
          }
		 break;   
     case VolumeDn: 
         if (phaseID==Vapor)
          {//partial molar volume (=V)
           double V=GAS_CONSTANT*T/P;
           for (j=0;j<nComp;j++) vals[j]=V;
          }
         else
          {//liquid volume
           for (j=0;j<nComp;j++) vals[j]=1.0/compounds[j]->liqDensCorrelation->Value(T);
          }
		 break;   
     case Enthalpy:
         //ideal part
         *vals=0;
         for (j=0;j<nComp;j++) if (X[j]>0) *vals+=X[j]*compounds[j]->CpCorrelation->IntValue(T);
         //the pressure integral from P = 0 to P for [V - T (dV/dT)|P] cancels out for an ideal gas as V = T*dV/dT)|P = RT/P
         if (phaseID==Liquid)
          {//correct for Hvap
           for (j=0;j<nComp;j++) if (X[j]>0) *vals-=X[j]*compounds[j]->HvapCorrelation->Value(T);
          }
		 break;   
     case EnthalpyDT:
         *vals=0;
         for (j=0;j<nComp;j++) if (X[j]>0) *vals+=X[j]*compounds[j]->CpCorrelation->Value(T);
         if (phaseID==Liquid)
          {//correct for Hvap
           for (j=0;j<nComp;j++) if (X[j]>0) *vals-=X[j]*compounds[j]->HvapCorrelation->ValueDT(T);
          }
		 break;   
     case EnthalpyDP:
         //neither liquid nor vapor enthalpy depends on pressure
         *vals=0;
		 break;   
     case EnthalpyDX:
     case EnthalpyDn:
         //loop over components
         // DX and Dn are the same because the X-dependence is linear
         for (j=0;j<nComp;j++) 
          {vals[j]=compounds[j]->CpCorrelation->IntValue(T);
           if (phaseID==Liquid) vals[j]-=compounds[j]->HvapCorrelation->Value(T);
          }
		 break;   
     case Entropy:
         *vals=0;
         //shared terms
         for (j=0;j<nComp;j++) 
          if (X[j]>0)
           *vals+=X[j]*(compounds[j]->CpCorrelation->IntValueOverT(T)-GAS_CONSTANT*log(X[j]));  
         if (phaseID==Vapor)
          {//pressure term
           *vals-=GAS_CONSTANT*log(P/REFERENCE_PRESSURE);
          }
         else
          {//pressure and hVap terms
           for (j=0;j<nComp;j++) 
            if (X[j]>0)
             *vals-=X[j]*(GAS_CONSTANT*log(compounds[j]->pSatCorrelation->Value(T)/REFERENCE_PRESSURE)+
                         compounds[j]->HvapCorrelation->Value(T)/T);
          }
		 break;   
     case EntropyDT:
         *vals=0;
         //shared terms
         for (j=0;j<nComp;j++) 
          if (X[j]>0)
           *vals+=X[j]*compounds[j]->CpCorrelation->Value(T)/T;  
         if (phaseID==Liquid)
          {//pressure and hVap terms
           for (j=0;j<nComp;j++) 
            if (X[j]>0)
             *vals-=X[j]*(GAS_CONSTANT*compounds[j]->pSatCorrelation->ValueDT(T)/compounds[j]->pSatCorrelation->Value(T)+
                         compounds[j]->HvapCorrelation->ValueDT(T)/T
                         -compounds[j]->HvapCorrelation->Value(T)/(T*T));
          }
		 break;   
     case EntropyDP:
         if (phaseID==Vapor) *vals=-GAS_CONSTANT/P;
         else *vals=0; //incompressible
		 break;   
     case EntropyDX:
     case EntropyDn:
         {//most terms are the same, except for d (XlnX) / dX vs d n*(XlnX) / dn as dX/dn is not unity
          // the correction for d (XlnX) / dX = 0, for d n*(XlnX) / dn = R sum(X)
          double correction=0;
          if (propIDs[i]==EntropyDn) 
           {for (j=0;j<nComp;j++) correction+=X[j];
            correction*=GAS_CONSTANT;
           }
          //shared terms
          for (j=0;j<nComp;j++) 
           {vals[j]=compounds[j]->CpCorrelation->IntValueOverT(T);
            //add -RlnX, where -RlnX is -infinity for X=0; we take -1e200
            double d=-GAS_CONSTANT*log(X[j]);
            if (!_finite(d)) d=-1e200; else if (d<-1e200) d=-1e200; //force continuity
            vals[j]+=d-GAS_CONSTANT+correction;
           }
          if (phaseID==Liquid)
           {//pressure and hVap terms
            for (j=0;j<nComp;j++) 
             vals[j]-=(GAS_CONSTANT*log(compounds[j]->pSatCorrelation->Value(T)/REFERENCE_PRESSURE)+
                         compounds[j]->HvapCorrelation->Value(T)/T);
           }
	         }
		 break;   
     case Fugacity:
         if (phaseID==Vapor)
          {//X*P
           for (j=0;j<nComp;j++) vals[j]=X[j]*P;
          }
         else
          {//liquid, fug[j]=x[j]*Psat[j]
           for (j=0;j<nComp;j++) vals[j]=X[j]*compounds[j]->pSatCorrelation->Value(T);
          }
		 break;   
     case FugacityDT:
         if (phaseID==Vapor)
          {//d/dT X*P = 0
           for (j=0;j<nComp;j++) vals[j]=0;
          }
         else
          {//liquid, fug[j]=x[j]*Psat[j]
           for (j=0;j<nComp;j++) vals[j]=X[j]*compounds[j]->pSatCorrelation->ValueDT(T);
          }
		 break;   
     case FugacityDP:
         if (phaseID==Vapor)
          {//d/dT X*P = X
           for (j=0;j<nComp;j++) vals[j]=X[j];
          }
         else
          {//liquid, incompressible
           for (j=0;j<nComp;j++) vals[j]=0;
          }
		 break;   
     case FugacityDX:
          if (phaseID==Vapor)
          {//d/dT X*P = P * identity
           memset(vals,0,sizeof(double)*nComp*nComp);
           for (j=0;j<nComp;j++) vals[j+nComp*j]=P;
          }
         else
          {//liquid, fug[j]=x[j]*Psat[j]
           memset(vals,0,sizeof(double)*nComp*nComp);
           for (j=0;j<nComp;j++) vals[j+nComp*j]=compounds[j]->pSatCorrelation->Value(T);
          }
		 break;   
     case FugacityDn:
         //for a total of 1 moles:
         //d X[i] / d n[i] = 1-X[i]
         //d X[i] / d n[j] = -X[i]
         if (phaseID==Vapor)
          {index=0;
           for (j=0;j<nComp;j++)
            {for (k=0;k<nComp;k++)
              {//d X[k] / d n[j]
               if (k==j) vals[index]=P*(1.0-X[k]); else vals[index]=-X[k]*P;
               index++;
              }
            }
          }
         else
          {index=0;
           vector<double> PSat;
           PSat.resize(nComp);
           for (j=0;j<nComp;j++) PSat[j]=compounds[j]->pSatCorrelation->Value(T);
           for (j=0;j<nComp;j++)
            {for (k=0;k<nComp;k++)
              {//d X[k] / d n[j]
               if (k==j) vals[index]=PSat[k]*(1.0-X[k]); else vals[index]=-X[k]*PSat[k];
               index++;
              }
            }
          }
		 break;   
     case FugacityCoefficient:
         if (phaseID==Vapor)
          {//unity
           for (j=0;j<nComp;j++) vals[j]=1.0;
          }
         else
          {//liquid, fug[j]=x[j]*Psat[j]=phi[j]*x[j]*P -> phi[j]=Psat[j]/P
           double invP=1.0/P;
           for (j=0;j<nComp;j++) vals[j]=compounds[j]->pSatCorrelation->Value(T)*invP;
          }
		 break;   
     case FugacityCoefficientDT:
         if (phaseID==Vapor)
          {//zero
           for (j=0;j<nComp;j++) vals[j]=0.0;
          }
         else
          {//liquid
           double invP=1.0/P;
           for (j=0;j<nComp;j++) vals[j]=compounds[j]->pSatCorrelation->ValueDT(T)*invP;
          }
		 break;   
     case FugacityCoefficientDP:
         if (phaseID==Vapor)
          {//zero
           for (j=0;j<nComp;j++) vals[j]=0.0;
          }
         else
          {//liquid
           double invP2=-1.0/(P*P);
           for (j=0;j<nComp;j++) vals[j]=compounds[j]->pSatCorrelation->Value(T)*invP2;
          }
		 break;   
     case FugacityCoefficientDX:
     case FugacityCoefficientDn:
         //zero for all phases
         memset(vals,0,sizeof(double)*nComp*nComp);
		 break;   
     case LogFugacityCoefficient:
         if (phaseID==Vapor)
          {//ln(unity)=0
           for (j=0;j<nComp;j++) vals[j]=0.0;
          }
         else
          {//liquid, ln(phi[j])=ln(Psat[j]/P)
           double lnP=log(P);
           for (j=0;j<nComp;j++) vals[j]=log(compounds[j]->pSatCorrelation->Value(T))-lnP;
          }
		 break;   
     case LogFugacityCoefficientDT:
         if (phaseID==Vapor)
          {//d/dT ln(unity)=0
           for (j=0;j<nComp;j++) vals[j]=0.0;
          }
         else
          {//liquid
           for (j=0;j<nComp;j++) vals[j]=compounds[j]->pSatCorrelation->ValueDT(T)/compounds[j]->pSatCorrelation->Value(T);
          }
		 break;   
     case LogFugacityCoefficientDP:
         if (phaseID==Vapor)
          {//d/dP ln(unity)=0
           for (j=0;j<nComp;j++) vals[j]=0.0;
          }
         else
          {//liquid
           double minInvP=-1.0/P;
           for (j=0;j<nComp;j++) vals[j]=minInvP;
          }
		 break;   
     case LogFugacityCoefficientDX:
     case LogFugacityCoefficientDn:
         memset(vals,0,sizeof(double)*nComp*nComp);
		 break;   
     case Activity:
         if (phaseID==Vapor)
          {lastError="Activity not supported for vapor phase";
           return false;
          }
         //liquid activity coefficent is unity, activity therefore equals X
         for (j=0;j<nComp;j++) vals[j]=X[j];
		 break;   
     case ActivityDT:
     case ActivityDP:
         if (phaseID==Vapor)
          {lastError="Activity not supported for vapor phase";
           return false;
          }
         //zero
         for (j=0;j<nComp;j++) vals[j]=0;
		 break;   
     case ActivityDX:
         if (phaseID==Vapor)
          {lastError="Activity not supported for vapor phase";
           return false;
          }
         //identity matrix
         memset(vals,0,sizeof(double)*nComp*nComp);
         for (j=0;j<nComp;j++) vals[j+nComp*j]=1.0;
		 break;   
     case ActivityDn:
         if (phaseID==Vapor)
          {lastError="Activity not supported for vapor phase";
           return false;
          }
         //for a total of 1 moles:
         //d X[i] / d n[i] = 1-X[i]
         //d X[i] / d n[j] = -X[i]
         index=0;
         for (j=0;j<nComp;j++)
          {for (k=0;k<nComp;k++)
            {//d X[k] / d n[j]
             if (k==j) vals[index]=1.0-X[k]; else vals[index]=-X[k];
             index++;
            }
          }
		 break;   
     default:
         lastError="Internal error: property calculation not defined";
         return false;
    }    
  }
 //all ok
 return true;
}

//! Get two-phase mixture properties at specified temperature, pressure and composition
/*!
  Calculate and get two-phase mixture properties. The properties are returned in arrays 
  that are allocated and stored by this DLL. The return values are only valid until the next
  call to GetSinglePhaseProperties, GetTwoPhaseProperties or Flash, so store the return values, but 
  not the pointers to them. Multiple properties can be requested in a single call. For each
  property, the values and number of values are returned
  
  The only supported properties are kvalue, which is fugacity of phase 2 divided by fugacity of phase 1
  and log(kvalue).
  
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
  \param ValueCount Receives the number of values for each of the properties, one value for each property
  \param Values Receives the values, one double array for each property. Size of the array corresponds to valueCount for each property
  \return True if ok
  \sa GetCompoundCount(), LastError(), Phase, TwoPhaseProperty
*/

bool PropertyPackage::GetTwoPhaseProperties(int nComp,const int *compIndices,Phase phaseID1,Phase phaseID2,double T1,double T2,double P1,double P2,const double *X1,const double *X2,int nProp,TwoPhaseProperty *propIDs,int *&ValueCount,double **&Values)
{//this implementation is for instructive purposes only; a production implementation would use
 // stored values for combined property evaluations, THIS ROUTINE DOES NOT TAKE ADVANTAGE OF 
 // SIMULTANEOUS PROPERTY CALCULATIONS!!!
 int i,j;
 if (!initialized)
  {lastError="Property package has not been initialized";
   return false;
  }
 //check the inputs
 if ((phaseID1!=Vapor)&&(phaseID1!=Liquid))
  {lastError="Invalid phase ID 1";
   return false;
  }
 if ((phaseID2!=Vapor)&&(phaseID2!=Liquid))
  {lastError="Invalid phase ID 2";
   return false;
  }
 if (phaseID1==phaseID2)
  {lastError="Phases 1 and 2 cannot be the same";
   return false;
  }
 for (i=0;i<nComp;i++)
  {if ((compIndices[i]<0)||(compIndices[i]>=(int)compounds.size()))
    {lastError="Compound index out of range";
     return false;
    }
   for (j=0;j<i;j++) 
    if (compIndices[i]==compIndices[j])
     {lastError="At least one compound appears in the mixture more than once";
      return false;
     }
   if (T1>compounds[compIndices[i]]->TC)
    {lastError="Temperature of phase 1 exceeds critical temperature of one of the compounds in the mixture";
     return false;
    }
   if (T2>compounds[compIndices[i]]->TC)
    {lastError="Temperature of phase 2 exceeds critical temperature of one of the compounds in the mixture";
     return false;
    }
   //note: this routine does not actually use compositions; all K values are independent of compositions
   if (_isnan(X1[i]))
    {lastError="At least one value for composition of phase 1 is missing";
     return false;
    }
   if (_isnan(X2[i]))
    {lastError="At least one value for composition of phase 2 is missing";
     return false;
    }
   if (!_finite(X1[i]))
    {lastError="At least one value for composition of phase 1 is not finite";
     return false;
    }
   if (!_finite(X2[i]))
    {lastError="At least one value for composition of phase 2 is not finite";
     return false;
    }
   if (X1[i]<0)
    {lastError="At least one value for composition of phase 1 is negative";
     return false;
    }
   if (X2[i]<0)
    {lastError="At least one value for composition of phase 2 is negative";
     return false;
    }
  }
 //even though we only use T and P of the liquid phase, we expect both of them to be valid
 // (mostly they would be equal in any case)
 if (!CheckTemperature(T1)) return false;
 if (!CheckTemperature(T2)) return false;
 if (!CheckPressure(P1)) return false;
 if (!CheckPressure(P2)) return false;
 //check the properties and allocate and assign the return values
 int offset;
 valueCounts.resize(nProp);
 valueOffsets.resize(nProp);
 valuePointers.resize(nProp);
 offset=0;
 for (i=0;i<nProp;i++)
  {//get the number of values for this property
   if ((propIDs[i]<0)||(propIDs[i]>=TwoPhasePropertyCount))
    {lastError="One or more invalid two-phase property IDs";
     return false;
    }
   int nVal=1;
   int dim=TwoPhasePropertyDimension[propIDs[i]];
   if (dim==DIMENSION_MATRIX) nVal=2; //as we only serve vector properties, a matrix is a derivative. Values for both phases are returned
   while (dim)
    {nVal*=nComp;
     dim--;
    }
   valueCounts[i]=nVal;
   valueOffsets[i]=offset;
   offset+=nVal;
  }
 values.resize(offset); //offset now contains total count
 for (i=0;i<nProp;i++) valuePointers[i]=VECPTR(values)+valueOffsets[i];
 ValueCount=VECPTR(valueCounts);
 Values=VECPTR(valuePointers);
 //calculate the properties
 // Kvalue = FugacityCoefficient2/FugacityCoefficient1
 //  if phase 2 is Liquid, then phase 1 must be Vapor and Kvalue = Psat/P/1 = Psat/P
 //  if phase 1 is Liquid, then phase 2 must be Vapor and Kvalue = 1/(Psat/P) = P/Psat
 // so the K values depend on pressure and temperature (Psat=f(T)) but not on composition 
 // here, P and T are those of the liquid phase
 for (i=0;i<nProp;i++) 
  {double *vals=valuePointers[i];
   switch (propIDs[i])
    {case Kvalue:
         if (phaseID1==Vapor) 
          {//Kvalue = Psat(T2)/P2
           double invP=1.0/P2;
           for (j=0;j<nComp;j++) vals[j]=compounds[j]->pSatCorrelation->Value(T2)*invP;
          }
         else 
          {//Kvalue = P1/Psat(T1)
           for (j=0;j<nComp;j++) vals[j]=P1/compounds[j]->pSatCorrelation->Value(T1);
          }
		 break;   
     case KvalueDT:
         if (phaseID1==Vapor) 
          {//Kvalue = Psat(T2)/P2
           double invP=1.0/P2;
           for (j=0;j<nComp;j++) vals[j]=compounds[j]->pSatCorrelation->ValueDT(T2)*invP;
          }
         else 
          {//Kvalue = P1/Psat(T1)
           for (j=0;j<nComp;j++) 
            {double Psat=compounds[j]->pSatCorrelation->Value(T1);
             vals[j]=-P1*compounds[j]->pSatCorrelation->ValueDT(T1)/(Psat*Psat);
            }
          }
		 break;   
     case KvalueDP:
         if (phaseID1==Vapor) 
          {//Kvalue = Psat(T2)/P2
           double invP2=-1.0/(P2*P2);
           for (j=0;j<nComp;j++) vals[j]=compounds[j]->pSatCorrelation->Value(T2)*invP2;
          }
         else 
          {//Kvalue = P1/Psat(T1)
           for (j=0;j<nComp;j++) vals[j]=1.0/compounds[j]->pSatCorrelation->Value(T1);
          }
		 break;   
     case LogKvalue: 
         //LogKValue=LogFugacity2-LogFugacity1
         if (phaseID1==Vapor) 
          {//Kvalue = Psat(T2)/P2
           double invP=1.0/P2;
           for (j=0;j<nComp;j++) vals[j]=log(compounds[j]->pSatCorrelation->Value(T2)*invP);
          }
         else 
          {//Kvalue = P1/Psat(T1)
           for (j=0;j<nComp;j++) vals[j]=log(P1/compounds[j]->pSatCorrelation->Value(T1));
          }
		 break;   
     case LogKvalueDT:
         if (phaseID1==Vapor) 
          {//Kvalue = Psat(T2)/P2
           for (j=0;j<nComp;j++) vals[j]=compounds[j]->pSatCorrelation->ValueDT(T2)/compounds[j]->pSatCorrelation->Value(T2);
          }
         else 
          {//Kvalue = P1/Psat(T1)
           for (j=0;j<nComp;j++) vals[j]=-compounds[j]->pSatCorrelation->ValueDT(T1)/compounds[j]->pSatCorrelation->Value(T1);
          }
		 break;   
     case LogKvalueDP:
         if (phaseID1==Vapor) 
          {//Kvalue = Psat(T2)/P2
           double invP=-1.0/P2;
           for (j=0;j<nComp;j++) vals[j]=invP;
          }
         else 
          {//Kvalue = P1/Psat(T1)
           double invP=1.0/P1;
           for (j=0;j<nComp;j++) vals[j]=invP;
          }
		 break;   
     case LogKvalueDX:
     case LogKvalueDn:
     case KvalueDX:
     case KvalueDn:
         //no composition dependence for either phase!
         memset(vals,0,2*nComp*nComp*sizeof(double));
		 break;   
     default:
         lastError="Internal error: property calculation not defined";
         return false;
    }    
  }
 //all ok
 return true;
}

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
  \sa GetCompoundCount(), LastError(), Phase, FlashType, FlashPhaseType
*/

bool PropertyPackage::Flash(int nComp,const int *compIndices,const double *X,FlashType type,FlashPhaseType phaseType,double spec1,double spec2,int &phaseCount,Phase *&phases,double *&phaseFractions,double **&phaseCompositions,double &T, double &P)
{int i,j;
 double H,S,VF;
 if (!initialized)
  {lastError="Property package has not been initialized";
   return false;
  }
 flashPhaseType=phaseType;
 //check the inputs, set up compound map as we go (we only consider compounds with non-zero mole fraction)
 flashCompounds.clear();
 flashCompounds.reserve(nComp);
 flashCompoundMapping.clear();
 flashCompoundMapping.reserve(nComp);
 flashComposition.clear();
 flashComposition.reserve(nComp);
 for (i=0;i<nComp;i++)
  {if ((compIndices[i]<0)||(compIndices[i]>=(int)compounds.size()))
    {lastError="Compound index out of range";
     return false;
    }
   for (j=0;j<i;j++) 
    if (compIndices[i]==compIndices[j])
     {lastError="At least one compound appears in the mixture more than once";
      return false;
     }
   if (_isnan(X[i]))
    {lastError="At least one value for composition is missing";
     return false;
    }
   if (!_finite(X[i]))
    {lastError="At least one value for composition is not finite";
     return false;
    }
   if (X[i]<0)
    {lastError="At least one value for composition is negative";
     return false;
    }
   if (X[i]>0)
    {flashCompounds.push_back(compIndices[i]);
     flashComposition.push_back(X[i]);
     flashCompoundMapping.push_back(i);
    }
  }
 if (flashCompounds.size()==0)
  {lastError="All compositions are zero";
   return false;
  }
 //check flash type and calculate
 vapX.resize(flashComposition.size());
 liqX.resize(flashComposition.size());
 switch (type)
  {case TP: 
     //TP flash 
     T=spec1;
     P=spec2;
     if (!TPFlash(T,P)) return false; //error has been set
     break;
   case TVF:
     T=spec1;
     VF=spec2;
     if (!TVFFlash(T,VF,P)) return false; //error has been set
     break;
   case PVF:
     P=spec1;
     VF=spec2;
     if (!PVFFlash(P,VF,T)) return false; //error has been set
     break;
   case TVFm:
     T=spec1;
     VF=spec2;
     if (!TVFmFlash(T,VF,P)) return false; //error has been set
     break;
   case PVFm:
     P=spec1;
     VF=spec2;
     if (!PVFmFlash(P,VF,T)) return false; //error has been set
     break;
   case PH:
     P=spec1;
     H=spec2;
     if (!PHFlash(P,H,T)) return false; //error has been set
     break;
   case PS:
     P=spec1;
     S=spec2;
     if (!PSFlash(P,S,T)) return false; //error has been set
     break;
   default:
    lastError="Invalid flash type specification";
    return false;
  }
 //flash returned ok, map outputs
 phaseCount=0;
 if (vaporExists) phaseCount++;
 if (liquidExists) phaseCount++;
 existingPhases.resize(phaseCount);
 valuePointers.resize(phaseCount);
 j=phaseCount*(1+nComp);
 values.resize(j);//alloc space for phase fraction of each resulting phase followed by composition of each resulting phase
 for (i=0;i<j;i++) values[i]=0;
 j=0; //phase index
 phases=VECPTR(existingPhases);
 phaseFractions=VECPTR(values);
 phaseCompositions=VECPTR(valuePointers);
 if (vaporExists)
  {existingPhases[j]=Vapor;
   phaseFractions[j]=vapFrac;
   valuePointers[j]=VECPTR(values)+phaseCount+j*nComp;
   for (i=0;i<(int)flashCompoundMapping.size();i++)phaseCompositions[j][flashCompoundMapping[i]]=vapX[i];
   j++;
  }
 if (liquidExists)
  {existingPhases[j]=Liquid;
   phaseFractions[j]=liqFrac;
   valuePointers[j]=VECPTR(values)+phaseCount+j*nComp;
   for (i=0;i<(int)flashCompoundMapping.size();i++) phaseCompositions[j][flashCompoundMapping[i]]=liqX[i];
   j++;
  }
 //all ok 
 return true;
}

//! Check a temperature
/*!
  Internal routine to check a temperature, sets the error in case not ok
  \param T Temperature to check [K]
  \return True if ok
*/

bool PropertyPackage::CheckTemperature(double T)
{if (T<=0)
  {lastError="Temperature must be positive";
   return false;
  }
 if (_isnan(T))
  {lastError="Temperature is missing";
   return false;
  }
 if (!_finite(T))
  {lastError="Temperature is not finite";
   return false;
  }
 return true;
}

//! Check a pressure
/*!
  Internal routine to check a pressure, sets the error in case not ok
  \param P Pressure to check [Pa]
  \return True if ok
*/

bool PropertyPackage::CheckPressure(double P)
{if (P<=0)
  {lastError="Pressure must be positive";
   return false;
  }
 if (_isnan(P))
  {lastError="Pressure is missing";
   return false;
  }
 if (!_finite(P))
  {lastError="Pressure is not finite";
   return false;
  }
 return true;
}

//! Check a vapor fraction
/*!
  Internal routine to check a vapor fraction, sets the error in case not ok
  \param VF Vapor phase fraction to check [mol/mol] or [kg/kg]
  \return True if ok
*/

bool PropertyPackage::CheckVaporPhaseFraction(double VF)
{if (VF<0)
  {lastError="Vapor fraction cannot be negative";
   return false;
  }
 if (VF>1.0)
  {lastError="Vapor fraction cannot exceed unity";
   return false;
  }
 if (_isnan(VF))
  {lastError="Vapor fraction is missing";
   return false;
  }
 if (!_finite(VF))
  {lastError="Vapor fraction is not finite";
   return false;
  }
 return true;
}

//! Check an enthalpy
/*!
  Internal routine to check enthalpy, sets the error in case not ok
  \param H Enthalpy to check [J/mol]
  \return True if ok
*/

bool PropertyPackage::CheckEnthalpy(double H)
{if (_isnan(H))
  {lastError="Enthalpy is missing";
   return false;
  }
 if (!_finite(H))
  {lastError="Enthalpy is not finite";
   return false;
  }
 return true;
}

//! Check an entropy
/*!
  Internal routine to check entropy, sets the error in case not ok
  \param S Entropy to check [J/mol/K]
  \return True if ok
*/

bool PropertyPackage::CheckEntropy(double S)
{if (_isnan(S))
  {lastError="Entropy is missing";
   return false;
  }
 if (!_finite(S))
  {lastError="Entropy is not finite";
   return false;
  }
 return true;
}

//! Calculate dew point pressure given temperature
/*!
  Internal routine to calculate dew point pressure given temperature
  \return Dew point pressure [Pa]
  \sa Flash()
*/

double PropertyPackage::DewPointPressure()
{//Psat must have already been calculated at T
 double num,denom;
 int i,j;
 num=1.0;
 denom=0;
 for (i=0;i<(int)flashCompounds.size();i++)
  {num*=Psat[i];
   double d=1.0;
   for (j=0;j<(int)flashCompounds.size();j++) if (j!=i) d*=Psat[j];
   denom+=d*flashComposition[i];
  }
 return num/denom;
}

//! Calculate bubble point pressure given temperature
/*!
  Internal routine to calculate bubble point pressure given temperature
  \return Bubble point pressure [Pa]
  \sa Flash()
*/

double PropertyPackage::BubblePointPressure()
{//Psat must have already been calculated at T
 double Pbub=0;
 int i;
 for (i=0;i<(int)flashCompounds.size();i++) Pbub+=flashComposition[i]*Psat[i];
 return Pbub;
}

//! Target function for solving TP flash problem
/*!
  Target function for solving TP flash problem; solves the Rachford Rice
  equation for constant K values. Kminus1 needs to be set before calling
  this function
  \param param Parameter passed to solver constructor: PropertyPackage
  \param X Degree of freedom solved for: vapor fraction
  \param F Receives the function value at X
  \param error Receives the error description in case of failure
  \return True if ok
  \sa TPFlash(), Solver1Dim
*/

bool TPFlashFunc(void *param,double X,double &F,string &error)
{int i;
 PropertyPackage *pp=(PropertyPackage *)param;
 F=0;
 for (i=0;i<(int)pp->flashComposition.size();i++) F+=pp->flashComposition[i]*pp->Kminus1[i]/(1.0+X*pp->Kminus1[i]);
 return true;
}

//! Calculate TP phase equilibrium
/*!
  Internal routine to calculate TP equilibrium
  
  For a single compound, P is compared to the vapor pressure.
  
  For a multi-compound mixture, vapor-only or liquid-only solutions are
  returned if P > Pbub or P < Pdew. For Pdew < P < Pbub, the two 
  phase solution is solved for constant K values by solving the 
  Rachford Rice equation.
  
  \param T Temperature [K]
  \param P Pressure [Pa]
  \return True if ok
  \sa Flash(), TPFlashFunc()
*/

bool PropertyPackage::TPFlash(double T,double P)
{int i;
 for (i=0;i<(int)flashCompounds.size();i++)
  {if (T>compounds[flashCompounds[i]]->TC)
    {lastError="Temperature exceeds critical temperature of at least one compound";
     return false;
    }
  }
 if (!CheckTemperature(T)) return false;
 if (!CheckTemperature(P)) return false;
 switch (flashPhaseType)
  {case VaporLiquid:
    break;
   case VaporOnly:
    goto vapOnly;
   case LiquidOnly:
    goto liqOnly;
   default:
    lastError="Invalid/unsupported flashPhaseType argument";
    return false;
  }
 if (flashCompounds.size()==1)
  {//single compound TP flash
   double PSat=compounds[flashCompounds[0]]->pSatCorrelation->Value(T);
   if (P>PSat)
    {//all liquid
     liqOnly:
     for (i=0;i<(int)flashCompounds.size();i++) liqX[i]=flashComposition[i];
     vaporExists=false;
     liquidExists=true;
     liqFrac=1.0;
     vapFrac=0.0; //not required for TP flash, but may be required for other flashes that iterate over this flash
    }
   else
    {//all vapor
     vapOnly: 
     for (i=0;i<(int)flashCompounds.size();i++) vapX[i]=flashComposition[i];
     vaporExists=true;
     liquidExists=false;
     vapFrac=1.0;
     liqFrac=0.0; //not required for TP flash, but may be required for other flashes that iterate over this flash
    }
   return true;
  }
 //pre-calc Psat
 Psat.resize(flashCompounds.size());
 for (i=0;i<(int)flashCompounds.size();i++) Psat[i]=compounds[flashCompoundMapping[i]]->pSatCorrelation->Value(T);
 //check ranges of two-phase solution
 double Pbub=BubblePointPressure();
 if (P>Pbub) goto liqOnly;
 double Pdew=DewPointPressure();
 if (P<Pdew) goto vapOnly;
 //two phase solution, solve using Rachford-Rice
 // http://en.wikipedia.org/wiki/Flash_evaporation
 Kminus1.resize(flashCompounds.size());
 for (i=0;i<(int)flashCompounds.size();i++) Kminus1[i]=Psat[i]/P-1.0;
 Solver1Dim solver(TPFlashFunc,0,1,this,1e-8);
 if (!solver.Solve(vapFrac,lastError)) 
  {lastError="TP flash solution failed: "+lastError;
   return false;
  }
 //fill in results
 vaporExists=liquidExists=true;
 liqFrac=1.0-vapFrac;
 for (i=0;i<(int)flashComposition.size();i++)
  {liqX[i]=flashComposition[i]/(1.0+vapFrac*Kminus1[i]);
   vapX[i]=(1.0+Kminus1[i])*liqX[i];
  }
 return true;
}

//! Target function for solving TVF flash problem
/*!
  Target function for solving TVF flash problem; solves the TP
  flash and returns vapFrac-flashVF
  \param param Parameter passed to solver constructor: PropertyPackage
  \param X Degree of freedom solved for: pressure
  \param F Receives the function value at X
  \param error Receives the error description in case of failure
  \return True if ok
  \sa TPFlash(), Solver1Dim
*/

bool TVFFlashFunc(void *param,double X,double &F,string &error)
{PropertyPackage *pp=(PropertyPackage *)param;
 if (!pp->TPFlash(pp->Tflash,X)) 
  {error=pp->lastError;
   return false;
  }
 F=pp->vapFrac-pp->VFflash;
 return true;
}

//! Calculate TVF phase equilibrium
/*!
  Internal routine to calculate TVF equilibrium
  
  For a single compound, pressure equals the vapor pressure and the solution is trivial.
  
  For VF = 0, pressure equals Pbub and y = x * Psat / P.
  
  For VF = 1, pressure equals Pdew and x = y * P / Psat.
  
  For 0 < VF < 1, the TP flash is iteratively solved for resulting the proper vapor fraction.
  
  \param T Temperature [K]
  \param VF Vapor phase fraction [mol/mol]
  \param P Receives equilibrium pressure [Pa]
  \return True if ok
  \sa Flash(), TVFFlashFunc()
*/

bool PropertyPackage::TVFFlash(double T,double VF,double &P)
{int i;
 for (i=0;i<(int)flashCompounds.size();i++)
  {if (T>compounds[flashCompounds[i]]->TC)
    {lastError="Temperature exceeds critical temperature of at least one compound";
     return false;
    }
  }
 if (!CheckTemperature(T)) return false;
 if (!CheckVaporPhaseFraction(VF)) return false;
 switch (flashPhaseType)
  {case VaporLiquid:
    break;
   case VaporOnly:
   case LiquidOnly:
    lastError="Single phase flashes with vapor fraction specification are not supported";
    return false;
   default:
    lastError="Invalid/unsupported flashPhaseType argument";
    return false;
  }
 vapFrac=VF; //we know the resulting phase fractions
 liqFrac=1.0-VF;
 vaporExists=liquidExists=true;
 if (flashCompounds.size()==1)
  {//single compound TVF flash
   P=compounds[flashCompounds[0]]->pSatCorrelation->Value(T);
   vapX[0]=liqX[0]=1.0;
   return true;
  }
 //pre-calc the vapor pressures
 Psat.resize(flashCompounds.size());
 for (i=0;i<(int)flashCompounds.size();i++) Psat[i]=compounds[flashCompoundMapping[i]]->pSatCorrelation->Value(T);
 if (VF==0)
  {//bubble point calculation
   P=BubblePointPressure();
   for (i=0;i<(int)flashCompounds.size();i++)
    {liqX[i]=flashComposition[i];
     vapX[i]=liqX[i]*compounds[flashCompounds[i]]->pSatCorrelation->Value(T)/P;
    }
   return true;   
  } 
 if (VF==1.0)
  {//dew point calculation
   P=DewPointPressure();
   for (i=0;i<(int)flashCompounds.size();i++)
    {vapX[i]=flashComposition[i];
     liqX[i]=vapX[i]*P/compounds[flashCompounds[i]]->pSatCorrelation->Value(T);
    }
   return true;   
  } 
 //find P so that VF is ok by solving TP flash
 double Pdew=DewPointPressure();
 double Pbub=BubblePointPressure();
 Tflash=T;
 VFflash=VF;
 Solver1Dim solver(TVFFlashFunc,Pdew,Pbub,this,1e-4);
 if (!solver.Solve(P,lastError)) 
  {lastError="TVF flash solution failed: "+lastError;
   return false;
  }
 //results are already filled in by TP flash, but make sure phase fractions are ok
 vapFrac=VF; 
 liqFrac=1.0-VF;
 return true;
}

//! Target function for solving Psat(T)=Tspec
/*!
  Target function for solving Psat(T)=Tspec for a single compound
  \param param Parameter passed to solver constructor: PropertyPackage
  \param X Degree of freedom solved for: temperature
  \param F Receives the function value at X
  \param error Receives the error description in case of failure
  \return True if ok
  \sa PVFFlash(), Solver1Dim
*/

bool TsatFlashFunc(void *param,double X,double &F,string &error)
{PropertyPackage *pp=(PropertyPackage *)param;
 F=pp->compounds[pp->flashCompounds[0]]->pSatCorrelation->Value(X)-pp->Pflash;
 return true;
}

//! Target function for solving Pbub(T)=Tspec
/*!
  Target function for solving Pbub(T)=Tspec for a mixture
  \param param Parameter passed to solver constructor: PropertyPackage
  \param X Degree of freedom solved for: temperature
  \param F Receives the function value at X
  \param error Receives the error description in case of failure
  \return True if ok
  \sa PVFFlash(), Solver1Dim
*/

bool TbubFlashFunc(void *param,double X,double &F,string &error)
{int i;
 PropertyPackage *pp=(PropertyPackage *)param;
 for (i=0;i<(int)pp->flashCompounds.size();i++) pp->Psat[i]=pp->compounds[pp->flashCompoundMapping[i]]->pSatCorrelation->Value(X);
 F=pp->BubblePointPressure()-pp->Pflash;
 return true;
}

//! Target function for solving Pdew(T)=Tspec
/*!
  Target function for solving Pdew(T)=Tspec for a mixture
  \param param Parameter passed to solver constructor: PropertyPackage
  \param X Degree of freedom solved for: temperature
  \param F Receives the function value at X
  \param error Receives the error description in case of failure
  \return True if ok
  \sa PVFFlash(), Solver1Dim
*/

bool TdewFlashFunc(void *param,double X,double &F,string &error)
{int i;
 PropertyPackage *pp=(PropertyPackage *)param;
 for (i=0;i<(int)pp->flashCompounds.size();i++) pp->Psat[i]=pp->compounds[pp->flashCompoundMapping[i]]->pSatCorrelation->Value(X);
 F=pp->DewPointPressure()-pp->Pflash;
 return true;
}

//! Target function for solving PVF flash problem
/*!
  Target function for solving PVF flash problem.  
  \param param Parameter passed to solver constructor: PropertyPackage
  \param X Degree of freedom solved for: temperature
  \param F Receives the function value at X
  \param error Receives the error description in case of failure
  \return True if ok
  \sa PVFFlash(), Solver1Dim
*/

bool PVFFlashFunc(void *param,double X,double &F,string &error)
{PropertyPackage *pp=(PropertyPackage *)param;
 if (!pp->TPFlash(X,pp->Pflash)) 
  {error=pp->lastError;
   return false;
  }
 F=pp->vapFrac-pp->VFflash;
 return true;
}

//! Calculate PVF phase equilibrium
/*!
  Internal routine to calculate PVF equilibrium
  
  For a single compound, PSat(T) = P is solved.
  
  For VF = 0, Pbub(T)=P is solved and y = x * Psat / P.
  
  For VF = 1, Pdew(T)=P is solved and x = y * P / Psat.
  
  For 0 < VF < 1, the TP flash is iteratively solved for resulting the proper vapor fraction.
  
  Solutions are limited between 10 < T < min(TC)

  \param P Pressure [Pa]
  \param VF Vapor phase fraction [mol/mol]
  \param T Receives equilibrium temperature [K]
  \return True if ok
  \sa Flash(), TsatFlashFunc(), TbubFlashFunc(), TdewFlashFunc(), PVFFlashFunc()
  
*/

bool PropertyPackage::PVFFlash(double P,double VF,double &T)
{int i;
 if (!CheckPressure(P)) return false;
 if (!CheckVaporPhaseFraction(VF)) return false;
 switch (flashPhaseType)
  {case VaporLiquid:
    break;
   case VaporOnly:
   case LiquidOnly:
    lastError="Single phase flashes with vapor fraction specification are not supported";
    return false;
   default:
    lastError="Invalid/unsupported flashPhaseType argument";
    return false;
  }
 vapFrac=VF; //we know the resulting phase fractions
 liqFrac=1.0-VF;
 vaporExists=liquidExists=true;
 Pflash=P;
 if (flashCompounds.size()==1)
  {//single compound PVF flash, solve Psat(T) = P for T
   Solver1Dim solver(TsatFlashFunc,50.0,compounds[flashCompounds[0]]->TC,this,1e-4);
   if (!solver.Solve(T,lastError)) 
    {lastError="PVF flash solution failed: "+lastError;
     return false;
    }
   vapX[0]=liqX[0]=1.0;
   return true;
  }
 Psat.resize(flashCompounds.size());
 double Tmax=compounds[flashCompounds[0]]->TC; //get Tmax = min(TC)
 for (i=1;i<(int)flashCompounds.size();i++) if (compounds[flashCompounds[i]]->TC<Tmax) Tmax=compounds[flashCompounds[i]]->TC;
 if (VF==0)
  {//bubble point calculation
   Solver1Dim solver(TbubFlashFunc,50.0,Tmax,this,1e-4);
   if (!solver.Solve(T,lastError)) 
    {lastError="PVF flash solution failed: "+lastError;
     return false;
    }
   //compositions
   for (i=0;i<(int)flashCompounds.size();i++)
    {liqX[i]=flashComposition[i];
     vapX[i]=liqX[i]*compounds[flashCompounds[i]]->pSatCorrelation->Value(T)/P;
    }
   return true;   
  } 
 if (VF==1.0)
  {//dew point calculation
   Solver1Dim solver(TdewFlashFunc,50.0,Tmax,this,1e-4);
   if (!solver.Solve(T,lastError)) 
    {lastError="PVF flash solution failed: "+lastError;
     return false;
    }
   for (i=0;i<(int)flashCompounds.size();i++)
    {vapX[i]=flashComposition[i];
     liqX[i]=vapX[i]*P/compounds[flashCompounds[i]]->pSatCorrelation->Value(T);
    }
   return true;   
  } 
 //find T so that VF is ok by solving TP flash
 double Tbub,Tdew;
 Solver1Dim solverTbub(TbubFlashFunc,50.0,Tmax,this,1e-4);
 if (!solverTbub.Solve(Tbub,lastError)) 
  {lastError="PVF flash solution failed: "+lastError;
   return false; 
  }
 Solver1Dim solverTdew(TdewFlashFunc,50.0,Tmax,this,1e-4);
 if (!solverTdew.Solve(Tdew,lastError)) 
  {lastError="PVF flash solution failed: "+lastError;
   return false; 
  }
 VFflash=VF;
 Solver1Dim solver(PVFFlashFunc,Tbub,Tdew,this,1e-4);
 if (!solver.Solve(T,lastError)) 
  {lastError="PVF flash solution failed: "+lastError;
   return false;
  }
 //results are already filled in by TP flash, but make sure phase fractions are ok
 vapFrac=VF; 
 liqFrac=1.0-VF;
 return true;
}

//! Returns the mass vapor fraction during VFm flash calculations
/*!
  Returns the mass vapor fraction during VFm flash calculations
  \return Mass vapor fraction [kg/kg]
  \sa TPFlash()
*/


double PropertyPackage::MassVapFrac()
{if ((vapFrac==0)||(vapFrac==1.0)) return vapFrac;
 double vapMass,liqMass;
 int i;
 vapMass=liqMass=0;
 for (i=0;i<(int)flashCompounds.size();i++)
  {double MW=compounds[flashCompounds[i]]->MW;
   vapMass+=vapX[i]*MW;
   liqMass+=liqX[i]*MW;
  }
 return vapFrac*vapMass/(vapFrac*vapMass+liqFrac*liqMass);
}

//! Target function for solving TVFm flash problem
/*!
  Target function for solving TVFm flash problem; solves the TP
  flash and returns massVapFrac-flashVF
  \param param Parameter passed to solver constructor: PropertyPackage
  \param X Degree of freedom solved for: pressure
  \param F Receives the function value at X
  \param error Receives the error description in case of failure
  \return True if ok
  \sa TPFlash(), MassVapFrac(), Solver1Dim
*/

bool TVFmFlashFunc(void *param,double X,double &F,string &error)
{PropertyPackage *pp=(PropertyPackage *)param;
 if (!pp->TPFlash(pp->Tflash,X)) 
  {error=pp->lastError;
   return false;
  }
 F=pp->MassVapFrac()-pp->VFflash;
 return true;
}

//! Calculate TVF phase equilibrium
/*!
  Internal routine to calculate TVF equilibrium
  
  For VF = 0, VF = 1 or single compound, returns the molar TVF flash result
  
  For mixtures with 0 < VF < 1, the TP flash is iteratively solved for resulting the proper mass vapor fraction.
  
  \param T Temperature [K]
  \param VF Vapor phase fraction [kg/kg]
  \param P Receives equilibrium pressure [Pa]
  \return True if ok
  \sa Flash(), TVFmFlashFunc()
*/

bool PropertyPackage::TVFmFlash(double T,double VF,double &P)
{int i;
 for (i=0;i<(int)flashCompounds.size();i++)
  {if (T>compounds[flashCompounds[i]]->TC)
    {lastError="Temperature exceeds critical temperature of at least one compound";
     return false;
    }
  }
 if (!CheckTemperature(T)) return false;
 if (!CheckVaporPhaseFraction(VF)) return false;
 switch (flashPhaseType)
  {case VaporLiquid:
    break;
   case VaporOnly:
   case LiquidOnly:
    lastError="Single phase flashes with vapor fraction specification are not supported";
    return false;
   default:
    lastError="Invalid/unsupported flashPhaseType argument";
    return false;
  }
 if ((VF==0)||(VF==1.0)||(flashCompounds.size()==1)) return TVFFlash(T,VF,P); //same as molar phase fraction
 //find P so that VF is ok by solving TP flash
 double Pdew=DewPointPressure();
 double Pbub=BubblePointPressure();
 Tflash=T;
 VFflash=VF;
 Solver1Dim solver(TVFmFlashFunc,Pdew,Pbub,this,1e-4);
 if (!solver.Solve(P,lastError)) 
  {lastError="TVF flash solution failed: "+lastError;
   return false;
  }
 //results are already filled in by TP flash
 return true;
}

//! Target function for solving PVFm flash problem
/*!
  Target function for solving PVFm flash problem; solves the TP
  flash and returns massVapFrac-flashVF  
  \param param Parameter passed to solver constructor: PropertyPackage
  \param X Degree of freedom solved for: temperature
  \param F Receives the function value at X
  \param error Receives the error description in case of failure
  \return True if ok
  \sa PVFFlash(), Solver1Dim, MassVapFrac()
*/

bool PVFmFlashFunc(void *param,double X,double &F,string &error)
{PropertyPackage *pp=(PropertyPackage *)param;
 if (!pp->TPFlash(X,pp->Pflash)) 
  {error=pp->lastError;
   return false;
  }
 F=pp->MassVapFrac()-pp->VFflash;
 return true;
}

//! Calculate PVF phase equilibrium
/*!
  Internal routine to calculate PVF equilibrium
  \param P Pressure [Pa]
  \param VF Vapor phase fraction [kg/kg]
  \param T Receives equilibrium temperature [K]
  \return True if ok
  \sa Flash(), PVFmFlashFunc()
*/

bool PropertyPackage::PVFmFlash(double P,double VF,double &T)
{int i;
 if (!CheckPressure(P)) return false;
 if (!CheckVaporPhaseFraction(VF)) return false;
 switch (flashPhaseType)
  {case VaporLiquid:
    break;
   case VaporOnly:
   case LiquidOnly:
    lastError="Single phase flashes with vapor fraction specification are not supported";
    return false;
   default:
    lastError="Invalid/unsupported flashPhaseType argument";
    return false;
  }
 if ((VF==0)||(VF==1.0)||(flashCompounds.size()==1)) return TVFFlash(T,VF,P); //same as molar phase fraction
 //determine Tmax = min(TC)
 double Tmax=compounds[flashCompounds[0]]->TC;
 for (i=1;i<(int)flashCompounds.size();i++) if (compounds[flashCompounds[i]]->TC<Tmax) Tmax=compounds[flashCompounds[i]]->TC;
 //find T so that VF is ok by solving TP flash
 double Tbub,Tdew;
 Solver1Dim solverTbub(TbubFlashFunc,50.0,Tmax,this,1e-4);
 if (!solverTbub.Solve(Tbub,lastError)) 
  {lastError="PVF flash solution failed: "+lastError;
   return false; 
  }
 Solver1Dim solverTdew(TdewFlashFunc,50.0,Tmax,this,1e-4);
 if (!solverTdew.Solve(Tdew,lastError)) 
  {lastError="PVF flash solution failed: "+lastError;
   return false; 
  }
 VFflash=VF;
 Solver1Dim solver(PVFmFlashFunc,Tbub,Tdew,this,1e-4);
 if (!solver.Solve(T,lastError)) 
  {lastError="PVF flash solution failed: "+lastError;
   return false;
  }
 //results are already filled in by TP flash
 return true;
}

//! Target function for solving PH flash problem
/*!
  Target function for solving PH flash problem.  
  \param param Parameter passed to solver constructor: PropertyPackage
  \param X Degree of freedom solved for: temperature
  \param F Receives the function value at X
  \param error Receives the error description in case of failure
  \return True if ok
  \sa PHFlash(), Solver1Dim
*/

bool PHFlashFunc(void *param,double X,double &F,string &error)
{PropertyPackage *pp=(PropertyPackage *)param;
 if (!pp->TPFlash(X,pp->Pflash)) 
  {error=pp->lastError;
   return false;
  }
 F=-pp->Hflash;
 SinglePhaseProperty prop=Enthalpy;
 int *valueCount;
 double **values;
 if (pp->vaporExists)
  {if (!pp->GetSinglePhaseProperties((int)pp->flashCompounds.size(),VECPTR(pp->flashCompounds),Vapor,X,pp->Pflash,VECPTR(pp->vapX),1,&prop,valueCount,values))
    {error="Vapor enthalpy calculation failed: "+error;
     return false;
    }
   F+=pp->vapFrac*values[0][0];
  }
 if (pp->liquidExists)
  {if (!pp->GetSinglePhaseProperties((int)pp->flashCompounds.size(),VECPTR(pp->flashCompounds),Liquid,X,pp->Pflash,VECPTR(pp->liqX),1,&prop,valueCount,values))
    {error="Liquid enthalpy calculation failed: "+error;
     return false;
    }
   F+=pp->liqFrac*values[0][0];
  }
 return true;
}

//! Calculate PH phase equilibrium
/*!
  Internal routine to calculate PH equilibrium. Solves TP flash and calculates H to 
  find T for which H = Hspec. Allowed range is 50 < T < min(TC)
  \param P Pressure [Pa]
  \param H Enthalpy [J/mol]
  \param T Receives equilibrium temperature [K]
  \return True if ok
  \sa Flash(), PHFlashFunc()
*/

bool PropertyPackage::PHFlash(double P,double H,double &T)
{int i;
 if (!CheckPressure(P)) return false;
 if (!CheckEnthalpy(H)) return false;
 //determine Tmax = min(TC)
 double Tmax=compounds[flashCompounds[0]]->TC;
 for (i=1;i<(int)flashCompounds.size();i++) if (compounds[flashCompounds[i]]->TC<Tmax) Tmax=compounds[flashCompounds[i]]->TC;
 //find T so that VF is ok by solving TP flash
 Hflash=H;
 Pflash=P;
 Solver1Dim solver(PHFlashFunc,50,Tmax,this,1e-4);
 if (!solver.Solve(T,lastError)) 
  {lastError="PH flash solution failed: "+lastError;
   return false;
  }
 //results are already filled in by TP flash
 return true;
}

//! Target function for solving PS flash problem
/*!
  Target function for solving PS flash problem.  
  \param param Parameter passed to solver constructor: PropertyPackage
  \param X Degree of freedom solved for: temperature
  \param F Receives the function value at X
  \param error Receives the error description in case of failure
  \return True if ok
  \sa PSFlash(), Solver1Dim
*/

bool PSFlashFunc(void *param,double X,double &F,string &error)
{PropertyPackage *pp=(PropertyPackage *)param;
 if (!pp->TPFlash(X,pp->Pflash)) 
  {error=pp->lastError;
   return false;
  }
 F=-pp->Sflash;
 SinglePhaseProperty prop=Entropy;
 int *valueCount;
 double **values;
 if (pp->vaporExists)
  {if (!pp->GetSinglePhaseProperties((int)pp->flashCompounds.size(),VECPTR(pp->flashCompounds),Vapor,X,pp->Pflash,VECPTR(pp->vapX),1,&prop,valueCount,values))
    {error="Vapor entropy calculation failed: "+error;
     return false;
    }
   F+=pp->vapFrac*values[0][0];
  }
 if (pp->liquidExists)
  {if (!pp->GetSinglePhaseProperties((int)pp->flashCompounds.size(),VECPTR(pp->flashCompounds),Liquid,X,pp->Pflash,VECPTR(pp->liqX),1,&prop,valueCount,values))
    {error="Liquid entropy calculation failed: "+error;
     return false;
    }
   F+=pp->liqFrac*values[0][0];
  }
 return true;
}

//! Calculate PS phase equilibrium
/*!
  Internal routine to calculate PS equilibrium. Solves TP flash and calculates S to 
  find T for which S = Sspec. Allowed range is 50 < T < min(TC)
  \param P Pressure [Pa]
  \param S Entropy [J/mol/K]
  \param T Receives equilibrium temperature [K]
  \return True if ok
  \sa Flash(), PSFlashFunc()
*/

bool PropertyPackage::PSFlash(double P,double S,double &T)
{int i;
 if (!CheckPressure(P)) return false;
 if (!CheckEntropy(S)) return false;
 //determine Tmax = min(TC)
 double Tmax=compounds[flashCompounds[0]]->TC;
 for (i=1;i<(int)flashCompounds.size();i++) if (compounds[flashCompounds[i]]->TC<Tmax) Tmax=compounds[flashCompounds[i]]->TC;
 //find T so that VF is ok by solving TP flash
 Sflash=S;
 Pflash=P;
 Solver1Dim solver(PSFlashFunc,50,Tmax,this,1e-4);
 if (!solver.Solve(T,lastError)) 
  {lastError="PS flash solution failed: "+lastError;
   return false;
  }
 //results are already filled in by TP flash
 return true;
}

//! Edit the property package
/*!
  Edit the property package
  \return True if the changes are accepted, False in case the user cancels
*/

bool PropertyPackage::Edit()
{PackageEditor editor(this);
 return editor.Edit();
}

//! Get Property Calculation Result
/*!
  This is merely a helper routine for exporting result to VB
  \param index Result index
  \param count Receives the number of values in the result
  \param vals Receives the result values
  \sa PPGetPropertyResult
*/

BOOL PropertyPackage::GetPropertyResult(int index,int &count,double *&vals)
{if ((index<0)||(index>=(int)valueCounts.size()))
  {lastError="Result index out of range";
   return FALSE;
  }
 count=valueCounts[index];
 vals=valuePointers[index];
 return TRUE;
}

//! Get Flash phase result
/*!
  This is merely a helper routine for exporting result to VB
  \param index Resulting phase index
  \param phase Receives the phase type
  \param phaseFrac Receives the phase fraction
  \param Xcount Receives the length of the composition array
  \param X Receives the composition array
  \sa PPFlashPhaseResult
*/

BOOL PropertyPackage::GetFlashPhase(int index,Phase &phase,double &phaseFrac,int &Xcount,double *&X)
{int phaseCount;
 Phase phases[2];
 //make phase list
 phaseCount=0;
 if (vaporExists) phases[phaseCount++]=Vapor;
 if (liquidExists) phases[phaseCount++]=Liquid;
 if ((index<0)||(index>=phaseCount))
  {lastError="Result index out of range";
   return FALSE;
  }
 phase=phases[index];
 phaseFrac=(phase==Vapor)?vapFrac:liqFrac;
 vector<double> *composition=(phase==Vapor)?&vapX:&liqX;
 Xcount=(int)composition->size();
 X=VECPTR(*composition);
 return TRUE;
}

//! Get Flash phase type
/*!
  This is merely a helper routine for exporting result to VB
  \param index Resulting phase index
  \param phase Receives the phase type
  \sa PPFlashPhase
*/

BOOL PropertyPackage::GetFlashPhaseType(int index,Phase &phase)
{int phaseCount;
 Phase phases[2];
 //make phase list
 phaseCount=0;
 if (vaporExists) phases[phaseCount++]=Vapor;
 if (liquidExists) phases[phaseCount++]=Liquid;
 if ((index<0)||(index>=phaseCount))
  {lastError="Result index out of range";
   return FALSE;
  }
 phase=phases[index];
 return TRUE;
}
