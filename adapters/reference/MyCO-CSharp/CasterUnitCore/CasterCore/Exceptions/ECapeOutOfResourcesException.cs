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
    [Description("CapeOutOfResourcesException")]
    [Guid("42B785A7-2EDD-4808-AC43-9E6E96373616")]
    [ComVisible(true)]
    [ClassInterface(ClassInterfaceType.None)]
    [Serializable]
    public class ECapeOutOfResourcesException : ECapeBoundariesException, ECapeOutOfResources, ECapeComputation
    {
        #region Constructor
        public ECapeOutOfResourcesException(string message, Exception inner, double LowerBound, double UpperBound, double value, string type)
            : base(message, inner, LowerBound, UpperBound, value, type)
        {
        }

        public ECapeOutOfResourcesException(SerializationInfo info, StreamingContext context, double LowerBound, double UpperBound, double value, string type)
            : base(info, context, LowerBound, UpperBound, value, type)
        {
        }

        public ECapeOutOfResourcesException(string message, double LowerBound, double UpperBound, double value, string type)
            : base(message, LowerBound, UpperBound, value, type)
        {
        }

        public ECapeOutOfResourcesException(double LowerBound, double UpperBound, double value, string type)
            : base(LowerBound, UpperBound, value, type)
        {
        }
        #endregion
        protected override void Initialize()
        {
            this.HResult = (int)ECapeErrorHResult.ECapeOutOfResourcesHR;
            this.interfaceName = "ECapeOutOfResources";
            this.name = "CapeOutOfResourcesException";
        }
    }
}
