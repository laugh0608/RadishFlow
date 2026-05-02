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
    [Description("CapePersistenceNotFoundException")]
    [Guid("FC1A572F-4264-4BCA-A813-048B3F67E634")]
    [ClassInterface(ClassInterfaceType.None)]
    [ComVisible(true)]
    [Serializable]
    public class ECapePersistenceNotFoundException : ECapeUserException, ECapePersistenceNotFound, ECapePersistence
    {
        public string itemName { get; private set; }

        #region Constructor
        public ECapePersistenceNotFoundException(string itemName, string message, Exception inner)
            : base(message,inner)
        {
            this.itemName = itemName;
        }

        public ECapePersistenceNotFoundException(string itemName, SerializationInfo info, StreamingContext context)
            : base(info,context)
        {
            this.itemName = itemName;
        }


        #endregion

        protected override void Initialize()
        {
            this.HResult = (int)ECapeErrorHResult.ECapePersistenceNotFoundHR;
            this.interfaceName = "ECapePersistenceNotFound";
            this.name = "CapePersistenceNotFoundException";
        }

    }
}
