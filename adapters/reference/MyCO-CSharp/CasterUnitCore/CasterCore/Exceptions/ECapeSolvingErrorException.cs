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
    /// <summary>
    /// 
    /// </summary>
    [Description("CapeSolvingErrorException")]
    [ComVisible(true)]
    [ClassInterface(ClassInterfaceType.None)]
    [Guid("F617AFEA-0EEE-4395-8C82-288BF8F2A136")]
    [Serializable]
    public class ECapeSolvingErrorException : ECapeComputationException, ECapeSolvingError
    {
        #region Constructor
        public ECapeSolvingErrorException(string message, Exception inner)
            : base(message, inner)
        {
        }

        public ECapeSolvingErrorException(SerializationInfo info, StreamingContext context)
            : base(info, context)
        {
        }

        public ECapeSolvingErrorException(string message)
            : base(message)
        {
        }

        public ECapeSolvingErrorException()
        {
        }
        #endregion

        protected override void Initialize()
        {
            this.HResult = (int)ECapeErrorHResult.ECapeSolvingErrorHR;
            this.interfaceName = "ECapeSolvingError";
            this.name = "CapeSolvingErrorException";
        }
    }
}
