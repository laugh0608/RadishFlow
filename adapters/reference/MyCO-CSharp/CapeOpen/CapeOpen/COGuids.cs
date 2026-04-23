using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

//#pragma once
//#ifndef _COGUIDS_IDL_
//public const String  _COGUIDS_IDL_
namespace CapeOpen
{


    /* IMPORTANT NOTICE
    (c) The CAPE-OPEN Laboratory Network, 2002.
    All rights are reserved unless specifically stated otherwise

    Visit the web site at www.colan.org

    This file has been edited using the editor from Microsoft Visual Studio 6.0
    This file can viewed properly with any basic editors and browsers (validation done under MS Windows and Unix)
    */

    // This file was developed/modified by JEAN-PIERRE BELAUD for CO-LaN organisation - March 2003

    class COGuids
    {

        // CAPE-OPEN Category
        public const String CapeOpenComponent_CATID = "{678c09a1-7d66-11d2-a67d-00105a42887f}";
        // External CAPE-OPEN Thermo Routines
        public const String CapeExternalThermoRoutine_CATID = "{678c09a2-7d66-11d2-a67d-00105a42887f}";
        // CAPE-OPEN Thermo System
        public const String CapeThermoSystem_CATID = "{678c09a3-7d66-11d2-a67d-00105a42887f}";
        // CAPE-OPEN Thermo Property Package
        public const String CapeThermoPropertyPackage_CATID = "{678c09a4-7d66-11d2-a67d-00105a42887f}";
        // CAPE-OPEN Thermo Equilibrium Server
        public const String CapeThermoEquilibriumServer_CATID = "{678c09a6-7d66-11d2-a67d-00105a42887f}";
        // CAPE-OPEN Unit Operation
        public const String CapeUnitOperation_CATID = "{678c09a5-7d66-11d2-a67d-00105a42887f}";
        // CAEP-OPEN Reactions Package manager components
        public const String CAPEOPENReactionsPackageManager_CATID = "{678c09aa-0100-11d2-a67d-00105a42887f}";
        // CAPE-OPEN Standalone Reactions Package
        public const String CAPEOPENReactionsPackage_CATID = "{678c09ab-0100-11d2-a67d-00105a42887f}";
        // CAPE-OPEN MINLP Solver Package
        public const String CapeMINLPSolverPackage_CATID = "{678c09ac-7d66-11d2-a67d-00105a42887f}";
        // CAPE-OPEN PPDB Service
        public const String CapePPDBService_CATID = "{678c09aa-7d66-11d2-a67d-00105a42887f}";
        // CAPE-OPEN SMST Package
        public const String CapeSMSTPackage_CATID = "{678c09ab-7d66-11d2-a67d-00105a42887f}";
        // CAPE-OPEN Solvers Package
        public const String CapeSolversPackage_CATID = "{79DD785E-27E5-4939-B040-B1E45B1F2C64}";
        // CAPE-OPEN PSP Package
        public const String CapePSPPackage_CATID = "{3EFFA2BD-D9E7-4e55-B515-AD3E3623AAD5}";
        // CAPE-OPEN Monitoring Object
        public const String CATID_MONITORING_OBJECT = "{7BA1AF89-B2E4-493d-BD80-2970BF4CBE99}";
        // CAPE-OPEN Thermodynamics Consuming Object
        public const String Consumes_Thermo_CATID = "{4150C28A-EE06-403f-A871-87AFEC38A249}";
        // CAPE-OPEN Object Supporting Thermo 1.0
        public const String SupportsThermodynamics10_CATID = "{0D562DC8-EA8E-4210-AB39-B66513C0CD09}";
        // CAPE-OPEN Object Supporting Thermo 1.1
        public const String SupportsThermodynamics11_CATID = "{4667023A-5A8E-4cca-AB6D-9D78C5112FED}";
        /**************************************
         Identification Interfaces
        **************************************/
        public const String CapeValidationStatus_IID = "678c0b04-7d66-11d2-a67d-00105a42887f";
        public const String CapeIdentification_IID = "678c0990-7d66-11d2-a67d-00105a42887f";

        /**************************************
         Collection Interfaces
        **************************************/
        public const String ICapeCollection_IID = "678c099a-0093-11d2-a67d-00105a42887f";

        /**************************************
         Utilities Interfaces
        **************************************/
        public const String ICapeUtilities_IID = "678c0a9b-0100-11d2-a67d-00105a42887f";

