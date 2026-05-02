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
    [Description("CapeDataException")]
    [Guid("53551E7C-ECB2-4894-B71A-CCD1E7D40995")]
    [ComVisible(true)]
    [ClassInterface(ClassInterfaceType.None)]
    [Serializable]
    public class ECapeDataException : ECapeUserException, ECapeData
    {
        #region Constructor
        public ECapeDataException(string message, Exception inner)
            : base(message, inner)
        {
        }

        public ECapeDataException(SerializationInfo info, StreamingContext context)
            : base(info, context)
        {
        }

        public ECapeDataException(string message)
            : base(message)
        {
        }

        public ECapeDataException()
        {
        }
        #endregion
        protected override void Initialize()
        {
            this.HResult = (int )ECapeErrorHResult.ECapeDataHR;
            this.name = "CapeDataException";
            this.interfaceName = "ECapeData";
        }
    }
}
