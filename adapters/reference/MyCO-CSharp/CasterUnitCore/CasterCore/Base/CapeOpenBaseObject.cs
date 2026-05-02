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
using System.Runtime.InteropServices;
using CAPEOPEN;

namespace CasterCore
{
    /// <summary>
    /// This class is the base class of all CO component, default setting is canRename and name is "COComponent"
    /// </summary>
    [Serializable]
    [ComVisible(true)]
    public abstract class CapeOpenBaseObject :
        ICapeIdentification, IDisposable,
        ECapeRoot, ECapeUnknown, ECapeUser
    {
        private string _name; //ComponentName
        private string _description;  //ComponentDescription
        /// <summary>
        /// whether the ComponentName can be modified, most simulator will change the name of unit, so dont set to false for unit operation
        /// </summary>
        protected bool CanRename;
        /// <summary>
        /// whether the component should be registered to CasterLocator
        /// </summary>
        protected readonly bool RegisterToLocator;

        #region Constructor

        /// <summary>
        /// must have a non-paramCollection constructor to compatible with COM
        /// </summary>
        protected CapeOpenBaseObject() : this("COComponent")
        {
        }

        /// <summary>
        /// Constructor of base class of all CapeOpen component
        /// </summary>
        /// <param name="name">ComponentName</param>
        /// <param name="description">description of component</param>
        /// <param name="canRename">if false, component will raise an exception when trying to change ComponentName</param>
        /// <param name="registerToLocator">should this block be registered to CasterLocator</param>
        protected CapeOpenBaseObject(string name = "COComponent", string description = "",
            bool canRename = true, bool registerToLocator = false)
        {
            _name = name;
            _description = description;
            CanRename = canRename;
            Dirty = false;
            RegisterToLocator = registerToLocator;

            if (RegisterToLocator)
                CasterLocator.Register(this);
        }

        #endregion

        #region ICapeIdentification

        /// <summary>
        /// Description of this component
        /// </summary>
        public string ComponentDescription
        {
            get { return _description; }
            set { _description = value; Dirty = true; }
        }

        /// <summary>
        /// Component Name, if canRename is false in the constructor, will raise an error if facing changes
        /// </summary>
        public string ComponentName
        {
            get { return _name; }
            set
            {
                if (ComponentName == value)
                    return;
                if (!CanRename)
                    throw new ECapeUnknownException(this,
                        $"Rename Failed: {ComponentName} cannot be renamed.",
                        null, "ICapeIdentification");
                string oldname = _name;
                _name = value;
                Dirty = true;
                if (RegisterToLocator)
                {
                    CasterLocator.UnRegister(oldname);
                    CasterLocator.Register(this);
                }
            }
        }

        /// <summary>
        /// whether the component has been modified
        /// </summary>
        public bool Dirty { get; set; }

        #endregion

        #region Error Handle

        string _errorDesc;
        string _errorIface;
        string _errorScope;

        #region ECapeRoot

        /// <summary>
        /// A short description of the error.
        /// </summary>
        string ECapeRoot.name { get { return _errorDesc; } }

        #endregion

        #region ECapeUser

        /// <summary>
        /// Code to designate the subcategory of the error.
        /// </summary>
        int ECapeUser.code { get { return 0; } }
        /// <summary>
        /// The description of the error.
        /// </summary>
        string ECapeUser.description { get { return _errorDesc; } }
        /// <summary>
        /// The name of the interface where the error is thrown. 
        /// </summary>
        string ECapeUser.interfaceName { get { return _errorIface; } }
        /// <summary>
        /// An URL to a page, document, web site, … where more information on the error can be found.
        /// </summary>
        string ECapeUser.moreInfo { get { return "Visit https://github.com/liuboqian"; } }
        /// <summary>
        /// The name of the operation where the error is thrown. 
        /// </summary>
        string ECapeUser.operation { get { return "N/A"; } }
        /// <summary>
        /// The scope of the error. The list of packages where the error occurs separated by "::". For example CapeOpen::Common::Identification.
        /// </summary>
        string ECapeUser.scope { get { return _errorScope; } }

        /// <summary>
        /// Set the current error occurs
        /// </summary>
        public void SetError(string errorDesc, string errorIface, string errorScope)
        {
            this._errorDesc = errorDesc;
            this._errorIface = errorIface;
            this._errorScope = errorScope;
        }

        #endregion

        #endregion

        #region IDisposable

        /// <summary>
        /// The only function of Dispose is to unregister this block, if override, remember to unregister
        /// </summary>
        public void Dispose()
        {
            Dispose(true);
            GC.SuppressFinalize(this);
        }

        /// <summary>
        /// This method should release all non managered object and UnRegister this object from CasterLocator.
        /// Whether disposing is true or false, COM resource should be released.
        /// </summary>
        /// <param name="disposing">true if invoked by user, false if invoked by GC</param>
        protected virtual void Dispose(bool disposing)
        {
            if (RegisterToLocator)
                CasterLocator.UnRegister(this);
            if (disposing)
            {
            }
        }

        /// <summary>
        /// Finalizer, don't invoke manually
        /// </summary>
        ~CapeOpenBaseObject()
        {
            // Finalizer calls Dispose(false)  
            Dispose(false);
        }

        #endregion
    }//class
}
