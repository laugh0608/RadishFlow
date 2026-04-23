#pragma once

//! MaterialObjectWrapper class
/*!
  This is a wrapper class that defines an interface to the functionality of
  either a version 1.0 or version 1.1 material object, without the caller
  having to be concerned with it. Most functions in this class are 
  abstract, containing no implementation of any functionality. 
  
  The functionality for a CAPE-OPEN version 1.0 material object is implemented
  by the MaterialObject10Wrapper class.
  
  The functionality for a CAPE-OPEN version 1.1 material object is implemented
  by the MaterialObject11Wrapper class.
  
  Do not create this object directly; this class is managed via the Material
  class.
  
  \sa Material, MaterialObject10Wrapper, MaterialObject11Wrapper
  
*/

class MaterialObjectWrapper
{   protected:

	int refCount; /*!<  reference count; class will get destroyed if reference count hits zero */

	//this class can call all functions
	friend class Material;

	//! Constructor.
    /*!
      Sets reference count to 1
    */
    
    MaterialObjectWrapper()
    {refCount=1;
    }

	//! Destructor.
    /*!
      Does nothing. Is virtual, so that calling delete on a MaterialObjectWrapper properly calls the destuctor of the derived class
    */
  
    virtual ~MaterialObjectWrapper() 
     {
     }

	//! increases the reference count.
    /*!
      Increases the reference count. Reference count is one at construction. Class will get destroyed if reference count hits zero.
      \sa Release(), MaterialObjectWrapper()
    */
  
    void AddRef() 
     {refCount++;
     }

	//! decreases the refernce count.
    /*!
      Decreases the reference count. Reference count is one at construction. Class will get destroyed if reference count hits zero.
      \sa AddRef(), MaterialObjectWrapper()
    */

    void Release()
    {refCount--;
     if (refCount==0) delete this;
    }

	//! Get a duplicate material
    /*!
      Get a duplicate material. Material objects connected to feed ports
      we cannot alter. This means we cannot perform calculations on such 
      materials. We can however perform calculations on duplicates thereof.
      \param error receives textual error message upon failure
      \return a MaterialObjectWrapper in case of success, NULL in case of failure
    */
    
    virtual MaterialObjectWrapper *Duplicate(wstring &error)=0;

	//! Return list of compound IDs
    /*!
      Get the list of compound IDs on this material object. 
      \param list the list of compound IDs
      \param error error description in case of failure
      \return true in case of success
    */

    virtual bool GetCompoundIDs(CVariant &list,wstring &error)=0;
    
	//! Return list of single phase properties
    /*!
      Get the list of single phase properties supported by the material object
      \param list the list of properties
      \param error error description in case of failure
      \return true in case of success
    */
    
    virtual bool GetSinglePhasePropList(CVariant &list,wstring &error)=0;

	//! Get value of an overall property
    /*!
      Get the list of single phase properties
      \param propName property identifier
      \param basis property basis, can be NULL
      \param value receive the property value(s)
      \param error error description in case of failure
      \return true in case of success
    */

    virtual bool GetOverallProperty(const OLECHAR *propName,const OLECHAR *basis,CVariant &value,wstring &error)=0;

	//! Get list of present phases
    /*!
      Get the list of phases currently present on the material object
      requested via this function
      \param list receives the list of present phases
      \param error error description in case of failure
      \return true in case of success
    */
    
    virtual bool GetListOfPresentPhases(CVariant &list,wstring &error)=0;

	//! Calculate a single phase property
    /*!
      Calculate a single phase property for a given phase. It is assumed that only mixture properties will
      be calculated. Unit operation implementations that calculate multiple properties at the same 
      conditions should modify this function to allow for multiple property calculations in a single call.
      \param propName name of the property to calculate
      \param phaseName phase for which to calculate the property
      \param error error description in case of failure
      \return true in case of success
    */
    
    virtual bool CalcSinglePhaseProperty(const OLECHAR *propName,const OLECHAR *phaseName,wstring &error)=0;

	//! Get value of a single-phase property
    /*!
      Obtain value(s) a property of a given phase; it is assumed that no mixture or pure properties are 
      requested via this function
      \param propName property identifier
      \param phaseName phase for which to get the property
      \param calcType: mixture for mixture properties or NULL for fraction or phaseFraction. Ignored for version 1.1 thermo.
      \param basis property basis, can be NULL
      \param value receive the property value(s)
      \param error error description in case of failure
      \return true in case of success
    */
    
    virtual bool GetSinglePhaseProperty(const OLECHAR *propName,const OLECHAR *phaseName,const OLECHAR *calcType,const OLECHAR *basis,CVariant &value,wstring &error)=0;

	//! Get temperature from a PH flash at given P, H and composition
    /*!
      Calculate and return the temperature corresponding to a mixture at 
      given composition, enthalpy and pressure
      \param composition overall composition [mol/mol]
      \param P pressure [Pa]
      \param H enthalpy [J/mol]
      \param T receives temperature [K]
      \param error error description in case of failure
      \return true in case of success
    */
    
    virtual bool GetTemperatureFromPHFlash(CVariant &composition,double P,double H,double &T,wstring &error)=0;

	//! Specify a material object using composition, T and P
    /*!
      Specify a material object using total flow, composition, T and P. Perform a TP flash to complete the specification.
      \param composition overall composition [mol/mol]
      \param flow total flow [mol/s]
      \param P pressure [Pa]
      \param T temperature [K]
      \param error error description in case of failure
      \return true in case of success
    */
    
    virtual bool SetFromFlowTPX(CVariant &composition,double flow,double T,double P,wstring &error)=0;

};

