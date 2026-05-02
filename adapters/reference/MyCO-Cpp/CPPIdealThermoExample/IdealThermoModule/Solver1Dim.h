#pragma once

//! Func1Dim function type definition
/*!

    Signature for functions passed to Solver1Dim

	\param param Parameter passed to Solver1Dim constructor
	\param X Value of degree of freedom solved for
	\param F Receives the value of function for which to find zero
	\param error Receives error text in case of failure
	\return True if ok
    
    \sa Solver1Dim()
    
*/

typedef bool (*Func1Dim)(void *param,double X,double &F,string &error);

//! Solver1Dim class
/*!

    Class for solving non-linear 1-dimensional problems
    
    This is a simple solver solver that requires bracketing the 
    solution. The solution will then be found by reducing the 
    bracketed region based upon 1st order information (without
    the requirement to evaluate the derivative).
    
    The solver presumes the function to be solved is 
    monotonically increasing or decreasing
    
*/

class Solver1Dim
{private:
 double Xlo;	/*!< lower limit of bracketed region */
 double Xhi;	/*!< upper limit of bracketed region */
 double X;	    /*!< current value and solution */
 Func1Dim func; /*!< function to be solved */
 double tol;    /*!< required tolerance */
 void *param;   /*!< parameter passed to func */

 public:

 //! Constructor
 /*!
  Called upon construction of a LinearSolver instance
  \param func Function to be solved
  \param Xlo Lower limit of bracketed region
  \param Xhi Upper limit of bracketed region
  \param param Parameter passed to func
  \param tol Required abs value of F at solution, optional (defaults to 1e-8)
 */

 Solver1Dim(Func1Dim func,double Xlo,double Xhi,void *param,double tol=1e-8)
 {this->func=func;
  this->Xlo=Xlo;
  this->Xhi=Xhi;
  this->param=param;
  this->tol=tol;
 }
 
 //! Solve
 /*!
  Solve the function
  \param solution Receives the solution
  \param error Receives the error in case of failure
  \return True if ok
 */

 bool Solve(double &solution,string &error)
 {double Flo;
  double Fhi;
  double frac;
  double F;
  bool increasing;
  bool goUp;
  if (!(*func)(param,Xlo,Flo,error)) return false;
  if (fabs(Flo)<tol) {solution=Xlo;return true;}
  if (!(*func)(param,Xhi,Fhi,error)) return false;
  if (fabs(Fhi)<tol) {solution=Xhi;return true;}
  if (Flo*Fhi>0) 
   {error="Allowed region does not contain solution";
    return false;
   }
  increasing=(Fhi>Flo);
  for (;;)
  {//linear interpolation to find zero
   frac=1.0-Fhi/(Fhi-Flo);
   //limit to reasonable setp
   if (frac<1e-3) frac=1e-3; else if (frac>0.999) frac=0.999;
   X=Xlo+frac*(Xhi-Xlo);
   if ((X==Xlo)||(X==Xhi)) X=0.5*(Xhi+Xlo);
   if ((X==Xlo)||(X==Xhi)) {solution=X;return true;} //converged up to machine precision
   if (!(*func)(param,X,F,error)) return false;
   //check convergence
   if (fabs(F)<tol) {solution=X;return true;}
   //check direction   
   goUp=(F<0);
   if (!increasing) goUp=!goUp;
   if (goUp)
    {Xlo=X;
     Flo=F;
    }
   else
    {Xhi=X;
     Fhi=F;
    }
  }
 }
 
};
