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
    [Description("CapeNoImplException")]
    [ComVisible(true)]
    [ClassInterface(ClassInterfaceType.None)]
    [Guid("1D2488A6-C428-4e38-AFA6-04F2107172DA")]
    [Serializable]
    public class ECapeNoImplException : ECapeImplementationException, ECapeNoImpl
    {
        #region Constructor
        public ECapeNoImplException(string message, Exception inner)
            : base(message, inner)
        {
        }

        public ECapeNoImplException(SerializationInfo info, StreamingContext context)
            : base(info, context)
        {
        }

        public ECapeNoImplException(string message)
            : base(message)
        {
        }

        public ECapeNoImplException()
        {
        }
        #endregion
        protected override void Initialize()
        {
            this.HResult = (int)ECapeErrorHResult.ECapeNoImplHR;
            this.interfaceName = "ECapeNoImpl";
            this.name = "CapeNoImplException";
        }
    }
}
