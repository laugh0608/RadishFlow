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
    public class CapeRealParameterTests
    {
        private CapeRealParameter param;

        [TestInitialize()]
        public void CapeRealParameterInit()
        {
            param = new CapeRealParameter("real",
                UnitCategoryEnum.Pressure, CapeParamMode.CAPE_INPUT,
                -100, 100, 10);
        }

        [TestMethod()]
        public void CapeRealParameterTest()
        {
            param = new CapeRealParameter();
            Assert.AreEqual(double.NaN, param.SIValue);
            param.value = 5;
            Assert.AreEqual(5, param.SIValue);
            Assert.AreEqual(5, param.DoubleValue);
            param.CurrentUnitCategory = UnitCategoryEnum.Temperature;
            param.CurrentUnit = "C";
            Assert.AreEqual(5, param.DoubleValue);
            Assert.AreEqual(278.15, param.SIValue);
            param.CurrentUnit = "K";
            Assert.AreEqual(5, param.SIValue);
        }

        [TestMethod()]
        public void CapeRealParameterTest1()
        {
            param = new CapeRealParameter("real",
                UnitCategoryEnum.Pressure, CapeParamMode.CAPE_INPUT,
                -100, 100, 10);
            Assert.AreEqual("real", param.ComponentName);
            Assert.AreEqual(10, param.DoubleValue);
            Assert.AreEqual(10, param.SIValue);
            Assert.AreEqual(-100, param.LowerBound);
            Assert.AreEqual(100, param.UpperBound);
            Assert.AreEqual(10, param.DefaultValue);
        }

        [TestMethod()]
        public void ResetTest()
        {
            param.value = 23.684;
            Assert.AreEqual(23.684, param.DoubleValue);
            Assert.AreNotEqual(param.DefaultValue, param.DoubleValue);
            param.Reset();
            Assert.AreEqual(param.DefaultValue, param.DoubleValue);
        }

        [TestMethod()]
        public void ValidateTest()
        {
            param.value = 23.684;
            Assert.IsTrue(param.Validate());
            param.value = -200;
            Assert.IsFalse(param.Validate());
            param.value = double.NaN;
            Assert.IsFalse(param.Validate());
            param.value = 200;
            Assert.IsFalse(param.Validate());
            param = new CapeRealParameter();
            Assert.IsFalse(param.Validate());
        }

        [TestMethod()]
        public void ValidateTest2()
        {
            string mes = null;
            Assert.IsTrue(param.Validate(23.684,ref mes));
            Assert.IsFalse(param.Validate(-200, ref mes));
            Assert.IsFalse(param.Validate(double.NaN, ref mes));
            Assert.IsFalse(param.Validate(200, ref mes));
            Assert.IsFalse(param.Validate(double.PositiveInfinity, ref mes));
        }

        [TestMethod()]
        public void CompareToTest()
        {
            param.DoubleValue = 20;
            Assert.AreEqual(CapeValidationStatus.CAPE_NOT_VALIDATED,param.ValStatus);
            Assert.AreEqual(0, param.CompareTo(20));
            Assert.AreNotEqual(0, param.CompareTo(0));
            Assert.AreNotEqual(0, param.CompareTo(double.NaN));
        }

        [TestMethod()]
        public void CompareToTest1()
        {
            var p1 = new CapeRealParameter();
            p1.DoubleValue = 67;
            Assert.AreNotEqual(0, param.CompareTo(p1));
            param.DoubleValue = 67;
            Assert.AreEqual(0, param.CompareTo(p1));
        }

        [TestMethod()]
        public void CompareToTest2()
        {
            var p1 = new CapeRealParameter();
            p1.DoubleValue = 20;
            param.DoubleValue = 20;
            Assert.AreEqual(0, param.CompareTo(p1.value.ToString()));
        }

        [TestMethod()]
        public void CloneTest()
        {
            param.DoubleValue = 1.5;
            param.CurrentUnit = "MPag";
            var p1 = param.Clone() as CapeRealParameter;
            Assert.AreNotSame(p1, param);
            Assert.AreEqual(p1.ComponentName, param.ComponentName);
            Assert.AreEqual(p1.ComponentDescription, param.ComponentDescription);
            Assert.AreEqual((double)p1.value, (double)param.DoubleValue);
            Assert.AreEqual(p1.LowerBound, param.LowerBound);
            Assert.AreEqual(p1.UpperBound, param.UpperBound);
            Assert.AreEqual(p1.DefaultValue, param.DefaultValue);
            Assert.AreEqual(p1.CurrentUnit, param.CurrentUnit);
            Assert.AreEqual(p1.SIValue, param.SIValue);
            Assert.AreEqual(p1.CurrentUnitCategory, param.CurrentUnitCategory);
        }
    }
}