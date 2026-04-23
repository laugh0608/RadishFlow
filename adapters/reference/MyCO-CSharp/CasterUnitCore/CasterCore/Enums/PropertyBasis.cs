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
using System.Collections.Generic;
using System.ComponentModel;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace CasterCore
{
    /// <summary>
    /// Enum cannot override ToString, so use a class
    /// </summary>
    [Serializable]
    public sealed class PropertyBasis
    {
        /// <summary>
        /// mole-basis
        /// </summary>
        public static readonly PropertyBasis Mole = new PropertyBasis { _value = "mole" };
        /// <summary>
        /// mass-basis
        /// </summary>
        public static readonly PropertyBasis Mass = new PropertyBasis { _value = "mass" };
        /// <summary>
        /// no basis, used for T,P,etc.
        /// </summary>
        public static readonly PropertyBasis Undefined = new PropertyBasis { _value = "" };
        
        private string _value;

        /// <summary>
        /// actual string value of the enum
        /// </summary>
        /// <returns></returns>
        public override string ToString()
        {
            return _value;
        }
    }
}
