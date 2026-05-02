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
    [Description("CapeBadArgumentException")]
    [ClassInterface(ClassInterfaceType.None)]
    [ComVisible(true)]
    [Guid("D168E99F-C1EF-454c-8574-A8E26B62ADB1")]
    [Serializable]
    public class ECapeBadArgumentException : ECapeDataException, ECapeBadArgument
    {
        public virtual short position { get; protected set; }
        #region Constructor
        public ECapeBadArgumentException(string message, Exception inner, int position)
            : base(message, inner)
        {
            this.Initialize(position);
        }

        public ECapeBadArgumentException(SerializationInfo info, StreamingContext context, int position)
            : base(info, context)
        {
            this.Initialize(position);
        }

        public ECapeBadArgumentException(string message, int position)
            : base(message)
        {
            this.Initialize(position);
        }

        public ECapeBadArgumentException(int position)
        {
            this.Initialize(position);
        }
        #endregion
        protected void Initialize(int position)
        {
            this.HResult = (int)ECapeErrorHResult.ECapeBadArgumentHR;
            this.interfaceName = "ECapeBadArgument";
            this.name = "CapeBadArgumentException";
            this.position = (short)position;
        }
    }
}
