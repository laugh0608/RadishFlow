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

using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using CAPEOPEN;
using System;

namespace CasterCore
{
    //For now, I assume simulation context is all the same to all co component
    //If not, I should record different simulation context with its owner

    /// <summary>
    /// CapeCOSEUtilities which provides a holdall concept.
    /// That allows gathering many (very) basic functionalities
    /// without links between them within a single interface.
    /// At present it contains operations handling standardised values.
    /// </summary>
    public static class CapeCOSEUtilities
    {
        private static ICapeCOSEUtilities _utilities;

        /// <summary>
        /// Set Simulation context
        /// </summary>
        /// <param name="utilities">if null, will output a message to Debug</param>
        public static void SetSimulationContext(ICapeCOSEUtilities utilities)
        {
            if (utilities == null)
                CapeDiagnostic.LogMessage(
                    "ICapeCOSEUtilities is null, failed to set SimulationContext.");
            _utilities = utilities;
        }

        /// <summary>
        /// Returns a list of named values supported by the COSE.
        /// </summary>
        public static string[] NamedValueList
        {
            get
            {
                if (_utilities == null)
                {
                    CapeDiagnostic.LogMessage(
                        "Get NamedValueList Failed. No simulation context is given.");
                    throw new ECapeUnknownException(
                        "Get NamedValueList Failed. No simulation context is given.");
                }
                return (_utilities.NamedValueList as IEnumerable<string>)?.ToArray();
            }
        }

        /// <summary>
        /// Returns the value corresponding to the value named name.
        /// </summary>
        public static object NamedValue(string name)
        {
            if (_utilities == null)
            {
                CapeDiagnostic.LogMessage("Get NamedValue Failed.");
                throw new ECapeUnknownException(
                    "Get NamedValue Failed.");
            }
            return _utilities.get_NamedValue(name);
        }
    }
}
