#pragma once

#include "BSTR.h"

//! wrapper class for VARIANT values
/*!
This class is for Variant data manipilation. The main data type is a VARIANT.

The VARIANT is cleaned up at the destruction of this class, unless it is 
detected that the VARIANT is owned by somebody else. This is when the class
is created from an input variable, or when a return value is requested

For performance, the class is completely inline.

Please take great care of knowing who owns the content of a Variant and therefore
who is responsible for freeing the content; this shows from [in], [in, out] and
[out, retval] annotations in the IDL
*/

class CVariant
{

private:

   VARIANT value; /*!< the value contained in this class */
   BOOL mustDelete; /*!< keeps track of whether the value data is owned by this class */
   LONG count; /*!< number of elements in an array */
   #ifdef _DEBUG
   VARTYPE elementType; /*!< type of array elements */
   #endif

public:

	//! Constructor.
    /*!
      Default constructor; assumes we own the data
    */

   CVariant()
    {VariantInit(&value);   
     mustDelete=TRUE;
     count=0;
     #ifdef _DEBUG
     elementType=VT_EMPTY;
     #endif
    }

	//! Constructor.
    /*!
      Constructor using an existing VARIANT value; we need to know whether we own the data
      \param value initial value
      \param DestroyWhenDone set to true if we own the data
    */

    CVariant(VARIANT value,BOOL DestroyWhenDone)
     {mustDelete=DestroyWhenDone;
      this->value=value;
      count=0;
      #ifdef _DEBUG
      elementType=VT_EMPTY; //not checked
      #endif
     }

	//! Copy constructor.
    /*!
      Copy constructor, makes a copy of the value
      \param orig value to copy
    */

    CVariant(CVariant &orig)
     {mustDelete=TRUE;
      VariantCopy(&value,&orig.value);
     }
    
	//! Destructor.
    /*!
      Cleans up if we own the data
    */
    
    ~CVariant()
     {//destructor
      if (mustDelete) VariantClear(&value);
     }

	//! Create an array.
    /*!
      Create an array of elements of the specified type with specified count and lower bound of zero
      \param count number of elements in the array
      \param vt type of the elements
    */
    
    void MakeArray(int count,VARTYPE vt)
     {SAFEARRAYBOUND ba;
      //clean up old content
      if (mustDelete) 
       if (value.vt!=VT_EMPTY) 
        VariantClear(&value);
      //set new content
      mustDelete=TRUE;
      if (count>0)
       {//we have a count
        ba.cElements=count;
        ba.lLbound=0;
        value.parray=SafeArrayCreate(vt,1,&ba);
        value.vt=VT_ARRAY|vt;
        //keep track of content
        #ifdef _DEBUG
        elementType=vt;
        #endif
       }
      else ATLASSERT(value.vt==VT_EMPTY); //an empty array is an empty VARIANT
      this->count=count;
     }

	//! Value 
    /*!
      returns a VARIANT structure with the current value. The content will be destroyed after calling this
      function if the CVariant was marked as such; this function should be used to assign values 
      in case we maintain ownershop of the allocated value
    */
    
    VARIANT Value() 
     {return value;
     }

	//! Value for return values
    /*!
      returns a VARIANT structure with the current value. The content will not be destroyed after calling this
      function; this function should be used to assign values to [out, retval] parameter or [in, out] parameter (in the
      latter case the value that was in the parameter before should be destroyed first)
    */
    
    VARIANT ReturnValue() 
     {mustDelete=FALSE;
      return value;
     }

	//! Value for non-return values
    /*!
      returns a VARIANT structure with the current value. The content will still be destroyed after calling this
      function; this function should be to pass values to other functions, when we still own the value after 
      returning. Such as setting values with SetSinglePhaseProp. This function is implement as cast operator
    */

    operator VARIANT() 
     {return value;
     }

	//! Copy of the value 
    /*!
      return a copy of the current value (destorying is responsibility of caller)
    */

    VARIANT Copy()
     {VARIANT v;
      v.vt=VT_EMPTY;
      VariantCopy(&v,&value);
      return v;
     }

	//! Number of elements in an array
    /*!
      returns the number of elements in an array value
    */

    int GetCount() 
     {ATLASSERT(count>=0);
      return count;
     }

	//! Set value to something else
    /*!
     function to reset the content to something else (re-use a CVariant object) 
     \param value new value
     \param DestroyWhenDone indication on whether we own the value
    */

