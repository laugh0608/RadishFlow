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
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading.Tasks;
using CAPEOPEN;
using CasterCore;

namespace CasterUnitCore
{
    /// <summary>
    /// UnitPort connect to information
    /// </summary>
    [Serializable]
    [ComVisible(true)]
    [Guid("D5903AF9-CA6E-4323-AF60-D234B7D76CA6")]
    [ComDefaultInterface(typeof(ICapeUnitPort))]
    public class CapeInformationPort : CapeUnitPortBase, IEnumerable<ICapeParameter>
    {
        [NonSerialized]
        protected ICapeCollection paramCollection;

        /// <summary>
        /// Create a energy port, contains a ICapeCollection
        /// </summary>
        public CapeInformationPort(string name, CapePortDirection portDirection, string description = "", bool canRename = false)
            : base(name, CapePortType.CAPE_INFORMATION, portDirection, description, canRename)
        { }

        #region Override of CapeUnitPortBase

        /// <summary>
        /// connect to information, must be ICapeCollection
        /// </summary>
        public override void Connect(object objectToConnect)
        {
            CasterLogger.Debug($"Port {this.ComponentName} is connecting");
            Disconnect();
            if (objectToConnect is ICapeCollection)
            {
                paramCollection = objectToConnect as ICapeCollection;
            }
            else
                throw new ECapeUnknownException(this,"object connected to information port must be ICapeCollection");
            CasterLogger.Debug($"Port {this.ComponentName} connected.");
        }

        public override void Disconnect()
        {
            CasterLogger.Debug($"Port {this.ComponentName} is disconnecting");
            if (paramCollection != null && paramCollection.GetType().IsCOMObject)
                Marshal.FinalReleaseComObject(paramCollection);
            paramCollection = null;
            CasterLogger.Debug($"Port {this.ComponentName} disconnected.");
        }

        /// <summary>
        /// get the raw COM interface of ICapeCollection created by simulator
        /// </summary>
        public override object connectedObject
        {
            get
            {
                return paramCollection;
            }
        }

        public override bool IsConnected()
        {
            return paramCollection != null;
        }

        #endregion

        #region Information port
        /// <summary>
        /// Get Item by a string id or a int id
        /// </summary>
        public ICapeParameter this[string id]
        {
            get
            {
                if (paramCollection == null)
                    throw new ECapeUnknownException(this,
                        "There is no connected info stream");
                return paramCollection.Item(id);
            }
        }
        /// <summary>
        /// Get Item by a string id or a int id
        /// </summary>
        public ICapeParameter this[int index]
        {         //collection start with 1
            get
            {
                if (paramCollection == null)
                    throw new ECapeUnknownException(this,
                        "There is no connected info stream");
                return paramCollection.Item(index + 1);
            }
        }
        /// <summary>
        /// return the number in this collection
        /// </summary>
        /// <returns></returns>
        public int Count
        {
            get
            {
                if (paramCollection == null)
                    return 0;
                return paramCollection.Count();
            }
        }

        #endregion

        #region Implementation of IEnumerable

        public IEnumerator<ICapeParameter> GetEnumerator()
        {
            List<ICapeParameter> items = new List<ICapeParameter>(Count);
            for (int i = 0; i < Count; i++)
                items.Add(this[i]);
            return items.GetEnumerator();
        }

        IEnumerator IEnumerable.GetEnumerator()
        {
            return GetEnumerator();
        }

        #endregion
    }
}
