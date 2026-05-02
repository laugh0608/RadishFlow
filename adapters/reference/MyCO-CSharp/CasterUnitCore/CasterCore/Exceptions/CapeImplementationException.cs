using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Runtime.Serialization;
using CAPEOPEN;
#pragma warning disable 1591

namespace CasterUnitBase
{
    [Description("CapeImplementationException")]
    [Guid("7828A87E-582D-4947-9E8F-4F56725B6D75")]
    [ClassInterface(ClassInterfaceType.None)]
    [ComVisible(true)]
    [Serializable]
    public class CapeImplementationException : CapeUserException, ECapeImplementation
    {
        #region Constructor
        public CapeImplementationException(string message, Exception inner)
            : base(message, inner)
        {
        }

        public CapeImplementationException(SerializationInfo info, StreamingContext context)
            : base(info, context)
        {
        }

        public CapeImplementationException(string message)
            : base(message)
        {
        }

        public CapeImplementationException()
        {
        }
        #endregion

        protected override void Initialize()
        {
            this.HResult = (int)CapeErrorHResult.ECapeImplementationHR;
            this.interfaceName = "ECapeImplementation";
            this.name = "CapeImplementationException";
        }
    }
}
