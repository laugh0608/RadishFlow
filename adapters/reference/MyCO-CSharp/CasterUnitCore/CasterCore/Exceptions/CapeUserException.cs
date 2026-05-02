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
    [Description("CapeUserException")]
    [ComVisible(true)]
    [ClassInterface(ClassInterfaceType.None)]
    [Guid("28686562-77AD-448f-8A41-8CF9C3264A3E")]
    [Serializable]
    public abstract class CapeUserException : ApplicationException, ECapeUser, ECapeRoot
    {
        #region Constructor
        /// <summary>
        /// 
        /// </summary>
        /// <param name="message"></param>
        /// <param name="inner"></param>
        protected CapeUserException(string message, Exception inner)
            : base(message, inner)
        {
            this.description = message;
            this.Initialize();
        }
        /// <summary>
        /// 
        /// </summary>
        /// <param name="info"></param>
        /// <param name="context"></param>
        protected CapeUserException(SerializationInfo info, StreamingContext context)
            : base(info, context)
        {
            this.Initialize();
        }
        /// <summary>
        /// 
        /// </summary>
        /// <param name="message"></param>
        protected CapeUserException(string message)
            : base(message)
        {
            this.description = message;
            this.Initialize();
        }
        /// <summary>
        /// 
        /// </summary>
        protected CapeUserException()
        {
            this.description = "An application error has occurred.Caster";
            this.Initialize();
        }

        #endregion
        /// <summary>
        /// 
        /// </summary>
        protected abstract void Initialize();

        #region ECapeUser
        /// <summary>
        /// 
        /// </summary>
        public int code { get { return HResult; } }
        /// <summary>
        /// 
        /// </summary>
        public string description { get; set; }
        /// <summary>
        /// 
        /// </summary>
        public string scope { get { return Source; } }
        /// <summary>
        /// 
        /// </summary>
        public string interfaceName { get; protected set; }
        /// <summary>
        /// 
        /// </summary>
        public string operation { get { return StackTrace; } }
        /// <summary>
        /// 
        /// </summary>
        public string moreInfo { get { return this.HelpLink; } }

        #endregion

        #region ECapeRoot

        /// <summary>
        /// 
        /// </summary>
        public string name { get; protected set; }

        #endregion

    }
}
