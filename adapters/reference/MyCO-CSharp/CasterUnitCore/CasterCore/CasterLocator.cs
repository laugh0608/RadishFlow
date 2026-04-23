/*Copyright 2016 Caster

* Licensed under the Apache License, Version 2.0 (the "License");
* you may not use this file except in compliance with the License.
* You may obtain a copy of the License at
* 
*     http://www.apache.org/licenses/LICENSE-2.0
* 
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific language governing permissions and
* limitations under the License.*/

using System;
using System.Collections.Generic;
using CAPEOPEN;
using System.Linq;

namespace CasterCore
{
    /// <summary>
    /// Global locator for registered blocks
    /// </summary>
    public static class CasterLocator
    {
        static Dictionary<string, CapeOpenBaseObject> _components = new Dictionary<string, CapeOpenBaseObject>();

        /// <summary>
        /// Register a component, the id is ComponentName of the component, should be unique
        /// </summary>
        /// <param name="component">component to be added, normally a unit operation component or a property package</param>
        public static void Register(CapeOpenBaseObject component)
        {
            string name = component.ComponentName;
            CasterLogger.Debug($"Register component {name}.");
            if (_components.ContainsKey(name))
            {
                if (_components[name] == component)
                    return;
                //else   // cofe will load component in calculation, that leads to multiple instance with same name
                //    throw new ECapeUnknownException(
                //        $"Register Failed : Already create a different component with a same id : {name}.");
            }
            else
                _components.Add(name, component);
        }

        /// <summary>
        /// UnRegister a component with its name
        /// </summary>
        public static void UnRegister(string name)
        {
            CasterLogger.Debug($"UnRegister component {name}.");
            //if (!_components.ContainsKey(name))
            //    throw new ECapeUnknownException(
            //        $"UnRegister Failed : Key \"{name}\" is not registered.");
            //else
                _components.Remove(name);
        }

        /// <summary>
        /// UnRegister a component 
        /// </summary>
        public static void UnRegister(CapeOpenBaseObject component)
        {
            string name = component.ComponentName;
            CasterLogger.Debug($"UnRegister component {name}.");
            if (!_components.ContainsKey(name))
                return;
            //throw new ECapeUnknownException(
            //    $"UnRegister Failed : Key \"{name}\" is not registered.");
            else
                _components.Remove(name);
        }

        /// <summary>
        /// Get the instance of a component
        /// </summary>
        /// <param name="name">Component name of the component</param>
        /// <returns></returns>
        public static CapeOpenBaseObject GetInstance(string name)
        {
            if (_components.ContainsKey(name))
                return _components[name];
            else
                throw new ECapeUnknownException(
                    $"Load Failed : Key \"{name}\" is not registered.");
        }

        /// <summary>
        /// return all registered Components
        /// </summary>
        public static Dictionary<string, CapeOpenBaseObject> Components => new Dictionary<string, CapeOpenBaseObject>(_components);

        /// <summary>
        /// return all registered Components keys
        /// </summary>
        public static IEnumerable<string> Keys => _components.Keys;
        
        /// <summary>
        /// return all registered unit operations
        /// </summary>
        public static IEnumerable<ICapeUnit> UnitOperations
        {
            get
            {
                return from obj in _components.Values
                       where obj is ICapeUnit
                       select obj as ICapeUnit;
            }
        }

        /// <summary>
        /// return the PropertyPackageManager, throw exception if not existed or multiply instances existed
        /// </summary>
        public static ICapeThermoPropertyPackageManager PropertyPackageManager
        {
            get
            {
                try
                {
                    return (from obj in _components.Values
                            where obj is ICapeThermoPropertyPackageManager
                            select obj as ICapeThermoPropertyPackageManager).Single();
                }
                catch (ArgumentNullException)
                {
                    throw new ECapeUnknownException(
                        $"PropertyPackageManager is not existed.");
                }
                catch (InvalidOperationException)
                {
                    throw new ECapeUnknownException(
                        "More than one PropertyPackageManager existed.");
                }
            }
        }

        /// <summary>
        /// return all registered PropertyPackages
        /// </summary>
        public static IEnumerable<ICapeThermoPropertyPackage> PropertyPackages
        {
            get
            {
                return from obj in _components.Values
                       where obj is ICapeThermoPropertyPackage
                       select obj as ICapeThermoPropertyPackage;
            }
        }

        /// <summary>
        /// delete all registered blocks
        /// </summary>
        public static void Clear()
        {
            CasterLogger.Debug("CasterLocator clear");
            _components.Clear();
        }
    }
}
