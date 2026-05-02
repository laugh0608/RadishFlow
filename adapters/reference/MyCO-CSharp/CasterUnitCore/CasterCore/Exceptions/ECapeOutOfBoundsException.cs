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
    [Description("CapeOutOfBoundsException")]
    [ComVisible(true)]
    [Guid("4438458A-1659-48c2-9138-03AD8B4C38D8")]
    [ClassInterface(ClassInterfaceType.None)]
    [Serializable]
    public class ECapeOutOfBoundsException : ECapeBoundariesException, ECapeOutOfBounds, ECapeBadArgument, ECapeData
    {
        public virtual short position { get; protected set; }

        #region Constructor
        public ECapeOutOfBoundsException(string message, Exception inner, int position, double LowerBound, double UpperBound, double value, string type)
            : base(message, inner, LowerBound, UpperBound, value, type)
        {
            this.position = (short)position;
        }

        public ECapeOutOfBoundsException(SerializationInfo info, StreamingContext context, int position, double LowerBound, double UpperBound, double value, string type)
            : base(info, context, LowerBound, UpperBound, value, type)
        {
            this.position = (short)position;
        }

        public ECapeOutOfBoundsException(string message, int position, double LowerBound, double UpperBound, double value, string type)
            : base(message, LowerBound, UpperBound, value, type)
        {
            this.position = (short)position;
        }

        public ECapeOutOfBoundsException(int position, double LowerBound, double UpperBound, double value, string type)
            : base(LowerBound, UpperBound, value, type)
        {
            this.position = (short)position;
        }
        #endregion

        protected override void Initialize()
        {
            this.HResult = (int)ECapeErrorHResult.ECapeOutOfBoundsHR;
            this.interfaceName = "ECapeOutOfBounds";
            this.name = "CapeOutOfBoundsException";
        }
    }
}
