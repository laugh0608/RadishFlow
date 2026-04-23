using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Runtime.Serialization;
using CAPEOPEN;
#pragma warning disable 1591

namespace CasterUnitBase
{
    /// <summary>
    /// 
    /// </summary>
    [Description("CapeSolvingErrorException")]
    [ComVisible(true)]
    [ClassInterface(ClassInterfaceType.None)]
    [Guid("F617AFEA-0EEE-4395-8C82-288BF8F2A136")]
    [Serializable]
    public class CapeSolvingErrorException : CapeComputationException, ECapeSolvingError
    {
        #region Constructor
        public CapeSolvingErrorException(string message, Exception inner)
            : base(message, inner)
        {
        }

        public CapeSolvingErrorException(SerializationInfo info, StreamingContext context)
            : base(info, context)
        {
        }

        public CapeSolvingErrorException(string message)
            : base(message)
        {
        }

        public CapeSolvingErrorException()
        {
        }
        #endregion

        protected override void Initialize()
        {
            this.HResult = (int)CapeErrorHResult.ECapeSolvingErrorHR;
            this.interfaceName = "ECapeSolvingError";
            this.name = "CapeSolvingErrorException";
        }
    }
}