    void Set(VARIANT value,BOOL DestroyWhenDone)
    {//clean up old content
     if (mustDelete) 
      if (this->value.vt!=VT_EMPTY) 
       VariantClear(&this->value);
     //set to existing VARIANT
     mustDelete=DestroyWhenDone;
     this->value=value;
     count=0;
     #ifdef _DEBUG
     elementType=VT_EMPTY; //not checked
     #endif
    }
    
	//! Set value to real
    /*!
     function to reset the content to a real value
     \param val new value
    */

    void Set(double val)
    {//clean up old content
     if (mustDelete) 
      if (this->value.vt!=VT_EMPTY) 
       VariantClear(&this->value);
     //set to real
     value.vt=VT_R8;
     mustDelete=FALSE; //no need to, no associated memory
     value.dblVal=val;
    }

	//! Set value to string 
    /*!
     function to reset the content to a string value (that is allocated)
     \param val new value
    */

    void Set(const OLECHAR *val)
    {//clean up old content
     if (mustDelete) 
      if (this->value.vt!=VT_EMPTY) 
       VariantClear(&this->value);
     //set to string
     value.vt=VT_BSTR;
     mustDelete=TRUE; //the current Variant owns this value
     value.bstrVal=SysAllocString(val);
    }

	//! Set value to string 
    /*!
     function to reset the content to a string value (that is converted and allocated)
     \param val new value
    */

    void Set(const char *val)
    {//clean up old content
     if (mustDelete) 
      if (this->value.vt!=VT_EMPTY) 
       VariantClear(&this->value);
     //set to string
     value.vt=VT_BSTR;
     mustDelete=TRUE; //the current Variant owns this value
     value.bstrVal=SysAllocString(CA2CT(val));
    }

	//! Clear the value
    /*!
     function to clear the content 
    */
    
    void Clear()
     {//clean up old content
      if (mustDelete) 
       if (value.vt!=VT_EMPTY) 
        VariantClear(&value);
      value.vt=VT_EMPTY;
      count=0;
      mustDelete=TRUE;
     }

	//! Access elements of boolean arrays
    /*!
     function to set the elements of boolean arrays. 
     \param index should be between 0 and count-1, inclusive
     \param b new value for element at specified index
    */
    
    void SetBoolAt(LONG index,VARIANT_BOOL b)
     {//make sure to have called MakeArray before this
      ATLASSERT((count>0)&&(index<count));
      ATLASSERT(elementType==VT_BOOL);
      SafeArrayPutElement(value.parray,&index,&b);
     }

	//! Access elements of VARIANT arrays
    /*!
     function to set the elements of VARIANT arrays. 
     \param index should be between 0 and count-1, inclusive
     \param v new value for element at specified index
    */
    
    void SetVariantAt(LONG index,VARIANT v)
     {//make sure to have called MakeArray before this
      ATLASSERT((count>0)&&(index<count));
      ATLASSERT(elementType==VT_VARIANT);
      SafeArrayPutElement(value.parray,&index,&v);
     }

	//! Access elements of real arrays
    /*!
     function to set the elements of real arrays. 
     \param index should be between 0 and count-1, inclusive
     \param d new value for element at specified index
    */
    
    void SetDoubleAt(LONG index,double d)
     {//make sure to have called MakeArray before this
      ATLASSERT((count>0)&&(index<count));
      ATLASSERT(elementType==VT_R8);
      SafeArrayPutElement(value.parray,&index,&d);
     }

	//! Access elements of real arrays
    /*!
     function to get the elements of real arrays. 
     \param index should be between 0 and count-1, inclusive
     \return value for element at specified index
    */
     
    double GetDoubleAt(LONG index)
     {//make sure CheckArray was called before this
      double d;
      ATLASSERT((count>0)&&(index<count));
      ATLASSERT(elementType==VT_R8);
      SafeArrayGetElement(value.parray,&index,&d);
      return d;
     }

	//! Access elements of string arrays
    /*!
     function to set the elements of string arrays. 
     \param index should be between 0 and count-1, inclusive
     \param b new value for element at specified index; must be a valid BSTR value
    */

    void SetStringAt(LONG index,BSTR b)
     {//make sure to have called MakeArray before this
      ATLASSERT((count>0)&&(index<count));
      ATLASSERT(elementType==VT_BSTR);
      SafeArrayPutElement(value.parray,&index,b);
     }

	//! Access elements of string arrays
    /*!
     function to set the elements of string arrays. 
     \param index should be between 0 and count-1, inclusive
     \param str new value for element at specified index; a BSTR value will be allocated from this value
    */

