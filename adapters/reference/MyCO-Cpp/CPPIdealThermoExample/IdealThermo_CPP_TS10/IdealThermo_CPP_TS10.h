

/* this ALWAYS GENERATED file contains the definitions for the interfaces */


 /* File created by MIDL compiler version 6.00.0366 */
/* at Tue Jul 05 10:48:58 2011
 */
/* Compiler settings for .\IdealThermo_CPP_TS10.idl:
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

#ifndef __IdealThermo_CPP_TS10_h__
#define __IdealThermo_CPP_TS10_h__

#if defined(_MSC_VER) && (_MSC_VER >= 1020)
#pragma once
#endif

/* Forward Declarations */ 

#ifndef __ThermoSystem_FWD_DEFINED__
#define __ThermoSystem_FWD_DEFINED__

#ifdef __cplusplus
typedef class ThermoSystem ThermoSystem;
#else
typedef struct ThermoSystem ThermoSystem;
#endif /* __cplusplus */

#endif 	/* __ThermoSystem_FWD_DEFINED__ */


#ifndef __PropertyPackage_FWD_DEFINED__
#define __PropertyPackage_FWD_DEFINED__

#ifdef __cplusplus
typedef class PropertyPackage PropertyPackage;
#else
typedef struct PropertyPackage PropertyPackage;
#endif /* __cplusplus */

#endif 	/* __PropertyPackage_FWD_DEFINED__ */


/* header files for imported files */
#include "oaidl.h"
#include "ocidl.h"

#ifdef __cplusplus
extern "C"{
#endif 

void * __RPC_USER MIDL_user_allocate(size_t);
void __RPC_USER MIDL_user_free( void * ); 


#ifndef __IdealThermo_CPP_TS10Lib_LIBRARY_DEFINED__
#define __IdealThermo_CPP_TS10Lib_LIBRARY_DEFINED__

/* library IdealThermo_CPP_TS10Lib */
/* [helpstring][version][uuid] */ 


EXTERN_C const IID LIBID_IdealThermo_CPP_TS10Lib;

EXTERN_C const CLSID CLSID_ThermoSystem;

#ifdef __cplusplus

class DECLSPEC_UUID("A52095F0-2205-4AC8-BDE3-33FA329C35AE")
ThermoSystem;
#endif

EXTERN_C const CLSID CLSID_PropertyPackage;

#ifdef __cplusplus

class DECLSPEC_UUID("A23A5E69-DA16-4168-84CF-0BA4A84C83F6")
PropertyPackage;
#endif
#endif /* __IdealThermo_CPP_TS10Lib_LIBRARY_DEFINED__ */

/* Additional Prototypes for ALL interfaces */

/* end of Additional Prototypes */

#ifdef __cplusplus
}
#endif

#endif


