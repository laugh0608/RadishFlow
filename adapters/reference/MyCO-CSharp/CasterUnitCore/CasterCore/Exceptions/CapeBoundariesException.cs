using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Runtime.Serialization;
using CAPEOPEN;
#pragma warning disable 1591

namespace CasterUnitBase
{
    [Description("CapeBoundariesException")]
    [Guid("62B1EE2F-E488-4679-AFA3-D490694D6B33")]
    [ComVisible(true)]
    [ClassInterface(ClassInterfaceType.None)]
    [Serializable]
    public abstract class CapeBoundariesException : CapeUserException, ECapeBoundaries
    {
        #region Constructor
        public CapeBoundariesException(string message, Exception inner, double LowerBound, double UpperBound, double value, string type)
            : base(message, inner)
        {
            this.SetBoundaries(LowerBound, UpperBound, value, type);
        }

        public CapeBoundariesException(SerializationInfo info, StreamingContext context, double LowerBound, double UpperBound, double value, string type)
            : base(info, context)
        {
            this.SetBoundaries(LowerBound, UpperBound, value, type);
        }

        public CapeBoundariesException(string message, double LowerBound, double UpperBound, double value, string type)
            : base(message)
        {
            this.SetBoundaries(LowerBound, UpperBound, value, type);
        }

        public CapeBoundariesException(double LowerBound, double UpperBound, double value, string type)
        {
            this.SetBoundaries(LowerBound, UpperBound, value, type);
        }
        #endregion

        #region Implementation of ECapeBoundaries

        public double LowerBound { get; protected set; }
        public double UpperBound { get; protected set; }
        public double value { get; protected set; }
        public string Type { get; protected set; }

        #endregion

        private void SetBoundaries(double LowerBound, double UpperBound, double value, string type)
        {
            this.LowerBound = LowerBound;
            this.UpperBound = UpperBound;
            this.value = value;
            this.Type = type;
        }

    }
}
