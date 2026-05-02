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

namespace CasterCore
{
    /// <summary>
    /// CAPE-OPEN exceptions, can be catched by simulators
    /// </summary>
    [Serializable]
    [Description("CapeUnknownException")]
    [ClassInterface(ClassInterfaceType.None)]
    [ComVisible(true)]
    [Guid("B550B2CA-6714-4e7f-813E-C93248142410")]
    public class ECapeUnknownException :
        ApplicationException,
        ECapeUser, ECapeRoot, ECapeUnknown
    {
        /// <summary>
        /// Object which throw the exception
        /// </summary>
        public CapeOpenBaseObject ErrorObject;

        /// <summary>
        /// actual .net exception
        /// </summary>
        public new Exception InnerException { get; set; }

        #region Constructor

        ///// <summary>
        ///// create a cape unknown exception by .net exception, obj should be the unit which throw the exception, in most case, it's UnitOp
        ///// </summary>
        ///// <paramCollection name="message"></paramCollection>
        ///// <paramCollection name="inner"></paramCollection>
        //public ECapeUnknownException(CapeOpenBaseObject obj, Exception inner, string interfaceName = null)
        //    : base(inner.Message, inner)
        //{
        //    _errorObject = obj;
        //    this.InnerException = inner;
        //    this.HResult = (int)ECapeErrorHResult.ECapeUnknownHR;
        //    this.name = inner.Message;
        //    this.interfaceName = interfaceName;
        //    obj.SetError(inner.Message, interfaceName, scope);
        //}

        /// <summary>
        /// throw an error, will create an empty CapeCollection to throw this exception
        /// </summary>
        public ECapeUnknownException(string message = null)
            : this(new CapeCollection(name: "ErrorObj"), message, null, null)
        { }

        /// <summary>
        /// create an cape unknown exception, obj should be the unit which throw the exception, in most case, it's UnitOp
        /// </summary>
        public ECapeUnknownException(CapeOpenBaseObject obj, string message, Exception inner = null, string interfaceName = null)
            : base(message)
        {
            ErrorObject = obj;
            this.InnerException = inner;
            this.HResult = (int)ECapeErrorHResult.ECapeUnknownHR;
            this.name = message ?? inner.Message;
            this.interfaceName = interfaceName;

            string errorMessage = $"CapeUnknownException : {message}";
            if (inner != null)
                errorMessage += $"\n InnerMessage : {inner.Message}";
            CapeDiagnostic.LogMessage(errorMessage);
            ErrorObject.SetError(message, interfaceName, scope);
        }

        #endregion

        #region ECapeUser
        /// <summary>
        /// Code to designate the subcategory of the error.
        /// </summary>
        public int code { get { return HResult; } }
        /// <summary>
        /// The description of the error.
        /// </summary>
        public string description { get { return Message; } }
        /// <summary>
        /// The scope of the error. The list of packages where the error occurs separated by "::". For example CapeOpen::Common::Identification.
        /// </summary>
        public string scope { get { return Source; } }
        /// <summary>
        /// The name of the interface where the error is thrown. 
        /// </summary>
        public string interfaceName { get; protected set; }
        /// <summary>
        /// The name of the operation where the error is thrown. 
        /// </summary>
        public string operation { get { return StackTrace; } }
        /// <summary>
        /// An URL to a page, document, web site, … where more information on the error can be found.
        /// </summary>
        public string moreInfo { get { return this.HelpLink; } }

        #endregion

        #region ECapeRoot

        /// <summary>
        /// A short description of the error.
        /// </summary>
        public string name { get; private set; }

        #endregion

        /// <summary>
        /// Serialization of ECapeUnknownException
        /// </summary>
        /// <param name="info">include original exception data, and ErrorObject, InnerException, name, interfaceName</param>
        /// <param name="context"></param>
        public override void GetObjectData(SerializationInfo info, StreamingContext context)
        {
            base.GetObjectData(info, context);

            info.AddValue("ErrorObject", ErrorObject);
            info.AddValue("InnerException", InnerException);
            info.AddValue("name", name);
            info.AddValue("interfaceName", interfaceName);
        }

        /// <summary>
        /// return "{ErrorObject.ComponentName}: {Message}. {interfaceName}"
        /// </summary>
        /// <returns></returns>
        public override string ToString()
        {
            return $"{ErrorObject.ComponentName}: {Message}. {interfaceName}";
        }
    }
}
