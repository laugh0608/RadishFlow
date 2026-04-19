#include "stdafx.h"
#include "Helpers.h"

//! Get error text from HRESULT code
/*!

  Get a descriptive error text from a HRESULT code  
  \param hr error code
  \return descriptive error text
  \sa CO_Error()
  
*/

wstring HResError(HRESULT hr)
{//use this routine to get a descriptive error text from just a HRESULT code
 wstring res;
 OLECHAR *buf,*ptr;
 buf=new OLECHAR[1024];
 if (!buf) res=L"<out of memory>";
 else
  {if (!FormatMessageW(FORMAT_MESSAGE_FROM_SYSTEM|FORMAT_MESSAGE_IGNORE_INSERTS,NULL,hr,MAKELANGID(LANG_NEUTRAL,SUBLANG_DEFAULT),buf,1024,NULL))
	{FormatMessageW(FORMAT_MESSAGE_ARGUMENT_ARRAY|FORMAT_MESSAGE_FROM_STRING,L"error code 0x%1!08X!",0,MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT),buf,1024,(va_list*)&hr);
	}
   ptr=buf;
   if (*ptr)
    {while (*ptr) ptr++;
     ptr--;
     if (*ptr==L'\n') 
  	  {*ptr=0;
	   if (ptr>buf)
	    {ptr--;
	     if (*ptr==L'\r') *ptr=0;
	    }
	  }
	}  
   res=buf;
   delete []buf;
  }
 if (res.empty()) res=L"<unknown error>";
 return res;
}

//! Get error text from CAPE-OPEN object
/*!

  Get a descriptive error text from a HRESULT code and a CAPE-OPEN object; uses CAPE-OPEN
  error interfaces if the HRESULT code is a CAPE-OPEN error code. Otherwise, uses HResError
  \param capeObject the CAPE-OPEN object that returned the error code
  \param hr error code
  \return descriptive error text
  \sa HResError()
  
*/

wstring CO_Error(IDispatch *capeObject,HRESULT hr)
{	//use this routine to get a descriptive error text from a HRESULT code and CAPE-OPEN interface that threw the error
    //get the standard CAPE-OPEN interface
	ECapeRoot *e;
	ECapeUser *ue;
	BSTR errName=NULL;
	wstring s;
	switch (hr)
	 {default: //non CAPE-OPEN
			s=HResError(hr);
			break;
	  case ECapeDataHR:
	  case ECapeLicenceErrorHR:
	  case ECapeBadCOParameterHR:
	  case ECapeBadArgumentHR:
	  case ECapeInvalidArgumentHR:
	  case ECapeOutOfBoundsHR:
	  case ECapeImplementationHR:
	  case ECapeNoImplHR:
	  case ECapeLimitedImplHR:
	  case ECapeComputationHR:
	  case ECapeOutOfResourcesHR:
	  case ECapeNoMemoryHR:
	  case ECapeTimeOutHR:
	  case ECapeFailedInitialisationHR:
	  case ECapeSolvingErrorHR:
	  case ECapeBadInvOrderHR:
	  case ECapeInvalidOperationHR:
	  case ECapePersistenceHR:
	  case ECapeIllegalAccessHR:
	  case ECapePersistenceNotFoundHR:
	  case ECapePersistenceSystemErrorHR:
	  case ECapePersistenceOverflowHR:
	  case ECapeOutsideSolverScopeHR:
	  case ECapeHessianInfoNotAvailableHR:
	  case ECapeThrmPropertyNotAvailableHR:
	  case ECapeUnknownHR:
		   //get the description from user;
		   if (FAILED(capeObject->QueryInterface(IID_ECapeUser,(LPVOID*)&ue))) 
			{//get description from root:
			 tryRoot:
			 if (FAILED(capeObject->QueryInterface(IID_ECapeRoot,(LPVOID*)&e))) goto defRemark;
			 else
			   {//get the cape error:
				e->get_name(&errName);
				if (errName) 
				if (*errName)
				 {s=errName;
				  SysFreeString(errName);
				 }
				else 
				 {defRemark:
				  switch (hr)
					 {case ECapeDataHR: s=L"invalid data";break;
					  case ECapeLicenceErrorHR: s=L"license problem";break;
					  case ECapeBadCOParameterHR: s=L"Bad Cape-Open parameter";break;
					  case ECapeBadArgumentHR: s=L"Bad argument";break;
					  case ECapeInvalidArgumentHR: s=L"Invalid argument";break;
					  case ECapeOutOfBoundsHR: s=L"index out of bounds";break;
					  case ECapeImplementationHR: s=L"current implementation related error";break;
					  case ECapeNoImplHR: s=L"not implemented";break;
					  case ECapeLimitedImplHR: s=L"implementation is limited";break;
					  case ECapeComputationHR: s=L"computation error";break;
					  case ECapeOutOfResourcesHR: s=L"out of resources";break;
					  case ECapeNoMemoryHR: s=L"out of memory";break;
					  case ECapeTimeOutHR: s=L"operation was timed out";break;
					  case ECapeFailedInitialisationHR: s=L"initialization error";break;
					  case ECapeSolvingErrorHR: s=L"solution error";break;
					  case ECapeBadInvOrderHR: s=L"invalid order of invocations";break;
					  case ECapeInvalidOperationHR: s=L"invalid operation";break;
					  case ECapePersistenceHR: s=L"storage / persistence error";break;
					  case ECapeIllegalAccessHR: s=L"illegal access";break;
					  case ECapePersistenceNotFoundHR: s=L"persistence error: not found";break;
					  case ECapePersistenceSystemErrorHR: s=L"persistence system error";break;
					  case ECapePersistenceOverflowHR: s=L"persistence overflow error";break;
					  case ECapeOutsideSolverScopeHR: s=L"solver scope error";break;
					  case ECapeHessianInfoNotAvailableHR: s=L"Hessian information not available";break;
					  case ECapeThrmPropertyNotAvailableHR: s=L"One or more of the requested thermodynamic properties are not set on the material";break;
					  default: s=L"Failed to get error name for ECapeUnknown";break;
					 }
				 }
				else goto defRemark;
				if (e) e->Release();
			   }
			}
		   else
			{//get the cape error:
			 BSTR errDesc=NULL;
			 ue->get_description(&errDesc);
			 ue->Release();
			 if (errDesc) 
			 if (*errDesc)
			  {s=errDesc;
			   SysFreeString(errDesc);
			  }
			 else goto tryRoot;
			 else goto tryRoot;
			}
		   break;
	 }
	return s;
}

