using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Runtime.Serialization;
using CAPEOPEN;
#pragma warning disable 1591

namespace CasterUnitBase
{
    [Description("CapeDataException")]
    [Guid("53551E7C-ECB2-4894-B71A-CCD1E7D40995")]
    [ComVisible(true)]
    [ClassInterface(ClassInterfaceType.None)]
    [Serializable]
    public class CapeDataException : CapeUserException, ECapeData
    {
        #region Constructor
        public CapeDataException(string message, Exception inner)
            : base(message, inner)
        {
        }

        public CapeDataException(SerializationInfo info, StreamingContext context)
            : base(info, context)
        {
        }

        public CapeDataException(string message)
            : base(message)
        {
        }

        public CapeDataException()
        {
        }
        #endregion
        protected override void Initialize()
        {
            this.HResult = (int )CapeErrorHResult.ECapeDataHR;
            this.name = "CapeDataException";
            this.interfaceName = "ECapeData";
        }
    }
}
