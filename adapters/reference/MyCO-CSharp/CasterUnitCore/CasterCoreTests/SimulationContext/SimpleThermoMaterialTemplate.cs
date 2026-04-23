using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using CAPEOPEN;
using System.Diagnostics;

namespace CasterCore.Tests
{
    public class SimpleThermoMaterialTemplate : ICapeThermoMaterialTemplate
    {
        public object CreateMaterialObject()
        {
            return new SimpleMaterialObject10();
        }

        public void SetProp(string property, object values)
        {
            Debug.WriteLine($"Property {property} is set to {values}");
        }
    }
}
