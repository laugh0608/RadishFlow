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
#pragma warning disable 1591

namespace CasterUnitCore
{
    [Description("CapeImplementationException")]
    [Guid("7828A87E-582D-4947-9E8F-4F56725B6D75")]
    [ClassInterface(ClassInterfaceType.None)]
    [ComVisible(true)]
    [Serializable]
    public class ECapeImplementationException : ECapeUserException, ECapeImplementation
    {
        #region Constructor
        public ECapeImplementationException(string message, Exception inner)
            : base(message, inner)
        {
        }

        public ECapeImplementationException(SerializationInfo info, StreamingContext context)
            : base(info, context)
        {
        }

        public ECapeImplementationException(string message)
            : base(message)
        {
        }

        public ECapeImplementationException()
        {
        }
        #endregion

        protected override void Initialize()
        {
            this.HResult = (int)ECapeErrorHResult.ECapeImplementationHR;
            this.interfaceName = "ECapeImplementation";
            this.name = "CapeImplementationException";
        }
    }
}
