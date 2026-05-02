using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using CAPEOPEN;

namespace CasterCore.Tests
{
    public class SimpleSimulationContext :
        ICapeSimulationContext, ICapeDiagnostic,
        ICapeCOSEUtilities, ICapeMaterialTemplateSystem
    {
        public List<string> names;
        public List<object> value;

        public SimpleSimulationContext()
        {
            names = new List<string>() { "name1", "name2" };
            value = new List<object>() { "value1", "value2" };
        }

        public void PopUpMessage(string message)
        {
            Console.WriteLine($"PopUp:{message}");
        }

        public void LogMessage(string message)
        {
            Console.WriteLine($"Log:{message}");
        }


        public object NamedValueList
        {
            get
            {
                return names;
            }
        }

        public object get_NamedValue(string name)
        {
            return value[names.IndexOf(name)];
        }

        public object CreateMaterialTemplate(string materialTemplateName)
        {
            return new SimpleThermoMaterialTemplate();
        }

        public object MaterialTemplates
        {
            get { return new string[] { "Temp1", "Temp2" }; }
        }
    }
}
