using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using CAPEOPEN;
using CasterCore;

namespace CasterUnitCore.Reports
{
    class LastRunReport : ReportBase
    {
        public override string Name => "Last run";

        public override string ProduceReport()
        {
            string report = null;

            //material
            int inlet = 0, outlet = 0, inoutlet = 0;
            foreach (ICapeUnitPort p in UnitOp.Ports.Values)
            {
                switch (p.direction)
                {
                    case CapePortDirection.CAPE_INLET:
                        inlet++;
                        break;
                    case CapePortDirection.CAPE_OUTLET:
                        outlet++;
                        break;
                    case CapePortDirection.CAPE_INLET_OUTLET:
                        inoutlet++;
                        break;
                }
            }
            report =
                $"Number of Inlet Stream :      {inlet}\n" +
                $"Number of Outlet Stream :     {outlet}\n" +
                $"Number of In & Out Stream :   {inoutlet}\n\n";

            foreach (var p in UnitOp.Results.Values)
            {
                report += $"{p.ComponentName} :\t{(p as ICapeParameter).value} {(p as CapeRealParameter)?.CurrentUnit}\n";
            }

            return report;
        }
    }
}
