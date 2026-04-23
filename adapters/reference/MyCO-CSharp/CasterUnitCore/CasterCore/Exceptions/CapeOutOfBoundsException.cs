using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Runtime.Serialization;
using CAPEOPEN;
#pragma warning disable 1591

namespace CasterUnitBase
{
    [Description("CapeOutOfBoundsException")]
    [ComVisible(true)]
    [Guid("4438458A-1659-48c2-9138-03AD8B4C38D8")]
    [ClassInterface(ClassInterfaceType.None)]
    [Serializable]
    public class CapeOutOfBoundsException : CapeBoundariesException, ECapeOutOfBounds, ECapeBadArgument, ECapeData
    {
        public virtual short position { get; protected set; }

        #region Constructor
        public CapeOutOfBoundsException(string message, Exception inner, int position, double LowerBound, double UpperBound, double value, string type)
            : base(message, inner, LowerBound, UpperBound, value, type)
        {
            this.position = (short)position;
        }

        public CapeOutOfBoundsException(SerializationInfo info, StreamingContext context, int position, double LowerBound, double UpperBound, double value, string type)
            : base(info, context, LowerBound, UpperBound, value, type)
        {
            this.position = (short)position;
        }

        public CapeOutOfBoundsException(string message, int position, double LowerBound, double UpperBound, double value, string type)
            : base(message, LowerBound, UpperBound, value, type)
        {
            this.position = (short)position;
        }

        public CapeOutOfBoundsException(int position, double LowerBound, double UpperBound, double value, string type)
            : base(LowerBound, UpperBound, value, type)
        {
            this.position = (short)position;
        }
        #endregion

        protected override void Initialize()
        {
            this.HResult = (int)CapeErrorHResult.ECapeOutOfBoundsHR;
            this.interfaceName = "ECapeOutOfBounds";
            this.name = "CapeOutOfBoundsException";
        }
    }
}
