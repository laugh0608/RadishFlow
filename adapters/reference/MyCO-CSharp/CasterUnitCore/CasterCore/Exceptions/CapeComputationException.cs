using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Runtime.Serialization;
using CAPEOPEN;
#pragma warning disable 1591

namespace CasterUnitBase
{
    [Description("CapeComputationException")]
    [ComVisible(true)]
    [ClassInterface(ClassInterfaceType.None)]
    [Guid("9D416BF5-B9E3-429a-B13A-222EE85A92A7")]
    [Serializable]
    public class CapeComputationException : CapeUserException, ECapeComputation
    {
        #region Constructor
        public CapeComputationException(string message, Exception inner)
            : base(message, inner)
        {
        }

        public CapeComputationException(SerializationInfo info, StreamingContext context)
            : base(info, context)
        {
        }

        public CapeComputationException(string message)
            : base(message)
        {
        }

        public CapeComputationException()
        {
        }

        #endregion

        protected override void Initialize()
        {
            this.HResult = (int)CapeErrorHResult.ECapeComputationHR;
            this.interfaceName = "ECapeComputation";
            this.name = "CapeComputationException";
        }
    }
}
