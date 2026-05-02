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
    [Description("CapeInvalidOperationException")]
    [ClassInterface(ClassInterfaceType.None)]
    [Guid("C0B943FE-FB8F-46b6-A622-54D30027D18B")]
    [ComVisible(true)]
    [Serializable]
    public class ECapeInvalidOperationException : ECapeComputationException, ECapeInvalidOperation
    {
        #region Constructor
        public ECapeInvalidOperationException(string message, Exception inner)
            : base(message, inner)
        {
        }

        public ECapeInvalidOperationException(SerializationInfo info, StreamingContext context)
            : base(info, context)
        {
        }

        public ECapeInvalidOperationException(string message)
            : base(message)
        {
        }

        public ECapeInvalidOperationException()
        {
        }
        #endregion

        protected override void Initialize()
        {
            this.HResult = (int)ECapeErrorHResult.ECapeInvalidOperationHR;
            this.interfaceName = "ECapeInvalidOperation";
            this.name = "CapeInvalidOperationException";
        }
    }
}
