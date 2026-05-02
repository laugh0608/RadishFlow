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
    /// Name of the component, will register at 
    /// \ClassRoot\{CapeNameAttribute},
    /// \ClassRoot\{CapeNameAttribute + CapeVersionAttribute},
    /// \ClassRoot\CLSID\{Guid}\ProgID,
    /// \ClassRoot\CLSID\{Guid}\VersionIndependentProgID,
    /// \ClassRoot\CLSID\{Guid}\AppID,
    /// \ClassRoot\CLSID\{Guid}\CapeDescription\Name
    /// </summary>
    [Serializable]
    [ComVisible(false)]
    [AttributeUsage(AttributeTargets.Class)]
    public class CapeNameAttribute : Attribute
    {
        /// <summary>
        /// 
        /// </summary>
        public string Name { get; private set; }
        /// <summary>
        /// 
        /// </summary>
        /// <paramCollection name="name"></paramCollection>
        public CapeNameAttribute(string name)
        {
            Name = name;
        }
    }
}
