using Microsoft.VisualStudio.TestTools.UnitTesting;
using CasterCore.Parameters;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using CasterCore;
using CAPEOPEN;

namespace CasterCore.Parameters.Tests
{
    [TestClass()]
    public class CapeIntParameterTests
    {
        CapeIntParameter param;

        [TestInitialize()]
        public void CapeBooleanParameterTestInit()
        {
            param = new CapeIntParameter("int", CapeParamMode.CAPE_INPUT, -100, 100, 10);
        }

        [TestMethod()]
        public void CapeIntParameterTest()
        {
            param = new CapeIntParameter("int", CapeParamMode.CAPE_INPUT, -100, 100, 10);
            Assert.AreEqual("int", param.ComponentName);
            Assert.AreEqual(10, param.IntValue);
            Assert.AreEqual(-100, param.LowerBound);
            Assert.AreEqual(100, param.UpperBound);
        }

        [TestMethod()]
        public void CapeIntParameterTest1()
        {
            param = new CapeIntParameter();
            Assert.AreEqual(default(int), param.IntValue);
        }

        [TestMethod()]
        public void ValidateTest()
        {
            Assert.AreEqual(true, param.Validate());
            param.IntValue = -1000;
            Assert.AreEqual(false, param.Validate());
        }

        [TestMethod()]
        public void ValidateTest1()
        {
            param.IntValue = -1000;
            string message = null;
            Assert.AreEqual(false, param.Validate(ref message));
            Assert.AreEqual("value is out of range", message);
        }

        [TestMethod()]
        public void ResetTest()
        {
            param.IntValue = 435;
            Assert.AreEqual(435, param.IntValue);
            param.Reset();
            Assert.AreEqual(param.DefaultValue, param.IntValue);
            Assert.AreEqual(CapeValidationStatus.CAPE_NOT_VALIDATED, param.ValStatus);
        }

        [TestMethod()]
        public void ValidateTest2()
        {
            string message = null;
            Assert.IsFalse(param.Validate(-34588, ref message));
            Assert.IsTrue(param.Validate(50, ref message));
        }

        [TestMethod()]
        public void CompareToTest()
        {
            param.IntValue = 20;
            Assert.AreNotEqual(0, param.CompareTo(845));
        }

        [TestMethod()]
        public void CompareToTest1()
        {
            param.IntValue = 20;
            object p1 = new CapeIntParameter("int2",CapeParamMode.CAPE_INPUT,defaultVal:234);
            Assert.AreNotEqual(0, param.CompareTo(p1));
        }

        [TestMethod()]
        public void CompareToTest2()
        {
            param.IntValue = 20;
            ICapeIntegerParameterSpec p1 = new CapeIntParameter("int2", CapeParamMode.CAPE_INPUT, defaultVal: 895);
            Assert.AreNotEqual(0, param.CompareTo(p1));
            (p1 as ICapeParameter).value = 20;
            Assert.AreEqual(0, param.CompareTo(p1));
            Assert.AreEqual(0, param.CompareTo("20"));
        }

        [TestMethod()]
        public void CloneTest()
        {
            var p1 = param.Clone() as CapeIntParameter;
            Assert.AreNotSame(p1, param);
            Assert.AreEqual(p1.ComponentName, param.ComponentName);
            Assert.AreEqual(p1.ComponentDescription, param.ComponentDescription);
            Assert.AreEqual((int)p1.value, (int)param.IntValue);
            Assert.AreEqual(p1.LowerBound, param.LowerBound);
            Assert.AreEqual(p1.UpperBound, param.UpperBound);
            Assert.AreEqual(p1.DefaultValue, param.DefaultValue);
        }
    }
}