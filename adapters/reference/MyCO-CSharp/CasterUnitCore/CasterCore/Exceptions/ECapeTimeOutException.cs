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
    [Description("CapeTimeOutException")]
    [ClassInterface(ClassInterfaceType.None)]
    [ComVisible(true)]
    [Guid("0D5CA7D8-6574-4c7b-9B5F-320AA8375A3C")]
    [Serializable]
    public class ECapeTimeOutException : ECapeOutOfResourcesException, ECapeTimeOut
    {
        #region Constructor
        /// <summary>
        /// 
        /// </summary>
        /// <paramCollection name="message"></paramCollection>
        /// <paramCollection name="inner"></paramCollection>
        /// <paramCollection name="LowerBound"></paramCollection>
        /// <paramCollection name="UpperBound"></paramCollection>
        /// <paramCollection name="value"></paramCollection>
        /// <paramCollection name="type"></paramCollection>
        public ECapeTimeOutException(string message, Exception inner, double LowerBound, double UpperBound, double value, string type)
            : base(message, inner, LowerBound, UpperBound, value, type)
        {
        }
        /// <summary>
        /// 
        /// </summary>
        /// <paramCollection name="info"></paramCollection>
        /// <paramCollection name="context"></paramCollection>
        /// <paramCollection name="LowerBound"></paramCollection>
        /// <paramCollection name="UpperBound"></paramCollection>
        /// <paramCollection name="value"></paramCollection>
        /// <paramCollection name="type"></paramCollection>
        public ECapeTimeOutException(SerializationInfo info, StreamingContext context, double LowerBound, double UpperBound, double value, string type)
            : base(info, context, LowerBound, UpperBound, value, type)
        {
        }
        /// <summary>
        /// 
        /// </summary>
        /// <paramCollection name="message"></paramCollection>
        /// <paramCollection name="LowerBound"></paramCollection>
        /// <paramCollection name="UpperBound"></paramCollection>
        /// <paramCollection name="value"></paramCollection>
        /// <paramCollection name="type"></paramCollection>
        public ECapeTimeOutException(string message, double LowerBound, double UpperBound, double value, string type)
            : base(message, LowerBound, UpperBound, value, type)
        {
        }
        /// <summary>
        /// 
        /// </summary>
        /// <paramCollection name="LowerBound"></paramCollection>
        /// <paramCollection name="UpperBound"></paramCollection>
        /// <paramCollection name="value"></paramCollection>
        /// <paramCollection name="type"></paramCollection>
        public ECapeTimeOutException(double LowerBound, double UpperBound, double value, string type)
            : base(LowerBound, UpperBound, value, type)
        {
        }
        #endregion
        protected override void Initialize()
        {
            this.HResult = (int)ECapeErrorHResult.ECapeTimeOutHR;
            this.interfaceName = "ECapeTimeOut";
            this.name = "CapeTimeOutException";
        }
    }
}
