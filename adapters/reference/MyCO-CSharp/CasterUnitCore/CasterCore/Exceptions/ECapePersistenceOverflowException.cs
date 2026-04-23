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
    [Description("CapePersistenceOverflowException")]
    [Guid("A119DE0B-C11E-4a14-BA5E-9A2D20B15578")]
    [ClassInterface(ClassInterfaceType.None)]
    [ComVisible(true)]
    [Serializable]
    public class ECapePersistenceOverflowException : ECapeBoundariesException, ECapePersistenceOverflow, ECapePersistence
    {
        #region Constructor
        public ECapePersistenceOverflowException(string message, Exception inner, double LowerBound, double UpperBound, double value, string type)
            : base(message, inner, LowerBound, UpperBound, value, type)
        {
        }

        public ECapePersistenceOverflowException(SerializationInfo info, StreamingContext context, double LowerBound, double UpperBound, double value, string type)
            : base(info, context, LowerBound, UpperBound, value, type)
        {
        }

        public ECapePersistenceOverflowException(string message, double LowerBound, double UpperBound, double value, string type)
            : base(message, LowerBound, UpperBound, value, type)
        {
        }

        public ECapePersistenceOverflowException(double LowerBound, double UpperBound, double value, string type)
            : base(LowerBound, UpperBound, value, type)
        {
        }
        #endregion
        protected override void Initialize()
        {
            this.HResult = (int)ECapeErrorHResult.ECapePersistenceOverflowHR;
            this.interfaceName = "ECapePersistenceOverflow";
            this.name = "CapePersistenceOverflowException";
        }
    }
}
