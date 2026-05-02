#pragma once

//! Antoine class
/*!
	Antoine equation class for temperature dependent vapor pressure. 
	
	Psat = 10^(A - B/(C+T))
	
	where T is in K and vapor pressure in Pa
	
	This particular implementation is not aware of limits of validity of the correlation.
	
	\sa Compound, Correlation
*/

class Antoine
{private:

	double A,B,C; /*!< correlation coefficients */
	double Bln10; /*!< constant */

 public:


	//! Constructor
	/*!
	  Gets called upon creation of the correlation object
	  \param A Antoine coefficient
	  \param B Antoine coefficient
	  \param C Antoine coefficient
	*/

	Antoine(double A,double B,double C)
	{this->A=A;
	 this->B=B;
	 this->C=C;
	 Bln10=B*log(10.0);
	}
	
	//! Value
	/*!
	  Gets vapor pressure at specific temperature
	  \param T Temperature / K
	  \return Vapore pressure / Pa
	*/
	
	double Value(double T)
	{return pow(10,A-B/(C+T));
	}

	//! ValueDT
	/*!
	  Gets temperature derivative of vapor pressure at specified temperature
	  \param T Temperature / K
	  \return Temperature derivative of vapor pressure / Pa/K
	*/
	
	double ValueDT(double T)
	{double d=C+T;
	 return Value(T)*Bln10/(d*d);
	}
};
