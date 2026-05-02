using Microsoft.VisualStudio.TestTools.UnitTesting;
using CasterCore.Parameters;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using CAPEOPEN;

namespace CasterCore.Parameters.Tests
{
    [TestClass()]
    public class CapeBooleanParameterTests
    {
        CapeBooleanParameter param;

        [TestInitialize()]
        public void CapeBooleanParameterTestInit()
        {
            param = new CapeBooleanParameter("bool", CapeParamMode.CAPE_INPUT);
        }

        [TestMethod()]
        public void CapeBooleanParameterTest()
        {
            param = new CapeBooleanParameter();
            Assert.AreEqual(false, param.value);
            Assert.AreEqual(CapeParamMode.CAPE_INPUT_OUTPUT, param.Mode);
            param = new CapeBooleanParameter("bool", CapeParamMode.CAPE_INPUT);
            Assert.AreEqual(false, param.value);
            Assert.AreEqual("bool", param.ComponentName);
            Assert.AreEqual(CapeParamMode.CAPE_INPUT, param.Mode);
            param = new CapeBooleanParameter("bool", CapeParamMode.CAPE_INPUT, true);
            Assert.AreEqual(true, param.value);
        }

        [TestMethod()]
        public void ValidateTest()
        {
            Assert.IsTrue(param.Validate());
        }

        [TestMethod()]
        public void ValidateTest1()
        {
            string err = "";
            param.Validate(ref err);
            Assert.AreEqual("", err);
        }

        [TestMethod()]
        public void ResetTest()
        {
            param.value = true;
            param.Reset();
            Assert.AreEqual(false, param.value);
            param = new CapeBooleanParameter("bool", CapeParamMode.CAPE_INPUT, true);
            param.value = false;
            param.Reset();
            Assert.AreEqual(true, param.value);
        }

        [TestMethod()]
        public void ValidateTest2()
        {
            string err = "";
            param.Validate(false, ref err);
            Assert.AreEqual("", err);
        }

        [TestMethod()]
        public void CompareToTest()
        {
            Assert.AreEqual(0, param.CompareTo(false));
            Assert.IsTrue(param.CompareTo(true) != 0);
            param.value = true;
            Assert.AreNotEqual(0, param.CompareTo(false));
            Assert.AreEqual(0, param.CompareTo(true));
        }

        [TestMethod()]
        public void CompareToTest1()
        {
            param.value = false;
            var p1 = new CapeBooleanParameter();
            p1.value = false;
            Assert.AreEqual(0, param.CompareTo(p1));
            p1.value = true;
            Assert.AreNotEqual(0, param.CompareTo(p1));
        }

        [TestMethod()]
        public void EqualsTest()
        {
            param.BoolValue = false;
            ICapeParameter p1 = new CapeBooleanParameter();
            p1.value = false;
            Assert.AreEqual(0, param.CompareTo(p1));
            Assert.AreNotEqual(0, param.CompareTo(null));
            p1.value = true;
            Assert.AreNotEqual(0, param.CompareTo(p1));
            //Assert.AreEqual(false, param == p1);
            p1.value = false;
            Assert.AreEqual(0, param.CompareTo(((object)p1.value).ToString()));
        }

        [TestMethod()]
        public void CloneTest()
        {
            param.value = "false";
            var p1 = param.Clone() as CapeBooleanParameter;
            Assert.AreNotSame(param, p1);
            Assert.AreEqual(0, param.CompareTo(p1));
            Assert.AreEqual(true, param.DefaultValue == p1.DefaultValue);
            Assert.AreEqual(true, param.ComponentName == p1.ComponentName);
            Assert.AreEqual(true, param.ComponentDescription == p1.ComponentDescription);
            param.BoolValue = true;
            p1 = param.Clone() as CapeBooleanParameter;
            Assert.AreNotSame(param, p1);
            Assert.AreEqual(0, param.CompareTo(p1));
        }
    }
}