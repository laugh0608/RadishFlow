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
using CAPEOPEN;
using System.Runtime.InteropServices;
using CasterCore;

namespace CasterUnitCore
{
    /// <summary>
    /// UnitPort base
    /// </summary>
    [Serializable]
    //[ComVisible(true)]
    //[Guid("C53E7E5D-CE57-4656-990A-7321D28BD396")]
    //[ComDefaultInterface(typeof(ICapeUnitPort))]
    public abstract class CapeUnitPortBase
        : CapeOpenBaseObject, ICapeUnitPort, ICapeUnitPortVariables
    {
        protected CapePortDirection _portDirection;
        protected CapePortType _portType;

        /// <summary>
        /// default name is "capeUnitPort"
        /// </summary>
        protected CapeUnitPortBase(string name, CapePortType type, CapePortDirection portDirection, string description = "", bool canRename = false)
            : base(name, description, canRename)
        {
            CasterLogger.Debug($"Create unit port {name}, type is {type}, direction is {portDirection}");
            _portDirection = portDirection;
            _portType = type;
        }
        /// <summary>
        /// connect to stream, can be material or ICapeCollection
        /// </summary>
        public abstract void Connect(object objectToConnect);
        /// <summary>
        /// release object
        /// </summary>
        public abstract void Disconnect();
        /// <summary>
        /// get port type
        /// </summary>
        public CapePortType portType
        { get { return _portType; } }
        /// <summary>
        /// get port direction
        /// </summary>
        public CapePortDirection direction
        { get { return _portDirection; } }
        /// <summary>
        /// get the raw COM interface of connected object
        /// </summary>
        public abstract object connectedObject { get; }
        /// <summary>
        /// Whether the port is filled with something
        /// </summary>
        public abstract bool IsConnected();

        #region ICapeUnitPortVariables

        void ICapeUnitPortVariables.SetIndex(string Variable_type, string Component, int index)
        {
            throw new NotImplementedException();
        }

        int ICapeUnitPortVariables.get_Variable(string Variable_type, string Component)
        {
            throw new NotImplementedException();
        }

        #endregion
    }
}
