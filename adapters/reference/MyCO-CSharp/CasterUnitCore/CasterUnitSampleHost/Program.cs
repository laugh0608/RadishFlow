using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using CasterUnitSample;

namespace CasterUnitSampleHost
{
    class Program
    {
        static void Main(string[] args)
        {
            CasterUnitSample.CasterUnitSample unit = new CasterUnitSample.CasterUnitSample();
            Console.WriteLine("Create unit");

            var componenttype = System.Type.GetTypeFromProgID("CasterUnitSample1.1");
            var unit2 = System.Activator.CreateInstance(componenttype);
            if (unit2 != null)
            {
                Console.WriteLine("Create unit from COM");
            }

            componenttype = System.Type.GetTypeFromCLSID(new Guid("7CF9589B-19D6-4B1F-B259-0E0413A88968"));
            unit2 = System.Activator.CreateInstance(componenttype);
            if (unit2 != null)
            {
                Console.WriteLine("Create unit from COM");
            }

            Console.ReadKey();
        }
    }
}
