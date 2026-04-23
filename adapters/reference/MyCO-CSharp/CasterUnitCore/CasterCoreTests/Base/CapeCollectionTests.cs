using Microsoft.VisualStudio.TestTools.UnitTesting;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using CasterCore;
using CasterCore.Parameters;
using CAPEOPEN;

namespace CasterCore.Tests
{
    [TestClass()]
    public class CapeCollectionTests
    {
        private static CapeCollection collection;
        private static CapeRealParameter realParam;
        private static CapeIntParameter intParam;

        [ClassInitialize]
        public static void CapeCollectionTest(TestContext context)
        {
            collection = new CapeCollection("collection", "a collection used to test");

            Assert.AreEqual(collection.ComponentName, "collection");
            Assert.AreEqual(collection.ComponentDescription, "a collection used to test");
        }

        [TestInitialize]
        public void AddDefaultItem()
        {
            collection.Clear();
            realParam = new CapeRealParameter("real",
                UnitCategoryEnum.Area, CapeParamMode.CAPE_INPUT);
            collection.Add(realParam);
            intParam = new CapeIntParameter("int", CapeParamMode.CAPE_INPUT);
            collection.Add(intParam);
        }

        [TestMethod()]
        public void GetItemTest()
        {
            Assert.AreEqual(collection.GetItem(0).Value.ComponentName, "real");
            Assert.AreEqual(collection.GetItem(0).Value.GetType(), typeof(CapeRealParameter));
            Assert.AreEqual(collection.GetItem(1).Value.ComponentName, "int");
        }

        [TestMethod()]
        [ExpectedException(typeof(ECapeUnknownException))]
        public void GetItemTest2()
        {
            collection.GetItem(2);
        }

        [TestMethod()]
        public void GetEnumeratorTest()
        {
            List<string> types = new List<string>();
            foreach (var i in collection)
            {
                types.Add(i.Key);
            }
            Assert.AreEqual(2, types.Count);
            Assert.AreEqual("real", types[0]);
            Assert.AreEqual("int", types[1]);
        }

        [TestMethod()]
        public void AddTest()
        {
            collection.Add(new CapeIntParameter("int2", CapeParamMode.CAPE_INPUT));
            Assert.AreEqual("int2", collection[2].ComponentName);
        }

        [TestMethod()]
        public void AddTest1()
        {
            collection.Add("int2", new CapeIntParameter("int2Obj", CapeParamMode.CAPE_INPUT));
            Assert.AreEqual("real", collection[0].ComponentName);
            Assert.AreEqual("int", collection[1].ComponentName);
            Assert.AreEqual("int2Obj", collection["int2"].ComponentName);
        }

        [TestMethod()]
        public void AddTest2()
        {
            collection.Add(new CapeCollectionPair("int3", new CapeIntParameter("int", CapeParamMode.CAPE_INPUT)));
            Assert.AreEqual("int", collection["int3"].ComponentName);
        }

        [TestMethod()]
        public void ClearTest()
        {
            collection.Clear();
            Assert.AreEqual(0, collection.Count);
            collection.Add(new CapeIntParameter("int", CapeParamMode.CAPE_INPUT));
            Assert.AreEqual("int", collection[0].ComponentName);
        }

        [TestMethod()]
        public void ContainsTest()
        {
            Assert.AreEqual(true, collection.Contains("real"));
            Assert.AreEqual(false, collection.Contains("real2"));
            Assert.AreEqual(false, collection.Contains((string)null));
        }

        [TestMethod()]
        public void ContainsTest1()
        {
            Assert.AreEqual(true, collection.Contains(realParam));
            Assert.AreEqual(true, collection.Contains(intParam));
            Assert.AreEqual(false, collection.Contains((CapeRealParameter)null));
            Assert.AreEqual(false, collection.Contains(
                new CapeIntParameter("real3", CapeParamMode.CAPE_INPUT)));
        }

        [TestMethod()]
        public void ContainsTest2()
        {
            Assert.AreEqual(true, collection.Contains(
                new CapeCollectionPair("real", realParam)));
            Assert.AreEqual(true, collection.Contains(
                new CapeCollectionPair("real", realParam)));
            Assert.AreEqual(false, collection.Contains(
                new CapeCollectionPair("int", realParam)));
            Assert.AreEqual(false, collection.Contains(
                new CapeCollectionPair("a", null)));
            Assert.AreEqual(false, collection.Contains(
                new CapeCollectionPair("a", new CapeIntParameter("a", CapeParamMode.CAPE_INPUT))));
        }

        [TestMethod()]
        public void CopyToTest()
        {
            CapeCollectionPair[] array = new CapeCollectionPair[5];
            collection.CopyTo(array, 1);
            Assert.AreEqual(null, array[0].Key);
            Assert.AreEqual("real", array[1].Value.ComponentName);
            Assert.AreEqual("int", array[2].Value.ComponentName);
            Assert.AreEqual(null, array[3].Key);
        }

        [TestMethod()]
        public void RemoveTest()
        {
            CapeCollectionPair p = new CapeCollectionPair("real", realParam);
            collection.Remove(p);
            Assert.AreEqual(1, collection.Count);
            Assert.AreEqual("int", collection[0].ComponentName);
        }

        [TestMethod()]
        public void RemoveTest1()
        {
            collection.Remove(realParam);
            Assert.AreEqual(1, collection.Count);
            Assert.AreEqual("int", collection[0].ComponentName);
        }

        [TestMethod()]
        public void RemoveTest2()
        {
            collection.Remove("real");
            Assert.AreEqual(1, collection.Count);
            Assert.AreEqual("int", collection[0].ComponentName);
        }

        [TestMethod()]
        public void CloneTest()
        {
            CapeCollection newCollection = collection.Clone() as CapeCollection;

            collection[0].ComponentDescription = "second";
            Assert.AreEqual("", newCollection[0].ComponentDescription);
            collection.Remove("real");
            Assert.AreEqual("real", newCollection[0].ComponentName);
        }
    }
}