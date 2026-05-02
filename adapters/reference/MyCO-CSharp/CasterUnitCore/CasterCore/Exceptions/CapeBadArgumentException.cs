using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Runtime.Serialization;
using CAPEOPEN;
#pragma warning disable 1591

namespace CasterUnitBase
{
    [Description("CapeBadArgumentException")]
    [ClassInterface(ClassInterfaceType.None)]
    [ComVisible(true)]
    [Guid("D168E99F-C1EF-454c-8574-A8E26B62ADB1")]
    [Serializable]
    public class CapeBadArgumentException : CapeDataException, ECapeBadArgument
    {
        public virtual short position { get; protected set; }
        #region Constructor
        public CapeBadArgumentException(string message, Exception inner, int position)
            : base(message, inner)
        {
            this.Initialize(position);
        }

        public CapeBadArgumentException(SerializationInfo info, StreamingContext context, int position)
            : base(info, context)
        {
            this.Initialize(position);
        }

        public CapeBadArgumentException(string message, int position)
            : base(message)
        {
            this.Initialize(position);
        }

        public CapeBadArgumentException(int position)
        {
            this.Initialize(position);
        }
        #endregion
        protected void Initialize(int position)
        {
            this.HResult = (int)CapeErrorHResult.ECapeBadArgumentHR;
            this.interfaceName = "ECapeBadArgument";
            this.name = "CapeBadArgumentException";
            this.position = (short)position;
        }
    }
}
