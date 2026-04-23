using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Runtime.Serialization;
using CAPEOPEN;
#pragma warning disable 1591

namespace CasterUnitBase
{
    [Description("CapePersistenceOverflowException")]
    [Guid("A119DE0B-C11E-4a14-BA5E-9A2D20B15578")]
    [ClassInterface(ClassInterfaceType.None)]
    [ComVisible(true)]
    [Serializable]
    public class CapePersistenceOverflowException : CapeBoundariesException, ECapePersistenceOverflow, ECapePersistence
    {
        #region Constructor
        public CapePersistenceOverflowException(string message, Exception inner, double LowerBound, double UpperBound, double value, string type)
            : base(message, inner, LowerBound, UpperBound, value, type)
        {
        }

        public CapePersistenceOverflowException(SerializationInfo info, StreamingContext context, double LowerBound, double UpperBound, double value, string type)
            : base(info, context, LowerBound, UpperBound, value, type)
        {
        }

        public CapePersistenceOverflowException(string message, double LowerBound, double UpperBound, double value, string type)
            : base(message, LowerBound, UpperBound, value, type)
        {
        }

        public CapePersistenceOverflowException(double LowerBound, double UpperBound, double value, string type)
            : base(LowerBound, UpperBound, value, type)
        {
        }
        #endregion
        protected override void Initialize()
        {
            this.HResult = (int)CapeErrorHResult.ECapePersistenceOverflowHR;
            this.interfaceName = "ECapePersistenceOverflow";
            this.name = "CapePersistenceOverflowException";
        }
    }
}
