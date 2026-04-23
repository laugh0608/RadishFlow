using System;
using System.Collections.Generic;
using System.Text;
using Mose.CapeOpenToolkit.UnitProxy;
using System.Runtime.InteropServices;

namespace CapeOpenExchangerCOCO
{
        [System.Runtime.InteropServices.GuidAttribute("24560354-d48e-433b-8820-0ab41ae90105")]
        public partial class ExchangerUnitCOCO
        {

            public ExchangerUnitCOCO()
                :
                    base()
            {
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

            protected override void OnInitialize()
            {
                Name = "COCO Exchanger Unit";
                Description = "COCO Exchanger Unit .Net";
                Ports.Add(CapeOpenUnitPort.Create(CapeOpenPortType.Material, "InputPort1", "Input Port 1 of Exchanger", CapeOpenPortDirection.Inlet));
                Ports.Add(CapeOpenUnitPort.Create(CapeOpenPortType.Material, "InputPort2", "Input Port 2 of Exchanger", CapeOpenPortDirection.Inlet));
                Ports.Add(CapeOpenUnitPort.Create(CapeOpenPortType.Material, "OutputPort1", "Output Port 1 of Splitter", CapeOpenPortDirection.Outlet));
                Ports.Add(CapeOpenUnitPort.Create(CapeOpenPortType.Material, "OutputPort2", "Output Port 2 of Splitter", CapeOpenPortDirection.Outlet));
            }
        }
}