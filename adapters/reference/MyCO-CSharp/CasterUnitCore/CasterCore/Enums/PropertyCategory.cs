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

// ReSharper disable InconsistentNaming

namespace CasterCore
{
    /// <summary>
    /// Property categories, used when you need to find the actual name of a property
    /// </summary>
    [Serializable]
    public enum PropertyCategory : byte
    {
        /// <summary>
        /// 
        /// </summary>
        SinglePhaseProp,
        /// <summary>
        /// 
        /// </summary>
        TwoPhaseProp,
        /// <summary>
        /// 
        /// </summary>
        ConstantProp,
        /// <summary>
        /// 
        /// </summary>
        UniversalConstantProp,
        /// <summary>
        /// 
        /// </summary>
        TDependentProp,
        /// <summary>
        /// 
        /// </summary>
        PDependentProp
    }
}
