using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Runtime.Serialization;
using CAPEOPEN;
#pragma warning disable 1591

namespace CasterUnitBase
{
    [Description("CapeOutOfResourcesException")]
    [Guid("42B785A7-2EDD-4808-AC43-9E6E96373616")]
    [ComVisible(true)]
    [ClassInterface(ClassInterfaceType.None)]
    [Serializable]
    public class CapeOutOfResourcesException : CapeBoundariesException, ECapeOutOfResources, ECapeComputation
    {
        #region Constructor
        public CapeOutOfResourcesException(string message, Exception inner, double LowerBound, double UpperBound, double value, string type)
            : base(message, inner, LowerBound, UpperBound, value, type)
        {
        }

        public CapeOutOfResourcesException(SerializationInfo info, StreamingContext context, double LowerBound, double UpperBound, double value, string type)
            : base(info, context, LowerBound, UpperBound, value, type)
        {
        }

        public CapeOutOfResourcesException(string message, double LowerBound, double UpperBound, double value, string type)
            : base(message, LowerBound, UpperBound, value, type)
        {
        }

        public CapeOutOfResourcesException(double LowerBound, double UpperBound, double value, string type)
            : base(LowerBound, UpperBound, value, type)
        {
        }
        #endregion
        protected override void Initialize()
        {
            this.HResult = (int)CapeErrorHResult.ECapeOutOfResourcesHR;
            this.interfaceName = "ECapeOutOfResources";
            this.name = "CapeOutOfResourcesException";
        }
    }
}
