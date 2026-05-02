using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Runtime.Serialization;
using CAPEOPEN;
#pragma warning disable 1591

namespace CasterUnitBase
{
    [Description("CapeInvalidOperationException")]
    [ClassInterface(ClassInterfaceType.None)]
    [Guid("C0B943FE-FB8F-46b6-A622-54D30027D18B")]
    [ComVisible(true)]
    [Serializable]
    public class CapeInvalidOperationException : CapeComputationException, ECapeInvalidOperation
    {
        #region Constructor
        public CapeInvalidOperationException(string message, Exception inner)
            : base(message, inner)
        {
        }

        public CapeInvalidOperationException(SerializationInfo info, StreamingContext context)
            : base(info, context)
        {
        }

        public CapeInvalidOperationException(string message)
            : base(message)
        {
        }

        public CapeInvalidOperationException()
        {
        }
        #endregion

        protected override void Initialize()
        {
            this.HResult = (int)CapeErrorHResult.ECapeInvalidOperationHR;
            this.interfaceName = "ECapeInvalidOperation";
            this.name = "CapeInvalidOperationException";
        }
    }
}
