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
    [Description("CapeTimeOutException")]
    [ClassInterface(ClassInterfaceType.None)]
    [ComVisible(true)]
    [Guid("0D5CA7D8-6574-4c7b-9B5F-320AA8375A3C")]
    [Serializable]
    public class CapeTimeOutException : CapeOutOfResourcesException, ECapeTimeOut
    {
        #region Constructor
        /// <summary>
        /// 
        /// </summary>
        /// <param name="message"></param>
        /// <param name="inner"></param>
        /// <param name="LowerBound"></param>
        /// <param name="UpperBound"></param>
        /// <param name="value"></param>
        /// <param name="type"></param>
        public CapeTimeOutException(string message, Exception inner, double LowerBound, double UpperBound, double value, string type)
            : base(message, inner, LowerBound, UpperBound, value, type)
        {
        }
        /// <summary>
        /// 
        /// </summary>
        /// <param name="info"></param>
        /// <param name="context"></param>
        /// <param name="LowerBound"></param>
        /// <param name="UpperBound"></param>
        /// <param name="value"></param>
        /// <param name="type"></param>
        public CapeTimeOutException(SerializationInfo info, StreamingContext context, double LowerBound, double UpperBound, double value, string type)
            : base(info, context, LowerBound, UpperBound, value, type)
        {
        }
        /// <summary>
        /// 
        /// </summary>
        /// <param name="message"></param>
        /// <param name="LowerBound"></param>
        /// <param name="UpperBound"></param>
        /// <param name="value"></param>
        /// <param name="type"></param>
        public CapeTimeOutException(string message, double LowerBound, double UpperBound, double value, string type)
            : base(message, LowerBound, UpperBound, value, type)
        {
        }
        /// <summary>
        /// 
        /// </summary>
        /// <param name="LowerBound"></param>
        /// <param name="UpperBound"></param>
        /// <param name="value"></param>
        /// <param name="type"></param>
        public CapeTimeOutException(double LowerBound, double UpperBound, double value, string type)
            : base(LowerBound, UpperBound, value, type)
        {
        }
        #endregion
        protected override void Initialize()
        {
            this.HResult = (int)CapeErrorHResult.ECapeTimeOutHR;
            this.interfaceName = "ECapeTimeOut";
            this.name = "CapeTimeOutException";
        }
    }
}
