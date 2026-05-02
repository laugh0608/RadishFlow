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
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace CasterCore
{
    /// <summary>
    /// This class is used to get phase name, because it might vary from softwares.
    /// If the phase you need is not in this class, just create a new instance with the Value Property is the actual phase name
    /// </summary>
    [Serializable]
    public sealed class Phases
    {
        /// <summary>
        /// Vapor phase
        /// </summary>
        public static Phases Vapor = new Phases("Vapor");

        /// <summary>
        /// Liquid phase
        /// </summary>
        public static Phases Liquid = new Phases("Liquid");

        /// <summary>
        /// Solid phase
        /// </summary>
        public static Phases Solid = new Phases("Solid");

        /// <summary>
        /// actual string of this phase， could be modified in runtime
        /// </summary>
        public string Value { get; set; }

        /// <summary>
        /// create a phase with the specified name
        /// </summary>
        /// <paramCollection name="phase"></paramCollection>
        public Phases(string phase)
        {
            Value = phase;
        }

        /// <summary>
        /// return the actual string of the phase
        /// </summary>
        /// <returns></returns>
        public override string ToString()
        {
            return Value;
        }

    }
}
