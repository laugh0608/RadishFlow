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
    [Description("CapeComputationException")]
    [ComVisible(true)]
    [ClassInterface(ClassInterfaceType.None)]
    [Guid("9D416BF5-B9E3-429a-B13A-222EE85A92A7")]
    [Serializable]
    public class ECapeComputationException : ECapeUserException, ECapeComputation
    {
        #region Constructor
        public ECapeComputationException(string message, Exception inner)
            : base(message, inner)
        {
        }

        public ECapeComputationException(SerializationInfo info, StreamingContext context)
            : base(info, context)
        {
        }

        public ECapeComputationException(string message)
            : base(message)
        {
        }

        public ECapeComputationException()
        {
        }

        #endregion

        protected override void Initialize()
        {
            this.HResult = (int)ECapeErrorHResult.ECapeComputationHR;
            this.interfaceName = "ECapeComputation";
            this.name = "CapeComputationException";
        }
    }
}