        /*****************************************
        CAPE-OPEN Flowsheet monitoring interface
        *****************************************/
        public const String ICapeFlowsheetMonitoring_IID = "2CC8CC79-F854-4d65-B296-F8CD3344A2CD";

        /**************************************
         Parameter Interfaces
        **************************************/
        public const String ICapeParameterSpec_IID = "678c099c-0093-11d2-a67d-00105a42887f";
        public const String ICapeRealParameterSpec_IID = "678c099d-0093-11d2-a67d-00105a42887f";
        public const String ICapeIntegerParameterSpec_IID = "678c099e-0093-11d2-a67d-00105a42887f";
        public const String ICapeOptionParameterSpec_IID = "678c099f-0093-11d2-a67d-00105a42887f";
        public const String ICapeParameter_IID = "678c09a0-0093-11d2-a67d-00105a42887f";
        public const String ICapeBooleanParameterSpec_IID = "678c09a8-0093-11d2-a67d-00105a42887f";
        public const String ICapeArrayParameterSpec_IID = "678c09a9-0093-11d2-a67d-00105a42887f";
        public const String CapeParamType_IID = "678c0b02-7d66-11d2-a67d-00105a42887f";
        public const String CapeParamMode_IID = "678c0b03-7d66-11d2-a67d-00105a42887f";

        /**************************************
         Error Interfaces
        **************************************/
        public const String CapeErrorInterfaceHRs_IID = "678c0b01-7d66-11d2-a67d-00105a42887f";
        public const String ECapeRoot_IID = "678c0b10-7d66-11d2-a67d-00105a42887f";
        public const String ECapeUser_IID = "678C0B11-7D66-11D2-A67D-00105A42887F";
        public const String ECapeUnknown_IID = "678c0b12-7d66-11d2-a67d-00105a42887f";
        public const String ECapeData_IID = "678c0b13-7d66-11d2-a67d-00105a42887f";
        public const String ECapeLicenceError_IID = "678c0b14-7d66-11d2-a67d-00105a42887f";
        public const String ECapeBadCOParameter_IID = "678c0b15-7d66-11d2-a67d-00105a42887f";
        public const String ECapeBadArgument_IID = "E29E42B3-E481-45c6-A737-78F4A7FC0391";
        public const String ECapeInvalidArgument_IID = "678c0b17-7d66-11d2-a67d-00105a42887f";
        public const String ECapeOutOfBounds_IID = "678c0b18-7d66-11d2-a67d-00105a42887f";
        public const String ECapeImplementation_IID = "678c0b19-7d66-11d2-a67d-00105a42887f";
        public const String ECapeNoImpl_IID = "678c0b1a-7d66-11d2-a67d-00105a42887f";
        public const String ECapeLimitedImpl_IID = "678c0b1b-7d66-11d2-a67d-00105a42887f";
        public const String ECapeComputation_IID = "678c0b1c-7d66-11d2-a67d-00105a42887f";
        public const String ECapeOutOfResources_IID = "678c0b1d-7d66-11d2-a67d-00105a42887f";
        public const String ECapeNoMemory_IID = "678c0b1e-7d66-11d2-a67d-00105a42887f";
        public const String ECapeTimeOut_IID = "678c0b1f-7d66-11d2-a67d-00105a42887f";
        public const String ECapeFailedInitialisation_IID = "678c0b20-7d66-11d2-a67d-00105a42887f";
        public const String ECapeSolvingError_IID = "678c0b21-7d66-11d2-a67d-00105a42887f";
        public const String ECapeBadInvOrder_IID = "678c0b22-7d66-11d2-a67d-00105a42887f";
        public const String ECapeInvalidOperation_IID = "678c0b23-7d66-11d2-a67d-00105a42887f";
        public const String ECapePersistence_IID = "678c0b24-7d66-11d2-a67d-00105a42887f";
        public const String ECapeIllegalAccess_IID = "678c0b25-7d66-11d2-a67d-00105a42887f";
        public const String ECapePersistenceNotFound_IID = "678c0b26-7d66-11d2-a67d-00105a42887f";
        public const String ECapePersistenceSystemError_IID = "678c0b27-7d66-11d2-a67d-00105a42887f";
        public const String ECapePersistenceOverflow_IID = "678c0b28-7d66-11d2-a67d-00105a42887f";
        public const String ECapeBoundaries_IID = "678c0b29-7d66-11d2-a67d-00105a42887f";
        public const String ECapeErrorDummy_IID = "678c0b07-7d66-11d2-a67d-00105a42887f";

