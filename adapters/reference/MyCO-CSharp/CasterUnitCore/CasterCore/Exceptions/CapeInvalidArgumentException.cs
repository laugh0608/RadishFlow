using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Runtime.Serialization;
using CAPEOPEN;
#pragma warning disable 1591

namespace CasterUnitBase
{
    [Description("CapeInvalidArgumentException")]
    [ClassInterface(ClassInterfaceType.None)]
    [Guid("B30127DA-8E69-4d15-BAB0-89132126BAC9")]
    [ComVisible(true)]
    [Serializable]
    public class CapeInvalidArgumentException : CapeBadArgumentException, ECapeInvalidArgument
    {
        #region Constructor
        public CapeInvalidArgumentException(string message, Exception inner, int position)
            : base(message, inner, position)
        {
        }

        public CapeInvalidArgumentException(SerializationInfo info, StreamingContext context, int position)
            : base(info, context, position)
        {
        }

        public CapeInvalidArgumentException(string message, int position)
            : base(message, position)
        {
        }

        public CapeInvalidArgumentException(int position)
            : base(position)
        {
        }
        #endregion

        protected override void Initialize()
        {
            this.HResult = (int)CapeErrorHResult.ECapeInvalidArgumentHR;
            this.interfaceName = "ECapeInvalidArgument";
            this.name = "CapeInvalidArgumentException";
        }
    }
}
