using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Runtime.Serialization;
using CAPEOPEN;

namespace CasterUnitBase
{
    /// <summary>
    /// 
    /// </summary>
    [Description("CapeUnknownException")]
    [ClassInterface(ClassInterfaceType.None)]
    [ComVisible(true)]
    [Guid("B550B2CA-6714-4e7f-813E-C93248142410")]
    [Serializable]
    public class CapeUnknownException:CapeUserException,ECapeUnknown
    {
        #region Constructor
        /// <summary>
        /// 
        /// </summary>
        /// <param name="message"></param>
        /// <param name="inner"></param>
        public CapeUnknownException(string message, Exception inner)
            : base(message, inner){}
        /// <summary>
        /// 
        /// </summary>
        /// <param name="info"></param>
        /// <param name="context"></param>
        public CapeUnknownException(SerializationInfo info, StreamingContext context)
            : base(info, context){}
        /// <summary>
        /// 
        /// </summary>
        /// <param name="message"></param>
        public CapeUnknownException(string message)
            : base(message){}
        /// <summary>
        /// 
        /// </summary>
        public CapeUnknownException(){}

        #endregion

        #region CapeUserException

        protected override void Initialize()
        {
            this.HResult = (int)CapeErrorHResult.ECapeUnknownHR;
            this.name = "CapeUnknownException";
            this.interfaceName = "ECapeUnknown";
        }

        #endregion
    }
}
