#pragma once

//! wrapper class for BSTR values
/*!

This class is for BSTR data manipilation. The main data type is a BSTR.

The BSTR is cleaned up at the destruction of this class.

Please take great care of knowing who owns the content of a BSTR and therefore
who is responsible for freeing the content; this shows from [in], [in, out] and
[out, retval] annotations in the IDL

Implementations in this class take into account that an empty string should 
be represented by NULL

*/

class CBSTR
{
 
 private:
 
 BSTR value; /*!< the value contained in this class */

 public: 
 
 //! Constructor.
 /*!
   Constructor that creates an empty value
 */

 CBSTR() 
  {value=NULL;
  }

 //! Copy constructor.
 /*!
   Constructor used for assignment 
 */

 CBSTR(const CBSTR &other) 
  {if (&other!=this)
    value=SysAllocString(other.value);
  }

 //! Constructor.
 /*!
   Constructor that allocated a BSTR value given a string value
 */

 CBSTR(const OLECHAR *value) 
  {this->value=NULL;
   if (value)
    if (*value)
     this->value=SysAllocString(value);
  }

 //! Destructor.
 /*!
   Cleans up the value, if any
 */
 
 ~CBSTR()
 {if (value) SysFreeString(value);
 }

 //! Set from a BSTR value
 /*!
   Set from a valid BSTR value; this class will own the BSTR value afterwards and the caller should not destroy it!
 */
 
 void SetFromBSTR(BSTR b)
 {if (value) SysFreeString(value);
  value=b;
 }
 
 //! Assignment from other CBSTR
 /*!
   Set from a CBSTR value; the value will be copied
 */

 void operator=(const CBSTR other)
 {if (&other!=this)
   {if (value) SysFreeString(value);
    value=NULL;
    if (other.value)
     if (*other.value)
      value=SysAllocString(other.value);
   }
 }

 //! Set from a string value
 /*!
   Set from a string value; the value will be allocated and owned by this object
 */

 void operator=(const OLECHAR *str)
 {if (value) SysFreeString(value);
  value=NULL;
  if (str)
   if (*str)
    value=SysAllocString(str);
 }
 
 //! String comparison
 /*!
   Returns true if two BSTR values or string values are equivalent. Takes special care of assuming NULL denoting an empty string.
 */
  
 static BOOL Same(const OLECHAR *str1,const OLECHAR *str2)
 {BOOL res=FALSE;
  if (str1)
   {if (str2) res=(lstrcmpiW(str1,str2)==0);
    else if (*str1==0) res=TRUE;
   }
  else 
   if (!str2) res=TRUE;
   else if (*str2==0) res=TRUE;
  return res;
 }

 //! Length 
 /*!
   Returns the length of the content
 */
 
 int Length()
 {return (value)?lstrlenW(value):0;
 }

 //! cast to BSTR value
 /*!
   this object can be used directly as BSTR (as BSTR is defined as OLECHAR *, therefore also as OLECHAR *)
 */
 
 operator BSTR() 
  {return value;}

 
};