    void AllocStringAt(LONG index,const OLECHAR *str)
     {//make sure to have called MakeArray before this
      ATLASSERT((count>0)&&(index<count));
      ATLASSERT(elementType==VT_BSTR);
      BSTR b;
      if (str) b=SysAllocString(str);
      else b=NULL;
      SafeArrayPutElement(value.parray,&index,b);
      if (b) SysFreeString(b);
     }

	//! Access elements of string arrays
    /*!
     function to get the elements of string arrays. 
     \param index should be between 0 and count-1, inclusive
     \return value at specified index
    */

    CBSTR GetStringAt(LONG index)
     {//make sure CheckArray was called before this
      BSTR b;
      ATLASSERT((count>0)&&(index<count));
      ATLASSERT(elementType==VT_BSTR);
      SafeArrayGetElement(value.parray,&index,&b);
      CBSTR res;
      res.SetFromBSTR(b);  
      return res;
     }

	//! Access elements of string arrays
    /*!
     function to get the elements of string arrays. This version returns a
     BSTR, which should be freed by the caller
     \param index should be between 0 and count-1, inclusive
     \return BSTR value at specified index
    */

    BSTR GetBSTRAt(LONG index)
     {//make sure CheckArray was called before this
      BSTR b;
      ATLASSERT((count>0)&&(index<count));
      ATLASSERT(elementType==VT_BSTR);
      SafeArrayGetElement(value.parray,&index,&b);
      return b;
     }

	//! Access elements of integer arrays
    /*!
     function to set the elements of integer arrays. 
     \param index should be between 0 and count-1, inclusive
     \param i value at specified index
    */

    void SetLongAt(LONG index,LONG i)
     {//make sure to have called MakeArray before this
      ATLASSERT((count>0)&&(index<count));
      ATLASSERT(elementType==VT_I4);
      SafeArrayPutElement(value.parray,&index,&i);
     }

	//! Access elements of integer arrays
    /*!
     function to get the elements of integer arrays. 
     \param index should be between 0 and count-1, inclusive
     \return value at specified index
    */

    LONG GetLongAt(LONG index)
     {//make sure CheckArray was called before this
      LONG i;
      ATLASSERT((count>0)&&(index<count));
      ATLASSERT(elementType==VT_I4);
      SafeArrayGetElement(value.parray,&index,&i);
      return i;
     }

	//! Check an array
    /*!
     function to check a proper array and its data type. Also keeps track of the number of elements. It is 
     expected that all arrays have their lowest element at index 0
     \param vt expected data type of the elements
     \param error description of the error in case of a false return value
     \param expectedCount if non-zero, verify element count
     \return true if the value describes a proper array of the expected data type. In case of false, the error parameter will contain a description of the problem.
    */

    BOOL CheckArray(VARTYPE vt,wstring &error,int expectedCount=0)
     {//sets the low bound and count if check is 
      #ifdef _DEBUG
      elementType=vt;
      #endif
      // performed ok
      BOOL res=FALSE;
      //check type
      if (value.vt==VT_EMPTY)
       {//this is ok for an empty array
        res=TRUE;
        count=0;
       }
      else if (value.vt!=(vt|VT_ARRAY)) 
       {//not the type we expected
        error=L"Expected an array of type ";
        switch (vt)
         {case VT_EMPTY: error+=L"EMPTY"; ATLASSERT(0); break;
          case VT_R8: error+=L"double precision";break;
          case VT_VARIANT: error+=L"VARIANT";break;
          case VT_BSTR: error+=L"string";break;
          case VT_I4: error+=L"long";break;
          default: error+=L"<unknown>";ATLASSERT(0);break;
         }
       }
      else
       {//type is ok, check dimension
        if (SafeArrayGetDim(value.parray)!=1) error=L"unexpected dimension; 1-dimensional array expected";
        else
         {//get lower and upper bound:
          LONG lBound;
          if (FAILED(SafeArrayGetLBound(value.parray,1,&lBound))) error=L"failed to get lower bound";
          else if (lBound!=0) error=L"unexpected lower bound; expected zero";
          else if (FAILED(SafeArrayGetUBound(value.parray,1,&count))) error=L"failed to get upper bound";
          else
           {//all ok
            count+=1; //lower bound is at zero, upper bound at count-1
            res=TRUE;
           }
         }
       }
      if (res)
       if (expectedCount)
        {//check count
         if (count!=expectedCount)
          {res=false;
           if (expectedCount==1) error=L"scalar expected";
           else error=L"unexpected number of values";
          }
        }
      return res;
     }

};


