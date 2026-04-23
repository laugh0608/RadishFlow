#region Copyright (c) 2005-2013, Mose, All rights reserved.
/*
// Copyright (c) 2005-2013, Mose Srl (http://www.mose.units.it/)
// Original capeopentoolkit.net Source Code: Copyright (c) 2005, Marco Carone, Marco Parenzan (e-mail: marco.carone@yahoo.it; marco.parenzan@libero.it)
// All rights reserved.
//  
// Redistribution and use in source and binary forms, with or without modification, are permitted 
// provided that the following conditions are met: 
//  
// (1) Redistributions of source code must retain the above copyright notice, this list of 
// conditions and the following disclaimer. 
// (2) Redistributions in binary form must reproduce the above copyright notice, this list of 
// conditions and the following disclaimer in the documentation and/or other materials 
// provided with the distribution. 
// (3) Neither the name of the Mose nor the names of its contributors may be used 
// to endorse or promote products derived from this software without specific prior 
// written permission.
//      
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS 
// OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY 
// AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR 
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL 
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, 
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER 
// IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT 
// OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
// -------------------------------------------------------------------------
//
// Original capeopentoolkit.net Source Code: Copyright (c) 2005, Marco Carone, Marco Parenzan (e-mail: marco.carone@yahoo.it; marco.parenzan@libero.it)
// 
// Mose is a registered trademark of Mose Srl.
// 
// For portions of this software, the some additional copyright notices may apply 
// which can either be found in the license.txt file included in the source distribution
// or following this notice. 
//
*/
#endregion

using System;
using System.Runtime.InteropServices;

using CAPEOPEN093;
namespace Mose.CapeOpenToolkit.UnitProxy
{
    public abstract class CapeOpenUnitPort : CapeOpenIdBase, ICapeUnitPort, ECapeData, ECapeUnknown, ECapeInvalidArgument, ECapeOutOfBounds
    {
        private CapeOpenPortDirection mDirection;
        private CapeOpenObjectBase mConnectedObject;

        public static CapeOpenUnitPort Create(CapeOpenPortType type, string name, string description, CapeOpenPortDirection direction)
        {
            CapeOpenUnitPort zPort = null;

            switch (type)
            {
                case CapeOpenPortType.Material:
                    zPort = new MaterialPort();
                    break;
                default:
                    throw new CapeOpenException();
            }

            zPort.Name = name;
            zPort.Description = description;
            zPort.Direction = direction;
            return zPort;
        }

        //Costruttore
        protected CapeOpenUnitPort()
        {
        }

        public CapeOpenPortDirection Direction
        {
            get
            {
                return mDirection;
            }

            set
            {
                mDirection = value;
            }
        }

        public abstract CapeOpenPortType Type
        {
            get;
        }

        public CapeOpenObjectBase ConnectedObject
        {
            get
            {
                return mConnectedObject;
            }

            set
            {
                mConnectedObject = value;
            }
        }

        #region ICapeUnitPort

        CapePortDirection ICapeUnitPort.direction
        {
            get
            {
                return CapeOpenPortDirectionHelper.Map(Direction);
            }
        }

        CapePortType ICapeUnitPort.portType
        {
            get
            {
                return CapeOpenPortTypeHelper.Map(Type);
            }
        }

        object ICapeUnitPort.connectedObject
        {
            get
            {
                if (ConnectedObject is CapeOpenThermoMaterialObject)
                {
                    CapeOpenThermoMaterialObject zConnectedObject = (CapeOpenThermoMaterialObject)ConnectedObject;
                    return zConnectedObject.ConnectedObject;
                }
                else
                {
                    return ConnectedObject;
                }
            }
        }

        void ICapeUnitPort.Connect(object objectToConnect)
        {
            if (objectToConnect == null)
            {
                ATCOError = new CapeOpenError();
                ATCOError.UserCode = (int)eCapeErrorInterfaceHR_tag.ECapeInvalidArgumentHR;
                ATCOError.UserInterfaceName = "ICapeUnitPort";
                ATCOError.UserOperation = "Connect";
                ATCOError.UserDescription = "The input argument has not been initialized";
                throw (new COMException("Errore", (int)eCapeErrorInterfaceHR_tag.ECapeInvalidArgumentHR));
            }
            else
            {
                if (objectToConnect is ICapeThermoMaterialObject)
                {
                    ConnectedObject = CapeOpenThermoMaterialObject.Connect((ICapeThermoMaterialObject)objectToConnect);
                }
                else
                {
                    throw new CapeOpenException();
                }
            }
        }

        void ICapeUnitPort.Disconnect()
        {
            ConnectedObject = null;
        }

        #endregion
    }
}
