// Collection.h : Declaration of the CCollection

#pragma once
#include "resource.h"       // main symbols

#include "CPPMixerSplitterexample.h"
#include "CAPEOPENBaseObject.h"

//! Generic CAPE-OPEN Collection class
/*!
  CAPE-OPEN Collection class for exposing a collection of CAPE-OPEN objects. 
  The items in the class are stored as references to CAPEOPENBaseObjects, so that 
  they can be cast to the proper type and so that their name is immediately 
  accesible  
*/

class ATL_NO_VTABLE CCollection :
	public CComCoClass<CCollection, &CLSID_Collection>,
	public IDispatchImpl<ICapeCollection, &__uuidof(ICapeCollection), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>,
	public CAPEOPENBaseObject
{

public:

	vector<CAPEOPENBaseObject *> items; /*!< the items in the collection, stored as CAPEOPENBaseObject pointers. A vector is not necesseraly the most efficient approach for large collections. For this example it will do.*/

	//! Helper function for creating the collection 
    /*!
      Helper function for creating the collection as an exposable COM object. After calling CreateCollection, the reference
      count is one; do not delete the returned object. Use Release() instead
      \param name name of the collection
      \param description description of the collection
      \sa CCollection()
    */
    
    static CComObject<CCollection> *CreateCollection(const OLECHAR *name,const OLECHAR *description)
    {CComObject<CCollection> *p;
     CComObject<CCollection>::CreateInstance(&p); //create the instance with zero references
     p->AddRef(); //now it has one reference, the caller must Release this object
     p->name=name;
     p->description=description;
     return p;
    }

	//! Helper function for adding elements
    /*!
      Helper function for adding elements to the collection; the collection will not destroy the elements; this is up to the caller.
      In the context of this example implementation, there is no function to remove or insert items
    */
    
    void AddItem(CAPEOPENBaseObject *newItem)
    {items.push_back(newItem);
    }

	//! Constructor.
    /*!
      Creates an empty collection of which the name cannot be changed by external applications.
      Use CreateCollection instead of new
      \sa CreateColllection()
    */

	CCollection() : CAPEOPENBaseObject(false)
	{
	}
	
	//this object cannot be created using CoCreateInstance, so we do not need to put anything in the registry

	DECLARE_NO_REGISTRY()

	//COM map, including that of the CAPEOPENBaseObject
	
	BEGIN_COM_MAP(CCollection)
		COM_INTERFACE_ENTRY2(IDispatch, ICapeCollection)
		COM_INTERFACE_ENTRY(ICapeCollection)
		BASEMAP
	END_COM_MAP()


	DECLARE_PROTECT_FINAL_CONSTRUCT()

	// ICapeCollection Methods

	//! ICapeCollection::Item
    /*!
      Return an item from the collection; items can be requested either by name (in which 
      case we will assume a case-insensitive comparison) or by integer index (in which 
      case the first item in the collection has index 1)
      \param id [in] ID of the requested item (name or index)
      \param Item [out, retval] receives the requested item, if succesful. Cannot be NULL.
    */

	STDMETHOD(Item)(VARIANT id, LPDISPATCH * Item)
	{   if (!Item) return E_POINTER; //invalid pointer
	    int index=-1; //index of the item in the collection, if we find it
	    //is the value a name? we presume so in case it is a string
	    if (id.vt==VT_BSTR)
	     {//string
	      for (index=0;index<(int)items.size();index++)
	       {if (CBSTR::Same(id.bstrVal,items[index]->name.c_str()))
	         break;
	       }
	      if (index==items.size())
	       {index=-1; //not found, see below
	       }
	     }
	    else
	     {//not a string, convert to an integer
	      VARIANT IndexInArray;
	      IndexInArray.vt=VT_EMPTY;
	      if (SUCCEEDED(VariantChangeType(&IndexInArray,&id,0,VT_I4)))
	       {//IndexInArray should now be of VT_I4 type
	        ATLASSERT(IndexInArray.vt==VT_I4); //this does not allocate any memory, no need to VariantClear it
	        index=IndexInArray.lVal-1; //convert from 1-based to 0-based
	       }
	     }
	    //check if ok
	    if (index<0)
	     {SetError(L"Requested item not found",L"ICapeCollection",L"Item");
	      return ECapeUnknownHR;
	     }
	    if (index>=(int)items.size()) 
	     {SetError(L"Index out of range",L"ICapeCollection",L"Item");
	      return ECapeUnknownHR;
	     }
	    //ok, return the object. Rather than below we can also call QueryInterface on the item
	    *Item=(ICapeIdentification*)items[index];
	    (*Item)->AddRef(); //caller must release the item
		return NOERROR;
	}

	//! ICapeCollection::Count
    /*!
      Return the number of elements in the collection
      \param itemsCount [out, retval] receives the number of elements. Cannot be NULL.
    */

	STDMETHOD(Count)(long * itemsCount)
	{	if (!itemsCount) return E_POINTER; //invalid pointer
	    *itemsCount=(long)items.size();
		return NOERROR;
	}

};

//convenience definition for variables of COM objects defined by this class
typedef CComObject<CCollection> CollectionObject;

OBJECT_ENTRY_AUTO(__uuidof(Collection), CCollection)
