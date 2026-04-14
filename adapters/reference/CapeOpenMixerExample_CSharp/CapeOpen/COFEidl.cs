using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CapeOpen
{

	/// <summary>
	/// Stream type enumeration used by COFE.
	/// </summary>
	/// <remarks>
	/// This enumeration provides the Stream type enumeration used by COFE.
	/// </remarks>
	[Serializable]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("D1B15843-C0F5-4CB7-B462-E1B80456808E")]
    public enum COFEStreamType
    {
        /// <summary>
        /// COFE Material Stream.
        /// </summary>
        STREAMTYPE_MATERIAL = 0,
        /// <summary>
        /// COFE Energy Stream.
        /// </summary>
        STREAMTYPE_ENERGY = 1,
        /// <summary>
        /// COFE Information Stream.
        /// </summary>
        STREAMTYPE_INFORMATION = 2
    };

    // Will play with this later...
    //[
    //  uuid(1A8DBC2C-B92B-4B5D-BB73-232044BBC756)
    //]
    //dispinterface ICOFEDocument {
    //    properties:
    //    methods:
    //        [id(0x00000001)]
    //        BSTR GetPropertyDimensionality(
    //                        BSTR propName, 
    //                        BSTR basis);
    //        [id(0x00000002)]
    //        void ShowMainConfiguration();
    //        [id(0x00000003)]
    //        void SaveToString(BSTR* string);
    //        [id(0x00000004)]
    //        void LoadFromString(BSTR* string);
    //        [id(0x00000005)]
    //        IDispatch* GetSimulationContext();
    //        [id(0x00000006)]
    //        BSTR GetError(
    //                        IDispatch* CO_Object, 
    //                        long errorCode);
    //        [id(0x00000007)]
    //        long GetUnitCount();
    //        [id(0x00000008)]
    //        VARIANT GetUnitNames();
    //        [id(0x00000009)]
    //        IDispatch* GetUnit(VARIANT unitID);
    //        [id(0x0000000a)]
    //        BSTR GetCompoundConstantDimensionality(BSTR propName);
    //        [id(0x0000000b)]
    //        long GetStreamCount();
    //        [id(0x0000000c)]
    //        VARIANT GetStreamNames();
    //        [id(0x0000000d)]
    //        IDispatch* GetStream(VARIANT streamID);
    //        [id(0x0000000e)]
    //        VARIANT_BOOL SaveCopy(BSTR path);
    //        [id(0x0000000f)]
    //        long GetReactionPackageCount();
    //        [id(0x00000010)]
    //        VARIANT GetReactionPackageNames();
    //        [id(0x00000011)]
    //        IDispatch* GetReactionObject(VARIANT reactionPackageID);
    //        [id(0x00000012)]
    //        VARIANT_BOOL SolveFlowsheet(BSTR* message);
    //        [id(0x00000013)]
    //        VARIANT_BOOL ValidateFlowsheet(VARIANT* messages);
    //};

    /// <summary>
    /// COFE Stream Interface.
    /// </summary>
    /// <remarks>
    /// Interface implemented by COFE Stream object.
    /// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(true)]
	[System.Runtime.InteropServices.GuidAttribute("B2A15C45-D878-4E56-A19A-DED6A6AD6F91")]
	[System.ComponentModel.DescriptionAttribute("ICOFEStream Interface")]
	public interface ICOFEStream 
    {
        /// <summary>
        /// Type of the Stream from COFE.
        /// </summary>
        /// <remarks>
        /// <para>Get the type of the COFE Stream. 
        /// It has three possible values:</para>
        /// <para>   (i)   MATERIAL</para>
        /// <para>   (ii)  ENERGY</para>
        /// <para>   (iii) INFORMATION</para>
        /// </remarks>
        /// <value>Type of the Stream from COFE.</value>
        [System.Runtime.InteropServices.DispIdAttribute(1)] 
		COFEStreamType StreamType
        {
            get;
        }

        /// <summary>
        /// Unit operation upstream of the COFE Stream.
        /// </summary>
        /// <remarks>
        /// <para>Gets the unit operation upstream of the current Stream.</para>para>
        /// </remarks>
        /// <value>Unit Operation upstream of the current stream.</value>
        [System.Runtime.InteropServices.DispIdAttribute(2)]
        object UpstreamUnit
        {
            [return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]
            get;
        }

        /// <summary>
        /// Unit operation downstream of the COFE Stream.
        /// </summary>
        /// <remarks>
        /// <para>
        /// Gets the unit operation downstream of the current Stream.
        /// </para>
        /// </remarks>
        /// <value>Unit Operation downstream of the current stream.</value>
        [System.Runtime.InteropServices.DispIdAttribute(3)]
        object DownstreamUnit
        {
            [return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]
            get;
        }
    };

    /// <summary>
    /// COFE Material Interface.
    /// </summary>
    /// <remarks>
    /// Interface implemented by COFE Material object.
    /// </remarks>
    [System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(true)]
	[System.Runtime.InteropServices.GuidAttribute("2BFFCBD3-7DAB-439D-9E25-FBECC8146BE8")]
	[System.ComponentModel.DescriptionAttribute("ICOFEMaterial Interface")]
	public interface ICOFEMaterial
    {
        /// <summary>
        /// Ma6terial type used by COFE.
        /// </summary>
        /// <remarks>
        /// This method provides the material type used by COFE.
        /// </remarks>
        [System.Runtime.InteropServices.DispIdAttribute(1)] 
        string MaterialType
        {
            get;
        }

        /// <summary>
        /// Phase list supported by this material in COFE.
        /// </summary>
        /// <remarks>
        /// Phase list supported by this material in COFE.
        /// </remarks>
        [System.Runtime.InteropServices.DispIdAttribute(2)] 
		object GetSupportedPhaseList();
    };

    //[
    //  odl,
    //  uuid(8A7CC0E6-999E-4135-8634-1359AA014FB8),
    //  hidden,
    //  dual,
    //  oleautomation
    //]
    //interface ICOFEStreamParamCollection : IDispatch {
    //    [id(0x00000001), hidden]
    //    HRESULT GetCollection(LONG_PTR* collectionPointer);
    //};

    //typedef [public    ,
    //  custom(F914481D-9C62-4B43-9340-E9B2E6252E5F, 1) ]
    //long LONG_PTR;

    //[System.Runtime.InteropServices.ComImport()]
    //[System.Runtime.InteropServices.ComVisibleAttribute(true)]
    //[System.Runtime.InteropServices.GuidAttribute("095A4D35-D761-430D-9388-CBF0E36FD38C")]
    //[System.ComponentModel.DescriptionAttribute("ICOFEMaterialObject Interface")]
    //interface ICOFEMaterialObject : IDispatch {
    //    [id(0x00000001), hidden]
    //    HRESULT SetMaterial(LONG_PTR materialPointer);
    //    [id(0x00000002), hidden]
    //    HRESULT GetMaterial(LONG_PTR* materialPointer);
    //    [id(0x00000003), hidden]
    //    HRESULT GetMaterialCOM10(LONG_PTR* materialCOM10Pointer);
    //};

    [System.Runtime.InteropServices.ComImport()]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("5F6333E0-434F-4C03-85E2-6EB493EAE846")]
    [System.ComponentModel.DescriptionAttribute("ICOFEIcon Interface")]
    interface ICOFEIcon
    {
        [System.Runtime.InteropServices.DispIdAttribute(1)]
        void SetUnitOperationIcon(string iconFileName);
    };

}
