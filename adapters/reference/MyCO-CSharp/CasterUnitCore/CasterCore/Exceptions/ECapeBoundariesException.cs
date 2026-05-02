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
    [Description("CapeBoundariesException")]
    [Guid("62B1EE2F-E488-4679-AFA3-D490694D6B33")]
    [ComVisible(true)]
    [ClassInterface(ClassInterfaceType.None)]
    [Serializable]
    public abstract class ECapeBoundariesException : ECapeUserException, ECapeBoundaries
    {
        #region Constructor
        public ECapeBoundariesException(string message, Exception inner, double LowerBound, double UpperBound, double value, string type)
            : base(message, inner)
        {
            this.SetBoundaries(LowerBound, UpperBound, value, type);
        }

        public ECapeBoundariesException(SerializationInfo info, StreamingContext context, double LowerBound, double UpperBound, double value, string type)
            : base(info, context)
        {
            this.SetBoundaries(LowerBound, UpperBound, value, type);
        }

        public ECapeBoundariesException(string message, double LowerBound, double UpperBound, double value, string type)
            : base(message)
        {
            this.SetBoundaries(LowerBound, UpperBound, value, type);
        }

        public ECapeBoundariesException(double LowerBound, double UpperBound, double value, string type)
        {
            this.SetBoundaries(LowerBound, UpperBound, value, type);
        }
        #endregion

        #region Implementation of ECapeBoundaries

        public double LowerBound { get; protected set; }
        public double UpperBound { get; protected set; }
        public double value { get; protected set; }
        public string Type { get; protected set; }

        #endregion

        private void SetBoundaries(double LowerBound, double UpperBound, double value, string type)
        {
            this.LowerBound = LowerBound;
            this.UpperBound = UpperBound;
            this.value = value;
            this.Type = type;
        }

    }
}
