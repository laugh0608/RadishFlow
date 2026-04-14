#pragma once

//! Correlation class
/*!
	Correlation class for temperature dependent pure-compound properties. 
	All correlations have the form
	
	prop = A + B * T + C * T^2 + D * T^3 + E * T^4
	
	where T is in K and the property is in SI units.
	
	This particular implementation is not aware of limits of validity of the correlation.
	
	\sa Compound, Antoine
*/

class Correlation
{private:

	double A,B,C,D,E; /*!< correlation coefficients */
	double twoC,threeD,fourE; /*!< used for differential */
	double halfB,thirdC,quarterD,fifthE; /*!< used for integrals */
	double halfC,thirdD,quarterE; /*!< used for integrals over T*/
	double intConstant,intConstantOverT; /*!< integration constants*/

 public:

	//! Constructor
	/*!
	  Gets called upon creation of the correlation object
	  \param A 0th order coefficient
	  \param B 1st order coefficient
	  \param C 2nd order coefficient
	  \param D 3rd order coefficient
	  \param E 4th order coefficient
	*/

	Correlation(double A,double B,double C,double D,double E)
	{this->A=A;
	 this->B=B;
	 this->C=C;
	 this->D=D;
	 this->E=E;
	 twoC=2.0*C;
	 threeD=3.0*D;
	 fourE=4.0*E;
	 halfB=0.5*B;
	 thirdC=C/3.0;
	 quarterD=0.25*D;
	 fifthE=0.2*E;
	 halfC=0.5*C;
	 thirdD=D/3.0;
	 quarterE=0.25*E;
	 intConstant=-REFERENCE_TEMPERATURE*(A+REFERENCE_TEMPERATURE*(halfB+REFERENCE_TEMPERATURE*(thirdC+REFERENCE_TEMPERATURE*(quarterD+REFERENCE_TEMPERATURE*fifthE))));
	 intConstantOverT=-REFERENCE_TEMPERATURE*(B+REFERENCE_TEMPERATURE*(halfC+REFERENCE_TEMPERATURE*(thirdD+REFERENCE_TEMPERATURE*quarterE)))-A*log(REFERENCE_TEMPERATURE);
	}
	
	//! Value
	/*!
	  Gets value at specific temperature
	  \param T temperature
	  \return Property value
	*/
	
	double Value(double T)
	{return A+T*(B+T*(C+T*(D+T*E)));
	}

	//! ValueDT
	/*!
	  Gets temperature derivative of value at specific temperature
	  \param T Temperature
	  \return Temperature derivative of property value
	*/
	
	double ValueDT(double T)
	{return B+T*(twoC+T*(threeD+T*fourE));
	}

	//! IntValue
	/*!
	  Gets the integral for the value from reference temperature to T
	  \param T Temperature
	  \return Integral from Tref to T of the value
	*/
	
	double IntValue(double T)
	{return T*(A+T*(halfB+T*(thirdC+T*(quarterD+T*fifthE))))+intConstant;
	}


	//! IntValueOverT
	/*!
	  Gets the integral for the value/T from reference temperature to T
	  \param T Temperature
	  \return Integral of value over T from Tref to T of the value
	*/

    double IntValueOverT(double T)
    {return A*log(T)+T*(B+T*(halfC+T*(thirdD+T*quarterE)))+intConstantOverT;
    }

};