        /**************************************
         COSE Interfaces
        **************************************/
        public const String ICapeSimulationContext_IID = "678c0a9c-0100-11d2-a67d-00105a42887f";
        public const String ICapeDiagnostic_IID = "678c0a9d-0100-11d2-a67d-00105a42887f";
        public const String ICapeMaterialTemplateSystem_IID = "678c0a9e-0100-11d2-a67d-00105a42887f";
        public const String ICapeCOSEUtilities_IID = "678c0a9f-0100-11d2-a67d-00105a42887f";

        /**************************************
         Thermo Interfaces
        **************************************/
        public const String ICapeThermoCalculationRoutine_IID = "678c0991-7d66-11d2-a67d-00105a42887f";
        public const String ICapeThermoReliability_IID = "678c0992-7d66-11d2-a67d-00105a42887f";
        public const String ICapeThermoMaterialTemplate_IID = "678c0993-7d66-11d2-a67d-00105a42887f";
        public const String ICapeThermoMaterialObject_IID = "678c0994-7d66-11d2-a67d-00105a42887f";
        public const String ICapeThermoSystem_IID = "678c0995-7d66-11d2-a67d-00105a42887f";
        public const String ICapeThermoPropertyPackage_IID = "678c0996-7d66-11d2-a67d-00105a42887f";
        public const String ICapeThermoEquilibriumServer_IID = "678c0997-7d66-11d2-a67d-00105a42887f";
        // Example CLSID - not for use 
        public const String AspenTechThermoSystem_CLSID = "678c09a7-7d66-11d2-a67d-00105a42887f";

        /**************************************
         Solvers Interfaces
        **************************************/
        public const String ICapeNumericMatrix_IID = "3AD3C8F6-E6EC-4e63-B51E-0E9403535463";
        public const String ICapeNumericUnstructuredMatrix_IID = "678c09af-7d66-11d2-a67d-00105a42887f";
        public const String ICapeNumericFullMatrix_IID = "678c0b71-0100-11d2-a67d-00105a42887f";
        public const String ICapeNumericBandedMatrix_IID = "678c0b72-0100-11d2-a67d-00105a42887f";
        public const String ICapeNumericESOManager_IID = "678c0b73-0100-11d2-a67d-00105a42887f";
        public const String ICapeNumericESO_IID = "9304E044-3311-41ed-8766-0123CB44038A";
        public const String ICapeNumericLAESO_IID = "678c0b74-0100-11d2-a67d-00105a42887f";
        public const String ICapeNumericNLAESO_IID = "678c0b75-0100-11d2-a67d-00105a42887f";
        public const String ICapeNumericDAESO_IID = "678c0b76-0100-11d2-a67d-00105a42887f";
        public const String ICapeNumericGlobalESO_IID = "678c0b77-0100-11d2-a67d-00105a42887f";
        public const String ICapeNumericGlobalLAESO_IID = "678c0b78-0100-11d2-a67d-00105a42887f";
        public const String ICapeNumericGlobalNLAESO_IID = "678c0b79-0100-11d2-a67d-00105a42887f";
        public const String ICapeNumericGlobalDAESO_IID = "678c0b7a-0100-11d2-a67d-00105a42887f";

        public const String ICapeNumericModel_IID = "678c0b7c-0100-11d2-a67d-00105a42887f";
        public const String ICapeNumericContinuousModel_IID = "678c0b7d-0100-11d2-a67d-00105a42887f";
        public const String ICapeNumericHierarchicalModel_IID = "678c0b7e-0100-11d2-a67d-00105a42887f";
        public const String ICapeNumericAggregateModel_IID = "678c0b7f-0100-11d2-a67d-00105a42887f";
        public const String ICapeNumericSTN_IID = "678c0b80-0100-11d2-a67d-00105a42887f";
        public const String ICapeNumericEvent_IID = "678c0b81-0100-11d2-a67d-00105a42887f";
        public const String ICapeNumericBasicEvent_IID = "678c0b82-0100-11d2-a67d-00105a42887f";
        public const String ICapeNumericCompositeEvent_IID = "678c0b83-0100-11d2-a67d-00105a42887f";
        public const String ICapeNumericBinaryEvent_IID = "678c0b84-0100-11d2-a67d-00105a42887f";
        public const String ICapeNumericUnaryEvent_IID = "678c0b85-0100-11d2-a67d-00105a42887f";
        public const String ICapeNumericEventInfo_IID = "678c0b86-0100-11d2-a67d-00105a42887f";
        public const String ICapeNumericExternalEventInfo_IID = "678c0b87-0100-11d2-a67d-00105a42887f";
        public const String ICapeNumericInternalEventInfo_IID = "678c0b88-0100-11d2-a67d-00105a42887f";

