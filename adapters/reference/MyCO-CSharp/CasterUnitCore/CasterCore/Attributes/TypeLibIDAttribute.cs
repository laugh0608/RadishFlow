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
using System.Runtime.InteropServices;

namespace CasterCore
{
    /// <summary>
    /// type lib id, will register at
    /// \ClassRoot\CLSID\{Guid}\TypeLib,
    /// will be ignored if set to Empty
    /// </summary>
    [Serializable]
    [ComVisible(false)]
    [AttributeUsage(AttributeTargets.Class)]
    public class TypeLibIDAttribute : Attribute
    {
        /// <summary>
        /// type lib GUID
        /// </summary>
        public Guid GUID { get; private set; }

        /// <summary>
        /// Constructor of TypeLibIDAttribute
        /// </summary>
        public TypeLibIDAttribute(string GUID)
        {
            this.GUID = Guid.Parse(GUID);
        }
    }
}
