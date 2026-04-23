using Microsoft.VisualStudio.TestTools.UnitTesting;
using CasterCore.Parameters;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace CasterCore.Parameters.Tests
{
    [TestClass()]
    public class UnitsTests
    {
        [TestMethod()]
        public void GetUnitListTest()
        {
            Assert.IsTrue(Units.GetUnitList(UnitCategoryEnum.Temperature).Count() > 2);
            Assert.IsTrue(Units.GetUnitList(UnitCategoryEnum.Pressure).Count() > 2);
        }

        [TestMethod()]
        public void UnitConvertTest()
        {
            Assert.AreEqual(100,
                Units.UnitConvert("C", 373.15, "K"));
            Assert.AreEqual(0,
                Units.UnitConvert("kPag", 101.325, "kPa"));
            Assert.AreEqual(2,
                Units.UnitConvert("bar", 200, "kPa"));
        }

        [TestMethod()]
        public void ConvertToSITest()
        {
            Assert.AreEqual(373.15,
                Units.ConvertToSI(100, "C"));
            Assert.AreEqual(0,
                Units.ConvertToSI(-101.325, "kPag"));
            Assert.AreEqual(1000,
                Units.ConvertToSI(1, "kmol/s"));
        }

        [TestMethod()]
        public void ConvertFromSITest()
        {
            Assert.AreEqual(0,
                Units.ConvertFromSI("C", 273.15));
            Assert.AreEqual(0,
                Units.ConvertFromSI("kPag", 101.325));
            Assert.AreEqual(1,
                Units.ConvertFromSI("kmol/s", 1000));
        }

        [TestMethod()]
        public void GetSIUnitTest()
        {
            Assert.AreEqual("K", Units.GetSIUnit(UnitCategoryEnum.Temperature));
            Assert.AreEqual("kPa", Units.GetSIUnit(UnitCategoryEnum.Pressure));
            Assert.AreEqual("mol/s", Units.GetSIUnit(UnitCategoryEnum.MolarFlowRate));
        }

        [TestMethod()]
        public void GetDimensionalityTest()
        {
            double[] dimen=new double[] 
            {
                1,1,-2,0,0,0,0,0
            };
            var temp = Units.GetDimensionality(UnitCategoryEnum.Force);
            Assert.IsTrue(temp.SequenceEqual(dimen));
        }

        [TestMethod()]
        public void TryGetUnitCategoryTest()
        {
            Assert.AreEqual(UnitCategoryEnum.Temperature, Units.TryGetUnitCategory("温度"));
        }

        [TestMethod()]
        public void SearchUnitCategoryByUnitNameTest()
        {
            Assert.AreEqual(UnitCategoryEnum.Temperature,
                Units.SearchUnitCategoryByUnitName("C"));
            Assert.AreEqual(UnitCategoryEnum.MolarFlowRate,
                Units.SearchUnitCategoryByUnitName("mol/s"));
            Assert.AreEqual(UnitCategoryEnum.Pressure,
                Units.SearchUnitCategoryByUnitName("bar"));
            Assert.AreEqual(UnitCategoryEnum.MassFlowRate,
               Units.SearchUnitCategoryByUnitName("kg/s"));
        }
    }
}