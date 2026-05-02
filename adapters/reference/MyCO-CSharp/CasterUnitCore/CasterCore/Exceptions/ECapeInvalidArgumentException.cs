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
    [Description("CapeInvalidArgumentException")]
    [ClassInterface(ClassInterfaceType.None)]
    [Guid("B30127DA-8E69-4d15-BAB0-89132126BAC9")]
    [ComVisible(true)]
    [Serializable]
    public class ECapeInvalidArgumentException : ECapeBadArgumentException, ECapeInvalidArgument
    {
        #region Constructor
        public ECapeInvalidArgumentException(string message, Exception inner, int position)
            : base(message, inner, position)
        {
        }

        public ECapeInvalidArgumentException(SerializationInfo info, StreamingContext context, int position)
            : base(info, context, position)
        {
        }

        public ECapeInvalidArgumentException(string message, int position)
            : base(message, position)
        {
        }

        public ECapeInvalidArgumentException(int position)
            : base(position)
        {
        }
        #endregion

        protected override void Initialize()
        {
            this.HResult = (int)ECapeErrorHResult.ECapeInvalidArgumentHR;
            this.interfaceName = "ECapeInvalidArgument";
            this.name = "CapeInvalidArgumentException";
        }
    }
}
