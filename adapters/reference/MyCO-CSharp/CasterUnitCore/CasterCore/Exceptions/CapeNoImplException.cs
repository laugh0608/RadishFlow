using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Runtime.Serialization;
using CAPEOPEN;
#pragma warning disable 1591

namespace CasterUnitBase
{
    [Description("CapeNoImplException")]
    [ComVisible(true)]
    [ClassInterface(ClassInterfaceType.None)]
    [Guid("1D2488A6-C428-4e38-AFA6-04F2107172DA")]
    [Serializable]
    public class CapeNoImplException : CapeImplementationException, ECapeNoImpl
    {
        #region Constructor
        public CapeNoImplException(string message, Exception inner)
            : base(message, inner)
        {
        }

        public CapeNoImplException(SerializationInfo info, StreamingContext context)
            : base(info, context)
        {
        }

        public CapeNoImplException(string message)
            : base(message)
        {
        }

        public CapeNoImplException()
        {
        }
        #endregion
        protected override void Initialize()
        {
            this.HResult = (int)CapeErrorHResult.ECapeNoImplHR;
            this.interfaceName = "ECapeNoImpl";
            this.name = "CapeNoImplException";
        }
    }
}
