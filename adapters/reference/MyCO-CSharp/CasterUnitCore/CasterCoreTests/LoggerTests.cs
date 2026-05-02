using Microsoft.VisualStudio.TestTools.UnitTesting;
using CasterCore;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.IO;

namespace CasterCore.Tests
{
    [TestClass()]
    public class LoggerTests
    {
        [TestMethod()]
        public void DebugTest()
        {
            CasterLogger.Debug("Debug message.");
            CasterLogger.Shutdown();
            var logfile = File.ReadAllLines(@"log\logfile.log");
            var content = logfile[logfile.Length - 1];
            Assert.IsTrue(content.EndsWith("Debug message."));
            var dateStr = content.Substring(0, 19); // example: 2020-05-01 22:00:09
            var date = Convert.ToDateTime(dateStr);
            Assert.IsTrue(DateTime.Now - date < new TimeSpan(0, 0, 1)); // less then a second
        }

    }
}