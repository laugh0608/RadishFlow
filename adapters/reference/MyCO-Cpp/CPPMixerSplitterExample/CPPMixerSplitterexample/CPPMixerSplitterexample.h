

/* this ALWAYS GENERATED file contains the definitions for the interfaces */


 /* File created by MIDL compiler version 6.00.0366 */
/* at Thu Jul 07 12:59:24 2011
 */
/* Compiler settings for .\CPPMixerSplitterexample.idl:
    Oicf, W1, Zp8, env=Win32 (32b run)
    protocol : dce , ms_ext, c_ext
    error checks: allocation ref bounds_check enum stub_data 
    VC __declspec() decoration level: 
         __declspec(uuid()), __declspec(selectany), __declspec(novtable)
         DECLSPEC_UUID(), MIDL_INTERFACE()
*/
//@@MIDL_FILE_HEADING(  )

#pragma warning( disable: 4049 )  /* more than 64k source lines */


/* verify that the <rpcndr.h> version is high enough to compile this file*/
#ifndef __REQUIRED_RPCNDR_H_VERSION__
#define __REQUIRED_RPCNDR_H_VERSION__ 440
#endif

#include "rpc.h"
#include "rpcndr.h"

#ifndef __CPPMixerSplitterexample_h__
#define __CPPMixerSplitterexample_h__

#if defined(_MSC_VER) && (_MSC_VER >= 1020)
#pragma once
#endif

/* Forward Declarations */ 

#ifndef __CPPMixerSplitterUnitOperation_FWD_DEFINED__
#define __CPPMixerSplitterUnitOperation_FWD_DEFINED__

#ifdef __cplusplus
typedef class CPPMixerSplitterUnitOperation CPPMixerSplitterUnitOperation;
#else
typedef struct CPPMixerSplitterUnitOperation CPPMixerSplitterUnitOperation;
#endif /* __cplusplus */

#endif 	/* __CPPMixerSplitterUnitOperation_FWD_DEFINED__ */


#ifndef __Collection_FWD_DEFINED__
#define __Collection_FWD_DEFINED__

#ifdef __cplusplus
typedef class Collection Collection;
#else
typedef struct Collection Collection;
#endif /* __cplusplus */

#endif 	/* __Collection_FWD_DEFINED__ */


#ifndef __RealParameter_FWD_DEFINED__
#define __RealParameter_FWD_DEFINED__

#ifdef __cplusplus
typedef class RealParameter RealParameter;
#else
typedef struct RealParameter RealParameter;
#endif /* __cplusplus */

#endif 	/* __RealParameter_FWD_DEFINED__ */


#ifndef __MaterialPort_FWD_DEFINED__
#define __MaterialPort_FWD_DEFINED__

#ifdef __cplusplus
typedef class MaterialPort MaterialPort;
#else
typedef struct MaterialPort MaterialPort;
#endif /* __cplusplus */

#endif 	/* __MaterialPort_FWD_DEFINED__ */


/* header files for imported files */
#include "oaidl.h"
#include "ocidl.h"

#ifdef __cplusplus
extern "C"{
#endif 

void * __RPC_USER MIDL_user_allocate(size_t);
void __RPC_USER MIDL_user_free( void * ); 


#ifndef __CPPMixerSplitterexampleLib_LIBRARY_DEFINED__
#define __CPPMixerSplitterexampleLib_LIBRARY_DEFINED__

/* library CPPMixerSplitterexampleLib */
/* [helpstring][version][uuid] */ 


EXTERN_C const IID LIBID_CPPMixerSplitterexampleLib;

EXTERN_C const CLSID CLSID_CPPMixerSplitterUnitOperation;

#ifdef __cplusplus

class DECLSPEC_UUID("3B7FDF53-EC5A-4BF8-99F2-819A8BF90579")
CPPMixerSplitterUnitOperation;
#endif

EXTERN_C const CLSID CLSID_Collection;

#ifdef __cplusplus

class DECLSPEC_UUID("7D9B4745-1510-4BCD-B7CF-87DE6796AD23")
Collection;
#endif

EXTERN_C const CLSID CLSID_RealParameter;

#ifdef __cplusplus

class DECLSPEC_UUID("A4F42270-9605-4EB2-BAB7-AF8B66188607")
RealParameter;
#endif

EXTERN_C const CLSID CLSID_MaterialPort;

#ifdef __cplusplus

class DECLSPEC_UUID("C53E7E5D-CE57-4656-990A-7321D28BD396")
MaterialPort;
#endif
#endif /* __CPPMixerSplitterexampleLib_LIBRARY_DEFINED__ */

/* Additional Prototypes for ALL interfaces */

/* end of Additional Prototypes */

#ifdef __cplusplus
}
#endif

#endif