        public const String ICapeNumericSolver_IID = "678c0b8a-0100-11d2-a67d-00105a42887f";
        public const String ICapeNumericLASolver_IID = "678c0b8b-0100-11d2-a67d-00105a42887f";
        public const String ICapeNumericNLASolver_IID = "678c0b8c-0100-11d2-a67d-00105a42887f";
        public const String ICapeNumericDAESolver_IID = "678c0b8d-0100-11d2-a67d-00105a42887f";

        /**************************************
         Unit Operation Interfaces
        **************************************/
        public const String ICapeUnit_IID = "678c0998-0100-11d2-a67d-00105a42887f";
        public const String ICapeUnitPort_IID = "678c0999-0093-11d2-a67d-00105a42887f";
        public const String ICapeUnitReport_IID = "678c099b-0093-11d2-a67d-00105a42887f";
        //public const String  ICapeUnitEdit_IID			"678c0a9a-0093-11d2-a67d-00105a42887f";
        //public const String  ICapeUnitCollection_IID	"678c099a-7d66-11d2-a67d-00105a42887f";
        // ICapeUnitPortVariables : new interface for mapping EO ESO to port variables
        public const String ICapeUnitPortVariables_IID = "678c09b1-7d66-11d2-a67d-00105a42887f";
        public const String CapePortDirection_IID = "678c0b05-7d66-11d2-a67d-00105a42887f";
        public const String CapePortType_IID = "678c0b06-7d66-11d2-a67d-00105a42887f";
        // Example CLSID - not for use !
        public const String HyprotechMixerSplitter_CLSID = "678c0a99-7d66-11d2-a67d-00105a42887f";

        /**************************************
         Reactions interfaces
        **************************************/
        public const String ICapeElectrolyteReactionContext_IID = "678c0afd-0100-11d2-a67d-00105a42887f";
        public const String ICapeKineticReactionContext_IID = "678c0afe-0100-11d2-a67d-00105a42887f";
        public const String ICapeReactionsPackageManager_IID = "678c0afc-0100-11d2-a67d-00105a42887f";
        public const String ICapeReactionChemistry_IID = "678c0afb-0100-11d2-a67d-00105a42887f";
        public const String ICapeReactionProperties_IID = "678c0afa-0100-11d2-a67d-00105a42887f";
        public const String ICapeReactionsRoutine_IID = "678c0af9-0100-11d2-a67d-00105a42887f";
        // ICapeThermoContext - actually part of the 1.1 thermo specification but 
        // included here becuase it is required for the Reactions interfaces
        public const String ICapeThermoContext_IID = "678c0b5f-0100-11d2-a67d-00105a42887f";
        public const String CapeReactionType_IID = "678c0b00-0100-11d2-a67d-00105a42887f";
        public const String CapeReactionRateBasis_IID = "678c0aff-0100-11d2-a67d-00105a42887f";

        /**************************************
         Petroleum Fractions interfaces
        **************************************/
        public const String ICapeThermoPetroFractions_IID = "678c0aa0-0100-11d2-a67d-00105a42887f";
        public const String ICapeUnitTypeInfo_IID = "678c0aa1-0100-11d2-a67d-00105a42887f";
        public const String CapeUnitType_IID = "678c0aa2-0100-11d2-a67d-00105a42887f";

        /**************************************
         SMST interfaces
        **************************************/
        public const String ICapeSMSTFlowsheetManager_IID = "678c0b65-0100-11d2-a67d-00105a42887f";
        public const String ICapeSMSTFlowsheet_IID = "678c0b66-0100-11d2-a67d-00105a42887f";
        public const String ICapeSMSTProcessGraph_IID = "678c0b67-0100-11d2-a67d-00105a42887f";
        public const String ICapeSMSTPartitionGraph_IID = "678c0b68-0100-11d2-a67d-00105a42887f";
        public const String ICapeSMSTOpenPartitionGraph_IID = "678c0b69-0100-11d2-a67d-00105a42887f";
        public const String ICapeSMSTAnalysisManager_IID = "678c0b6a-0100-11d2-a67d-00105a42887f";
        public const String ICapeSMSTAnalysis_IID = "678c0b6b-0100-11d2-a67d-00105a42887f";
        public const String ICapeSMSTSequencing_IID = "678c0b6c-0100-11d2-a67d-00105a42887f";
        public const String ICapeSMSTTearing_IID = "678c0b6d-0100-11d2-a67d-00105a42887f";
        public const String ICapeSMSTPartitioning_IID = "678c0b6e-0100-11d2-a67d-00105a42887f";
        public const String ICapeSMSTSMAnalysis_IID = "678c0b70-0100-11d2-a67d-00105a42887f";
        public const String CapeFlowsheetType_IID = "678c0b60-0100-11d2-a67d-00105a42887f";
        public const String CapeSMSTStream_IID = "678c0b61-0100-11d2-a67d-00105a42887f";
        public const String CapeAnalysisType_IID = "678c0b62-0100-11d2-a67d-00105a42887f";
        public const String CapeConsistencyCode_IID = "678c0b63-0100-11d2-a67d-00105a42887f";
        public const String CapeConvergenceCode_IID = "678c0b64-0100-11d2-a67d-00105a42887f";

