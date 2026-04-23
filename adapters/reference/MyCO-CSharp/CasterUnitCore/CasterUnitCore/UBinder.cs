using CasterCore;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Runtime.Serialization;
using System.Text;
using System.Threading.Tasks;

namespace CasterUnitCore
{
    public class UBinder : SerializationBinder
    {
        public override Type BindToType(string assemblyName, string typeName)
        {
            Type type;
            Assembly unitCoreAsm = Assembly.GetExecutingAssembly();
            type = unitCoreAsm.GetType(typeName);
            if (type != null)
                return type;

            var asms = AppDomain.CurrentDomain.GetAssemblies();
            foreach (var asm in asms)
            {
                if(asm.FullName == assemblyName)
                {
                    type = asm.GetType(typeName);
                    if(type != null)
                        return type;
                }
            }

            CasterLogger.Error($"Type {typeName} in Assemly {assemblyName} is not found.");
            return typeof(object);
        }
    }
}
