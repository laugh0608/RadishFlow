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
    public class CasterLocatorTests
    {
        [TestInitialize()]
        public void Init()
        {
            CasterLocator.Clear();
        }

        [TestMethod()]
        public void RegisterTest()
        {
            CapeCollection c1 = new CapeCollection("collection1");
            CasterLocator.Register(c1);
            Assert.AreSame(c1,CasterLocator.GetInstance("collection1"));
        }

        [TestMethod()]
        public void UnRegisterTest()
        {
            CapeCollection c1 = new CapeCollection("collection1");
            CasterLocator.Register(c1);
            CasterLocator.UnRegister("collection1");
            Assert.AreEqual(0, CasterLocator.Keys.Count());
        }

        [TestMethod()]
        public void UnRegisterTest1()
        {
            CapeCollection c1 = new CapeCollection("collection1");
            CasterLocator.Register(c1);
            CasterLocator.UnRegister(c1);
            Assert.AreEqual(0, CasterLocator.Keys.Count());
        }

        [TestMethod()]
        public void GetInstanceTest()
        {
            CapeCollection c1 = new CapeCollection("collection1");
            CasterLocator.Register(c1);
            Assert.AreSame(c1, CasterLocator.GetInstance("collection1"));
        }
    }
}