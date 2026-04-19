#pragma once

#include "Correlation.h"
#include "Antoine.h"

//! Compound class
/*!

    This class holds all data for a particular compound in a Property Package.
	
	\sa PropertyPack
  
*/

class Compound
{
public:

	string name; /*!< compound name */
	string formula; /*!< chemical formula */
	string CAS; /*!< CAS number */
	double MW; /*!< Relative molecular weight */
	double NBP; /*!< Normal boiling point / K */
	double TC; /*!< Critical temperature / K */
	double PC; /*!< Critical pressure / Pa */
	double VC; /*!< Critical volume / m3/mol */
	Correlation *CpCorrelation; /*!< Ideal gas heat capacity correlation / J/mol/K */
	Correlation *HvapCorrelation; /*!< Heat of vaporization correlation / J/mol */
	Correlation *liqDensCorrelation; /*!< liquid density correlation / mol/m3 */
	Antoine *pSatCorrelation; /*!< Saturated Vapor Pressure correlation / Pa */	

	//! Constructor
	/*!
	  Gets called upon creation of the Compound object
	  
	  \sa Load()
	*/

	Compound()
	{CpCorrelation=NULL;
	 HvapCorrelation=NULL;
	 liqDensCorrelation=NULL;
	 pSatCorrelation=NULL; 
	}
	
	//! Destructor
	/*!
	  Gets called upon destruction of the Compound object
	*/

	~Compound()
	{if (CpCorrelation) delete CpCorrelation;
	 if (HvapCorrelation) delete HvapCorrelation;
	 if (liqDensCorrelation) delete liqDensCorrelation;
	 if (pSatCorrelation) delete pSatCorrelation;
	}
	
	//member functions 
	bool Load(const char *compName,std::string &error);



};
