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
using System.Reflection;
using CasterCore;

namespace CasterUnitCore
{
    /// <summary>
    /// This class is the base class for all calculate class, must be inherit, and should also inherit the specified unit calculate interface
    /// </summary>
    public abstract class Calculator
    {
        #region fields
        /// <summary>
        /// Reference to the concrete unit, to get Parameters and etc.
        /// </summary>
        public CasterUnitOperationBase UnitOp { get; set; }

        #endregion

        #region Constructor
        /// <summary>
        /// Create base SpecCalculator, create collections instance
        /// </summary>
        protected Calculator()
        {
            CasterLogger.Debug("Calculator is created");
        }
        #endregion

        #region Calculate
        /// <summary>
        /// Call before calculate
        /// 1 Get material throught Ports, MUST Duplicate, the port material cannot manipulate.
        /// 2 Get Parameters throught Parameters and Results
        /// 3 Must get the Parameters from collections here, cannot use the variable assigned in InitParameters
        /// </summary>
        public abstract void BeforeCalculate();
        /// <summary>
        /// Actual calculate
        /// </summary>
        public abstract void Calculate();
        /// <summary>
        /// Call after calculate
        /// Output result, set material to Ports, set results to Results, and clear up material
        /// </summary>
        public abstract void OutputResult();

        #endregion

    }
}
