using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace CasterUnitCore.Reports
{
    /// <summary>
    /// Base class of unit reports
    /// </summary>
    public abstract class ReportBase
    {
        /// <summary>
        /// name of this report, should return a constant string
        /// </summary>
        public abstract string Name { get; }

        /// <summary>
        /// reference of the target unit operation, will be assigned automatically in Initialize() of unit operation block
        /// </summary>
        protected CasterUnitOperationBase UnitOp;

        /// <summary>
        /// Assign current unit operation
        /// </summary>
        /// <param name="unitOperation"></param>
        public void SetUnitOp(CasterUnitOperationBase unitOperation)
        {
            UnitOp = unitOperation;
        }

        /// <summary>
        /// product a report
        /// </summary>
        /// <returns>report to be returned</returns>
        public abstract string ProduceReport();
    }
}