        /**************************************
         MINLP Interfaces
        **************************************/
        public const String ICapeMINLP_IID = "678c09cc-7d66-11d2-a67d-00105a42887f";
        public const String ICapeMINLPSystem_IID = "678c09cd-7d66-11d2-a67d-00105a42887f";
        public const String ICapeMINLPSolverManager_IID = "678c09ce-7d66-11d2-a67d-00105a42887f";
        public const String ECapeOutsideSolverScope_IID = "678c0b0f-7d66-11d2-a67d-00105a42887f";
        public const String ECapeHessianInfoNotAvailable_IID = "3FF0B24B-4299-4dac-A46E-7843728AD205";

        /**************************************
         PPDB interfaces
        **************************************/
        public const String ICapePpdb_IID = "678c09b2-7d66-11d2-a67d-00105a42887f";
        public const String ICapePpdbRegister_IID = "678c09b3-7d66-11d2-a67d-00105a42887f";
        public const String ICapePpdbTables_IID = "678c09b4-7d66-11d2-a67d-00105a42887f";
        public const String ICapePpdbModels_IID = "678c09b5-7d66-11d2-a67d-00105a42887f";
        public const String CapeSpecCompound_IID = "678c0b0c-7d66-11d2-a67d-00105a42887f";
        public const String CapeSpecStructure_IID = "678c0b0d-7d66-11d2-a67d-00105a42887f";
        public const String CapeSpecDictionary_IID = "678c0b0e-7d66-11d2-a67d-00105a42887f";

        /**************************************
         PSP interfaces
        **************************************/
        public const String ICapePSPCollection_IID = "78DEFBBD-ED69-4f81-90A4-6B636CE13164";
        public const String ICapePSPResource_IID = "7A4D266A-E54D-4a7d-8877-632E89344FED";
        public const String ICapePSPRecipeEntity_IID = "85E4C4E2-57FC-43aa-A39A-78D392947080";
        public const String ICapePSPSchedule_IID = "F3E9CF96-DF8F-40f3-9543-E8D17CABBF96";
        public const String ICapePSPScheduleEntry_IID = "638D84DC-84E9-4aea-83A4-0BB8852832E9";
        public const String ICapePSPTransaction_IID = "45A1B544-4BE6-43f9-83A4-4A7CFEE802FE";
        public const String ICapePSPResourceRequirement_IID = "8F3C13F5-0C69-42a2-9438-4299C630A0A4";
        public const String ICapePSPReport_IID = "1DD05FA1-EB10-4cbe-A94A-24F6EA7E7815";
        public const String ICapePSPResourceCollection_IID = "E09E0B56-6A51-496f-B796-2C45C549B510";
        public const String ICapePSPRecipeEntityCollection_IID = "B3245794-9A3E-4af7-A8CA-2308290106F0";
        public const String ICapePSPScheduleCollection_IID = "033AC3EF-7449-4113-A10E-D70161B3FC22";
        public const String ICapePSPScheduleEntryCollection_IID = "25BC241A-8110-4806-AF7A-CB6CFA7B9A57";
        public const String ICapePSPTransactionCollection_IID = "052DAFEF-0F43-4f4d-88AB-50F6AD8FC0EB";
        public const String ICapePSPResourceRequirementCollection_IID = "DFF34851-E60E-40dd-BAA1-4FEDE69B3467";
        public const String ICapePSP_IID = "F840ECA2-941B-4af7-84DB-47E2187430A2";
        public const String ICapePetroFractions_IID = "72A94DE9-9A69-4369-B508-C033CDFD4F81";
        public const String CapeCompoundType_IID = "8091E285-3CFA-4a41-A5C4-141D0D709D87";
    }
}
