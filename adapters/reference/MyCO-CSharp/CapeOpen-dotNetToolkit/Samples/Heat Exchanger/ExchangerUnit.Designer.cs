#region Copyright (c) 2005-2007, Mose, All rights reserved.
/*
// Copyright (c) 2005-2007, Mose Srl (http://www.mose.units.it/)
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
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
using Mose.CapeOpenToolkit.UnitProxy;

namespace HeatExchanger
{
    [System.Runtime.InteropServices.GuidAttribute("14560354-d48e-433b-8820-0ab41ae90105")]
    public partial class ExchangerUnit
    {
        public ExchangerUnit()
            :
                base()
        {
        }

        public double ParameterValue
        {
            get
            {
                Mose.CapeOpenToolkit.UnitProxy.DoubleParameter zParam;
                zParam = ((Mose.CapeOpenToolkit.UnitProxy.DoubleParameter)(Parameters.GetParameterByName("ParameterValue")));
                return zParam.Value;
            }
            set
            {
                Mose.CapeOpenToolkit.UnitProxy.DoubleParameter zParam;
                zParam = ((Mose.CapeOpenToolkit.UnitProxy.DoubleParameter)(Parameters.GetParameterByName("ParameterValue")));
                zParam.Value = value;
            }
        }

        public Mose.CapeOpenToolkit.UnitProxy.CapeOpenThermoMaterialObject InputPort1
        {
            get
            {
                Mose.CapeOpenToolkit.UnitProxy.MaterialPort zPort;
                zPort = ((Mose.CapeOpenToolkit.UnitProxy.MaterialPort)(Ports.GetPortByName("InputPort1")));
                return ((Mose.CapeOpenToolkit.UnitProxy.CapeOpenThermoMaterialObject)(zPort.ConnectedObject));
            }
        }

        public Mose.CapeOpenToolkit.UnitProxy.CapeOpenThermoMaterialObject InputPort2
        {
            get
            {
                Mose.CapeOpenToolkit.UnitProxy.MaterialPort zPort;
                zPort = ((Mose.CapeOpenToolkit.UnitProxy.MaterialPort)(Ports.GetPortByName("InputPort2")));
                return ((Mose.CapeOpenToolkit.UnitProxy.CapeOpenThermoMaterialObject)(zPort.ConnectedObject));
            }
        }

        public Mose.CapeOpenToolkit.UnitProxy.CapeOpenThermoMaterialObject OutputPort1
        {
            get
            {
                Mose.CapeOpenToolkit.UnitProxy.MaterialPort zPort;
                zPort = ((Mose.CapeOpenToolkit.UnitProxy.MaterialPort)(Ports.GetPortByName("OutputPort1")));
                return ((Mose.CapeOpenToolkit.UnitProxy.CapeOpenThermoMaterialObject)(zPort.ConnectedObject));
            }
        }

        public Mose.CapeOpenToolkit.UnitProxy.CapeOpenThermoMaterialObject OutputPort2
        {
            get
            {
                Mose.CapeOpenToolkit.UnitProxy.MaterialPort zPort;
                zPort = ((Mose.CapeOpenToolkit.UnitProxy.MaterialPort)(Ports.GetPortByName("OutputPort2")));
                return ((Mose.CapeOpenToolkit.UnitProxy.CapeOpenThermoMaterialObject)(zPort.ConnectedObject));
            }
        }

        public OperationType Operation
        {
            set
            {
                mOperation = value;
            }
            get
            {
                return mOperation;
            }
        }

        protected override void OnInitialize()
        {
            Name = "Exchanger Unit";
            Description = "Exchanger Unit .Net";
            Ports.Add(CapeOpenUnitPort.Create(CapeOpenPortType.Material, "InputPort1", "Input Port 1 of Exchanger", CapeOpenPortDirection.Inlet));
            Ports.Add(CapeOpenUnitPort.Create(CapeOpenPortType.Material, "InputPort2", "Input Port 2 of Exchanger", CapeOpenPortDirection.Inlet));
            Ports.Add(CapeOpenUnitPort.Create(CapeOpenPortType.Material, "OutputPort1", "Output Port 1 of Splitter", CapeOpenPortDirection.Outlet));
            Ports.Add(CapeOpenUnitPort.Create(CapeOpenPortType.Material, "OutputPort2", "Output Port 2 of Splitter", CapeOpenPortDirection.Outlet));
            Parameters.Add(DoubleParameter.Create("ParameterValue", "Value of the parameter depending on calculation", CapeOpenParameterMode.ReadWrite, -1E+35, 1E+35, 250));
        }
    }
}
