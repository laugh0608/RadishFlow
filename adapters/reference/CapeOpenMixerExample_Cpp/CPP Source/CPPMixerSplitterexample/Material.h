#pragma once
#include "MaterialObject10Wrapper.h"
#include "MaterialObject11Wrapper.h"

//! Material class
/*!
  This is a wrapper class that defines an interface to the functionality of
  either a version 1.0 or version 1.1 material object, without the caller
  having to be concerned with it. 
  
  
  This class uses the MaterialObjectWrapper class; in order for the 
  called not to be concerned with its clean-up, this class will make sure the 
  derived class gets cleaned up properly.
  
  The functionality for a CAPE-OPEN version 1.0 material object is implemented
  by the MaterialObject10Wrapper class.
  
  The functionality for a CAPE-OPEN version 1.1 material object is implemented
  by the MaterialObject11Wrapper class.
  
  Do not create this object directly; it is obtained via either 
  MaterialPort::GetMaterial() or Material::Duplicate().
  
  \sa MaterialObjectWrapper, MaterialObject10Wrapper, MaterialObject11Wrapper
  
*/

class Material
{	public:

    MaterialObjectWrapper *materialObject; /*!< the material object wrapped by this class */


	//! Constructor
    /*!
      Initializes the MaterialObjectWrapper to be nothing. Must be followed by call to SetMaterial10 or SetMaterial11
    */
    
    Material() 
    {materialObject=NULL;
    }

	//! Destructor
    /*!
      Releases the contained MaterialObjectWrapper
    */
    
    ~Material() 
    {if (materialObject) materialObject->Release();
    }

	//! Copy constructor
    /*!
      Used for assignment; the resulting class references the same material object as the existing class. 
      For a new material object, use Duplicate
      \sa Duplicate()
    */
    
    Material(Material &other) 
    {materialObject=other.materialObject;
     materialObject->AddRef();
    }

	//! Assignment
    /*!
      Used for assignment; the resulting class references the same material object as the existing class. 
      For a new material object, use Duplicate
      \sa Duplicate()
    */
    
    void operator= (Material other) 
    {//release current
     if (materialObject) materialObject->Release();
     materialObject=NULL;
     if (other.materialObject)
      {materialObject=other.materialObject;
       materialObject->AddRef();
      }
    }

	//! Check validity
    /*!
      A Material object must be properly initialized before you can use it. It must be obtained either
      from MaterialPort::GetMaterial of a connected material port, or created by Duplicate of a 
      valid Material. This utility function allows checking whether a Material is properly initialized
      \return true if properly initialized
    */
    
    bool IsValid()
    {return materialObject!=0;
    }


	private:
    
    friend class CMaterialPort;
    
	//! Initialization
    /*!
      Initializes the material object from a version 1.0 external material object
    */
    
    void SetMaterial10(ICapeThermoMaterialObject *mat10)
    {materialObject=new MaterialObject10Wrapper(mat10);
    }
    
	//! Initialization
    /*!
      Initializes the material object from a version 1.1 external material object
    */
    
    void SetMaterial11(ICapeThermoMaterial *mat11)
    {materialObject=new MaterialObject11Wrapper(mat11);
    }
    
    public:
    
	//! Get a duplicate material
    /*!
      Get a duplicate material. Material objects connected to feed ports
      we cannot alter. This means we cannot perform calculations on such 
      materials. We can however perform calculations on duplicates thereof.
      \param m receives duplicate material
      \param error receives textual error message upon failure
      \return true for success
    */

	bool Duplicate(Material &m,wstring &error)
	{ATLASSERT(materialObject); //class should be instanciated properly
	 //create a new material object
	 MaterialObjectWrapper *MO=materialObject->Duplicate(error);
	 if (!MO) return false; //fail
	 //clean up old MO in m
	 if (m.materialObject) m.materialObject->Release();
	 //set new
	 m.materialObject=MO;
	 //ok
	 return true;
	}
    
	//! Return list of compound IDs
    /*!
      Get the list of compound IDs on this material object. 
      \param list the list of compound IDs
      \param error error description in case of failure
      \return true in case of success
    */

    bool GetCompoundIDs(CVariant &list,wstring &error)
     {ATLASSERT(materialObject); //class should be instanciated properly
      return materialObject->GetCompoundIDs(list,error);
     }
    
	//! Return list of single phase properties
    /*!
      Get the list of single phase properties supported by the material object
      \param list the list of properties
      \param error error description in case of failure
      \return true in case of success
    */
    
    bool GetSinglePhasePropList(CVariant &list,wstring &error)
    {ATLASSERT(materialObject); //class should be instanciated properly
     return materialObject->GetSinglePhasePropList(list,error);
    }
        
	//! Get value of an overall property
    /*!
      Obtain value(s) an overall property; 
      \param propName property identifier
      \param basis property basis, can be NULL
      \param value receive the property value(s)
      \param error error description in case of failure
      \return true in case of success
    */

    bool GetOverallProperty(const OLECHAR *propName,const OLECHAR *basis,CVariant &value,wstring &error)
    {ATLASSERT(materialObject); //class should be instanciated properly
     return materialObject->GetOverallProperty(propName,basis,value,error);
    }
    
	//! Get list of present phases
    /*!
      Get the list of phases currently present on the material object
      requested via this function
      \param list receives the list of present phases
      \param error error description in case of failure
      \return true in case of success
    */
    
    bool GetListOfPresentPhases(CVariant &list,wstring &error)
    {ATLASSERT(materialObject); //class should be instanciated properly
     return materialObject->GetListOfPresentPhases(list,error);
    }
    
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
    
    bool CalcSinglePhaseProperty(const OLECHAR *propName,const OLECHAR *phaseName,wstring &error)
    {ATLASSERT(materialObject); //class should be instanciated properly
     return materialObject->CalcSinglePhaseProperty(propName,phaseName,error);
    }
    
	//! Get value of a single-phase property
    /*!
      Obtain value(s) a property of a given phase
      \param propName property identifier
      \param phaseName phase for which to get the property
      \param calcType: mixture for mixture properties or NULL for fraction or phaseFraction. Ignored for version 1.1 thermo.
      \param basis property basis, can be NULL
      \param value receive the property value(s)
      \param error error description in case of failure
      \return true in case of success
    */
    
    bool GetSinglePhaseProperty(const OLECHAR *propName,const OLECHAR *phaseName,const OLECHAR *calcType,const OLECHAR *basis,CVariant &value,wstring &error)
    {ATLASSERT(materialObject); //class should be instanciated properly
     return materialObject->GetSinglePhaseProperty(propName,phaseName,calcType,basis,value,error);
    }

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
    
    bool GetTemperatureFromPHFlash(CVariant &composition,double P,double H,double &T,wstring &error)
    {ATLASSERT(materialObject); //class should be instanciated properly
     return materialObject->GetTemperatureFromPHFlash(composition,P,H,T,error);
    }

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
    
    bool SetFromFlowTPX(CVariant &composition,double flow,double T,double P,wstring &error)
    {ATLASSERT(materialObject); //class should be instanciated properly
     return materialObject->SetFromFlowTPX(composition,flow,T,P,error);
    }

};

