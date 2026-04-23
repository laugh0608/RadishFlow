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

namespace CasterCore
{
    /// <summary>
    /// Allows to manage the material template
    /// </summary>
    public static class CapeMaterialTemplateSystem
    {
        static ICapeMaterialTemplateSystem _materialTemplateSystem;

        /// <summary>
        /// Set Simulation context
        /// </summary>
        /// <param name="materialTemplate">if null, will output a message to Debug</param>
        public static void SetSimulationContext(ICapeMaterialTemplateSystem materialTemplate)
        {
            if (materialTemplate == null)
                Debug.WriteLine(
                    "ICapeMaterialTemplateSystem is null, failed to set SimulationContext.");
            _materialTemplateSystem = materialTemplate;
        }

        /// <summary>
        /// Returns array of material template names supported by the COSE. This can include:
        ///  CAPE-OPEN standalone property packages,
        ///  CAPE-OPEN property packages that depend on a Property System,
        ///  Property packages that are native to the COSE.
        /// </summary>
        public static string[] MaterialTemplates
        {
            get
            {
                if (_materialTemplateSystem == null)
                {
                    CapeDiagnostic.LogMessage("Get MaterialTemplates Failed.");
                    throw new ECapeUnknownException("Get MaterialTemplates Failed.");
                }
                return (_materialTemplateSystem.MaterialTemplates as IEnumerable<string>)?.ToArray();
            }
        }

        /// <summary>
        /// Creates a new thermo material template of the specified type
        /// </summary>
        public static ICapeThermoMaterialTemplate CreateMaterialTemplate(string materialTemplateName)
        {
            if (_materialTemplateSystem == null)
            {
                CapeDiagnostic.LogMessage("Create MaterialTemplate Failed.");
                throw new ECapeUnknownException("Create MaterialTemplate Failed.");
            }
            return _materialTemplateSystem.CreateMaterialTemplate(materialTemplateName);
        }
    }
}
