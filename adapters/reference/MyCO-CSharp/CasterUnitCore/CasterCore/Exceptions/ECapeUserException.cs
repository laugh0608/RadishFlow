/*Copyright 2016 Caster

* Licensed under the Apache License, Version 2.0 (the "License");
* you may not use this file except in compliance with the License.
* You may obtain a copy of the License at
* 
*     http://www.apache.org/licenses/LICENSE-2.0
* 
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific language governing permissions and
* limitations under the License.*/

using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Runtime.Serialization;
using CAPEOPEN;

namespace CasterUnitCore
{
    /// <summary>
    /// 
    /// </summary>
    [Description("CapeUserException")]
    [ComVisible(true)]
    [ClassInterface(ClassInterfaceType.None)]
    [Guid("28686562-77AD-448f-8A41-8CF9C3264A3E")]
    [Serializable]
    public abstract class ECapeUserException : ApplicationException, ECapeUser, ECapeRoot
    {
        #region Constructor
        /// <summary>
        /// 
        /// </summary>
        /// <paramCollection name="message"></paramCollection>
        /// <paramCollection name="inner"></paramCollection>
        protected ECapeUserException(string message, Exception inner)
            : base(message, inner)
        {
            this.description = message;
            this.Initialize();
        }
        /// <summary>
        /// 
        /// </summary>
        /// <paramCollection name="info"></paramCollection>
        /// <paramCollection name="context"></paramCollection>
        protected ECapeUserException(SerializationInfo info, StreamingContext context)
            : base(info, context)
        {
            this.Initialize();
        }
        /// <summary>
        /// 
        /// </summary>
        /// <paramCollection name="message"></paramCollection>
        protected ECapeUserException(string message)
            : base(message)
        {
            this.description = message;
            this.Initialize();
        }
        /// <summary>
        /// 
        /// </summary>
        protected ECapeUserException()
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
