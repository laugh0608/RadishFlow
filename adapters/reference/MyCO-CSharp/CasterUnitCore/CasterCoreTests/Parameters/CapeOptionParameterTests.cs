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
    public class CapeOptionParameterTests
    {
        private enum OptionEnum
        {
            option1, option2, option3, option8
        }

        private enum OptionEnum2
        {
            option1, option5, option4
        }

        private CapeOptionParameter param;

        [TestInitialize()]
        public void CapeOptionParameterInit()
        {
            param = new CapeOptionParameter("option", typeof(OptionEnum), CapeParamMode.CAPE_INPUT
                , OptionEnum.option2, true);
        }

        [TestMethod()]
        public void CapeOptionParameterTest()
        {
            param = new CapeOptionParameter();
            Assert.AreEqual(null, param.StringValue);
        }

        [TestMethod()]
        public void CapeOptionParameterTest1()
        {
            param = new CapeOptionParameter("option", typeof(OptionEnum), CapeParamMode.CAPE_INPUT
                , OptionEnum.option2, true);
            Assert.AreEqual("option2", param.StringValue);
            Assert.AreEqual(OptionEnum.option2, param.EnumValue);
            Assert.AreEqual("option2", param.value);
            Assert.AreEqual(4, (param.OptionList as string[]).Length);
        }

        [TestMethod()]
        public void CapeOptionParameterTest2()
        {
            param = new CapeOptionParameter("option",
                new[] { "option1", "option2" },
                CapeParamMode.CAPE_INPUT);
            Assert.AreEqual("option1", param.value);
            param.StringValue = "option2";
            Assert.AreEqual("option2", param.value);
            param.EnumValue = OptionEnum.option1;
            Assert.AreEqual("option1", param.value);
            param.EnumValue = OptionEnum.option8;
            Assert.AreEqual("option8", param.value);
        }

        [TestMethod()]
        public void ValidateTest()
        {
            param.StringValue = "option1";
            Assert.IsTrue(param.Validate());
            param.StringValue = "option8";
            Assert.IsTrue(param.Validate());
            param.StringValue = "option9";
            Assert.IsFalse(param.Validate());
            param.EnumValue = OptionEnum.option3;
            Assert.IsTrue(param.Validate());
            param.EnumValue = OptionEnum2.option1;
            Assert.IsTrue(param.Validate());
            param.EnumValue = OptionEnum2.option4;
            Assert.IsFalse(param.Validate());
            param.EnumType = typeof(OptionEnum2);
            Assert.IsTrue(param.Validate());
        }

        [TestMethod()]
        public void ResetTest()
        {
            param.StringValue = "option1";
            Assert.AreEqual(OptionEnum.option1, param.EnumValue);
            param.Reset();
            Assert.AreEqual(OptionEnum.option2, param.EnumValue);
        }

        [TestMethod()]
        public void ValidateTest2()
        {
            string mes = null;
            Assert.IsTrue(param.Validate("option1", ref mes));
            Assert.IsTrue(param.Validate("option8", ref mes));
            Assert.IsFalse(param.Validate("option9", ref mes));
        }

        [TestMethod()]
        public void ValidateTest3()
        {
            string mes = null;
            Assert.IsTrue(param.Validate(OptionEnum.option3, ref mes));
            Assert.IsTrue(param.Validate(OptionEnum2.option1, ref mes));
            Assert.IsFalse(param.Validate(OptionEnum2.option4, ref mes));
        }

        [TestMethod()]
        public void CompareToTest()
        {
            param.EnumValue = OptionEnum.option1;
            Assert.AreEqual(0, param.CompareTo(OptionEnum.option1));
            Assert.AreEqual(0, param.CompareTo(OptionEnum2.option1));
        }

        [TestMethod()]
        public void CompareToTest1()
        {
            var p1 = new CapeOptionParameter("option2", typeof(OptionEnum2), CapeParamMode.CAPE_INPUT);
            param.EnumValue = OptionEnum.option1;
            p1.EnumValue = OptionEnum2.option1;
            Assert.AreEqual(0, param.CompareTo(p1));
        }

        [TestMethod()]
        public void CompareToTest2()
        {
            var p1 = new CapeOptionParameter("option2", typeof(OptionEnum2), CapeParamMode.CAPE_INPUT);
            param.EnumValue = OptionEnum.option1;
            p1.EnumValue = OptionEnum2.option1;
            Assert.AreEqual(0, param.CompareTo("option1"));
            Assert.AreEqual(0, param.CompareTo(p1.value));
            Assert.AreEqual(0, param.CompareTo(p1.value.ToString()));
        }

        [TestMethod()]
        public void CloneTest()
        {
            var p1 = param.Clone() as CapeOptionParameter;
            Assert.AreNotSame(param, p1);
            Assert.AreEqual(param.value, p1.value);
            var optionList1 = param.OptionList as IEnumerable<string>;
            var optionList2 = p1.OptionList as IEnumerable<string>;
            Assert.IsTrue(optionList1.SequenceEqual(optionList2));
        }
    }
}