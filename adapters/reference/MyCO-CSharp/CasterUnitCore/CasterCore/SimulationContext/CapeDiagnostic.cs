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

using System.Diagnostics;
using CAPEOPEN;

namespace CasterCore
{
    /// <summary>
    /// Encloses the diagnostic functionality.
    /// </summary>
    public static class CapeDiagnostic
    {
        static ICapeDiagnostic _diagnostic;

        /// <summary>
        /// Set Simulation context
        /// </summary>
        /// <param name="diagnostic">if null, will output a message to Debug</param>
        public static void SetSimulationContext(ICapeDiagnostic diagnostic)
        {
            if (diagnostic == null)
                Debug.WriteLine(
                    "ICapeDiagnostic is null, failed to set SimulationContext.");
            _diagnostic = diagnostic;
        }

        /// <summary>
        /// Same usage as Console.Write(). A messageBox containing message should be poped by COSE.
        /// To be usable without simulation context, will log in Debug.
        /// </summary>
        public static void PopUpMessage(string msg, params object[] args)
        {
            if (_diagnostic != null)
                _diagnostic.PopUpMessage(string.Format(msg, args));
            CasterLogger.DebugFormatted("Popup: " + msg, args);
            Debug.WriteLine(string.Format(msg, args));
        }

        /// <summary>
        /// Same usage as Console.Write(). COSE should be log the message.
        /// To be usable without simulation context, will log in Debug.
        /// </summary>
        public static void LogMessage(string msg, params object[] args)
        {
            if (_diagnostic != null)
                _diagnostic.LogMessage(string.Format(msg, args));
            CasterLogger.DebugFormatted(msg, args);
            Debug.WriteLine(string.Format(msg, args));
        }
    }
}
