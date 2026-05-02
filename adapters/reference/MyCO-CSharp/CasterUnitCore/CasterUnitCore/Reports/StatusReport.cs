using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace CasterUnitCore.Reports
{
    class StatusReport : ReportBase
    {
        public override string Name => "Status";

        public override string ProduceReport()
        {
            string report = null;

            UnitOp.Validate(ref report);

            if (string.IsNullOrWhiteSpace(report))
                report = "Complete.";

            return report;
        }
    }
}
