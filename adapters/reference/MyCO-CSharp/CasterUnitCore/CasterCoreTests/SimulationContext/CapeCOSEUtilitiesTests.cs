using Microsoft.VisualStudio.TestTools.UnitTesting;
using CasterCore;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace CasterCore.Tests
{
    [TestClass()]
    public class CapeCOSEUtilitiesTests
    {
        [TestInitialize()]
        public void SetSimulationContextTest()
        {
            CapeCOSEUtilities.SetSimulationContext(new SimpleSimulationContext());
        }

        [TestMethod()]
        public void NamedValueTest()
        {
            for (int i = 0; i < 2; i++)
            {
                var c = CapeCOSEUtilities.NamedValueList[i];
                var item = CapeCOSEUtilities.NamedValue(c);
                if (c == "name1")
                    Assert.AreEqual("value1", item);
                else if (c == "name2")
                    Assert.AreEqual("value2", item);
            }
        }
    }
}