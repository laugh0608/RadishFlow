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
using System.Runtime.InteropServices;
using CAPEOPEN;
using System.Runtime.Serialization;
using System.Reflection;
using System.Diagnostics;
using System.Linq;

namespace CasterCore
{
    /// <summary>
    /// Option Parameters, can be initialize by enum, can compare to string or enum.
    /// The comparision between parameter and string, enum or other option parameters 
    /// will only compare their value, ignoring their OptionList or enum type.
    /// Because the user might want to compare between diffent parameters just to compare their value,
    /// for example two enum ReactPhase and MaterialPhase.
    /// </summary>
    [Serializable]
    [ComVisible(true)]
    [Guid("8EB0F647-618C-4fcc-A16F-39A9D57EA72E")]
    [ComDefaultInterface(typeof(ICapeParameter))]
    public class CapeOptionParameter
        : CapeParameterBase,
        ICapeOptionParameterSpec, IComparable<string>,
        IComparable<ICapeOptionParameterSpec>, IComparable<Enum>
    {
        private string _curString;
        private Enum _curEnum;
        private IEnumerable<string> _optionList;

        [NonSerialized]
        private Type _enumType;
        /// <summary>
        /// This string is used to deserialize enumType, only used when serialize and deserialize, no need to concern in other time.
        /// </summary>
        private string enumTypeName;
        /// <summary>
        /// used to retrive assembly and get type
        /// </summary>
        private string assemblyName;

        #region Constructor

        /// <summary>
        /// default name is "optionParameter" without any option
        /// </summary>
        public CapeOptionParameter()
            : this("optionParameter", (IEnumerable<string>)null,
                  CapeParamMode.CAPE_INPUT_OUTPUT)
        { }

        /// <summary>
        /// use string array to initialize, better use enum.
        /// </summary>
        public CapeOptionParameter(string name, IEnumerable<string> optionList,
            CapeParamMode mode, string defOption = null, bool restrictedToList = true)
            : base(name, CapeParamType.CAPE_OPTION, mode, UnitCategoryEnum.Dimensionless)
        {
            this._enumType = null;
            if (optionList != null)
                _optionList = new List<string>(optionList);
            else
                _optionList = new List<string>();
            RestrictedToList = restrictedToList;

            if (_optionList.Count() == 0)
                DefaultValue = null;
            else if (defOption == null)
                DefaultValue = _optionList.FirstOrDefault();
            else
                DefaultValue = defOption;

            StringValue = DefaultValue;
        }

        /// <summary>
        /// Highly Recommended! Use a enum type to initialize this parameter
        /// </summary>
        public CapeOptionParameter(string name, Type enumType, CapeParamMode mode,
            Enum defOption = null, bool restrictedToList = true)
            : base(name, CapeParamType.CAPE_OPTION, mode, UnitCategoryEnum.Dimensionless)
        {
            this._enumType = enumType;
            _optionList = new List<string>(enumType.GetEnumNames());
            RestrictedToList = restrictedToList;

            if (_optionList.Count() == 0)
                DefaultValue = null;
            else if (defOption == null)
                DefaultValue = _optionList.FirstOrDefault();
            else
                DefaultValue = enumType.GetEnumName(defOption);

            StringValue = DefaultValue;
        }
        #endregion

        #region CapeParameterBase

        /// <summary>
        /// Check whether the value must be restricted to list and the option is not in the list
        /// </summary>
        /// <returns></returns>
        public override bool Validate()
        {
            string message = "";
            return Validate(ref message);
        }
        /// <summary>
        /// Check whether the value must be restricted to list and the option is not in the list
        /// </summary>
        /// <paramCollection name="message"></paramCollection>
        /// <returns></returns>
        public override bool Validate(ref string message)
        {
            bool isAvailable;
            isAvailable = Validate(StringValue, ref message);

            ValStatus = isAvailable
                ? CapeValidationStatus.CAPE_VALID
                : CapeValidationStatus.CAPE_INVALID;
            return isAvailable;
        }

        /// <summary>
        /// reset value to default value
        /// </summary>
        public override void Reset()
        {
            StringValue = DefaultValue;
        }

        /// <summary>
        /// Accept string or enum or ICapeOptionParameterSpec,
        /// the enum is not specifically the enum type in initializing, 
        /// any one with the same string value is acceptable.
        /// </summary>
        public override object value
        {
            get { return StringValue; }
            set
            {
                try
                {
                    if (value is ICapeOptionParameterSpec)
                    {
                        StringValue = (string)((ICapeParameter)value).value;
                    }
                    else if (value is Enum)
                    //&& _optionList.Contains(value.ToString()))
                    //value.GetType() == enumType)
                    {
                        EnumValue = (Enum)value;
                    }
                    else
                    {
                        StringValue = Convert.ToString(value);
                    }
                }
                catch (Exception)
                {
                    throw new ECapeUnknownException(this,
                        $"value {value} is not string or enum or ICapeOptionParameterSpec");
                }
            }
        }

        /// <summary>
        /// value with type
        /// </summary>
        public string StringValue
        {
            get => _curString;
            set
            {
                if (_curString == value)
                    return;
                //should be checked in Validate()
                //if (RestrictedToList)
                //{
                //    if (!_optionList.Contains(value))
                //        throw new ECapeUnknownException(this,
                //            $"{ComponentName} doesn't accept {value}. The parameter is restricted to option list.");
                //}
                _curString = value;
                if (_enumType != null)
                {
                    foreach (var e in Enum.GetValues(_enumType))
                        if (value == e.ToString())
                            _curEnum = (Enum)e;
                    //throw new ECapeUnknownException(,"No enum is equal to the current value.");
                }
                Dirty = true;
                ValStatus = CapeValidationStatus.CAPE_NOT_VALIDATED;
            }
        }

        /// <summary>
        /// value with type
        /// </summary>
        public Enum EnumValue
        {
            get => _curEnum;
            set
            {
                if (_curEnum == value)
                    return;
                _curEnum = value;
                _curString = value.ToString();
                Dirty = true;
                ValStatus = CapeValidationStatus.CAPE_NOT_VALIDATED;
            }
        }

        /// <summary>
        /// The enum type of this option param.
        /// </summary>
        public Type EnumType
        {
            get => _enumType;
            set
            {
                if (_enumType == value)
                    return;
                _enumType = value;
                _optionList = new List<string>(((Type)value).GetEnumNames());
                Dirty = true;
                ValStatus = CapeValidationStatus.CAPE_NOT_VALIDATED;
            }
        }

        #endregion

        #region ICapeOptionParameterSpec

        /// <summary>
        /// Check whether the value must be restricted to list and the option is not in the list
        /// </summary>
        public bool Validate(string value, ref string message)
        {
            if (RestrictedToList)
            {
                if (_optionList.Contains(value))
                    return true;
                else
                {
                    message = "OptionList doesn't contain current value, and it must be restricted to list";
                    return false;
                }
            }
            else
            {
                return true;
            }
        }

        /// <summary>
        /// Check the value is avaliable to this option param
        /// </summary>
        public bool Validate(Enum value, ref string message)
        {
            return Validate(value.ToString(), ref message);
        }

        /// <summary>
        /// get and set option list, can be set to enum Type
        /// </summary>
        public object OptionList
        {
            get { return _optionList.ToArray(); }
            set
            {
                if (value is IEnumerable<string>)
                {
                    _optionList = new List<string>(value as IEnumerable<string>);
                    _enumType = null;
                }
                else if (value is Type)
                {
                    EnumType = value as Type;
                }
                else
                    throw new ECapeUnknownException(
                        $"Value {value} must be a enum or IEnumerable<string>");
            }
        }

        /// <summary>
        /// default value
        /// </summary>
        public string DefaultValue { get; set; }

        /// <summary>
        /// whether the value can be outside of option list
        /// </summary>
        public bool RestrictedToList { get; set; }

        #endregion

        #region IComparable

        /// <summary>
        /// Compare the option parameter to a string
        /// </summary>
        public int CompareTo(string other)
        {
            return StringValue.CompareTo(other);
        }

        /// <summary>
        /// Compare between bool parameters, return 1 if other is null
        /// </summary>
        public int CompareTo(ICapeOptionParameterSpec other)
        {
            return CompareTo((object)other);
        }

        /// <summary>
        /// Compare between option parameters or enum, only compare its value, ignore OptionList, return 1 if other is null
        /// </summary>
        public int CompareTo(object other)
        {
            switch (other)
            {
                case null:
                    return 1;
                case ICapeOptionParameterSpec optionObj:
                    return CompareTo((string)(other as ICapeParameter).value);
                case Enum enumObj:
                    return CompareTo(other.ToString());
                default:
                    string v = null;
                    try
                    {
                        v = Convert.ToString(other);
                    }
                    catch (Exception e)
                    {
                        throw new ECapeUnknownException(this,
                            $"value {other} can't convert to string",
                            e);
                    }
                    return CompareTo(v);
            }
        }

        /// <summary>
        /// Compare option parameters and a enum, only compare its value, ignore OptionList, return 1 if other is null
        /// </summary>
        public int CompareTo(Enum other)
        {
            if (other == null)
                return 1;
            return CompareTo(other.ToString());
        }

        ///// <summary>
        ///// Compare between option parameters
        ///// </summary>
        //public static bool operator ==(CapeOptionParameter thisParameter, ICapeOptionParameterSpec other)
        //{
        //    return EqualsHelper(thisParameter, other);
        //}

        ///// <summary>
        ///// Compare between opotion parameters
        ///// </summary>
        //public static bool operator !=(CapeOptionParameter thisParameter, ICapeOptionParameterSpec other)
        //{
        //    return !(thisParameter == other);
        //}

        ///// <summary>
        ///// Compare between option parameter and string
        ///// </summary>
        //public static bool operator ==(CapeOptionParameter thisParameter, string other)
        //{
        //    return thisParameter != (CapeOptionParameter)null &&
        //        thisParameter.CompareTo(other) == 0;
        //}

        ///// <summary>
        ///// Compare between option parameter and string
        ///// </summary>
        //public static bool operator !=(CapeOptionParameter thisParameter, string other)
        //{
        //    return !(thisParameter == other);
        //}

        ///// <summary>
        ///// Compare between option parameter and enum, only compare ToString() value, ignoring enum type
        ///// </summary>
        //public static bool operator ==(CapeOptionParameter thisParameter, Enum other)
        //{
        //    return thisParameter != (CapeOptionParameter)null &&
        //        thisParameter.Equals(other);
        //}

        ///// <summary>
        ///// Compare between option parameter and enum, only compare ToString() value, ignoring enum type
        ///// </summary>
        //public static bool operator !=(CapeOptionParameter thisParameter, Enum other)
        //{
        //    return !(thisParameter == other);
        //}

        ///// <summary>
        ///// Actual implement of ==, only compare value, ignoring OptionList and Enum type
        ///// </summary>
        //public override bool Equals(object obj)
        //{
        //    if (object.ReferenceEquals(this, obj))
        //        return true;
        //    else if (obj is string)
        //        return (string)obj == (string)this;
        //    else if (obj is Enum)
        //        return CompareTo(obj.ToString()) == 0;
        //    else
        //        return CompareTo(obj as ICapeOptionParameterSpec) == 0;
        //}

        ///// <summary>
        ///// use base.GetHashCode
        ///// </summary>
        //public override int GetHashCode()
        //{
        //    return base.GetHashCode();
        //}

        #endregion

        #region IPersist and Serialize
        /// <summary>
        /// Called on Serializing, save the enumType as string, don't call this method 
        /// </summary>
        [OnSerializing]
        private void ConvertEnumType2String(StreamingContext context)
        {
            assemblyName = _enumType.Assembly.FullName;
            enumTypeName = _enumType.FullName;
        }
        /// <summary>
        /// Called on DeSerializing, restore the enumType by string, don't call this method 
        /// </summary>
        [OnDeserialized]
        private void ConvertString2EnumType(StreamingContext context)
        {
            try
            {
                Assembly assembly = AppDomain.CurrentDomain.GetAssemblies()
                    .FirstOrDefault(x => x.FullName == assemblyName);
                _enumType = assembly.GetType(enumTypeName);
            }
            catch
            {
                CapeDiagnostic.LogMessage("Unable to load option parameter, possible reason is the path of assembly is changed.");
                //throw new ECapePersistenceNotFoundException(this.GetType().Name, "Unable to load option parameter, possible reason is the path of assembly is changed.", null);
            }
        }

        #endregion

        /// <summary>
        /// convert to Enum
        /// </summary>
        public static explicit operator Enum(CapeOptionParameter optionParameter)
        {
            return optionParameter != null
                ? optionParameter.EnumValue
                : default(Enum);
        }

        /// <summary>
        /// convert to string
        /// </summary>
        public static explicit operator string(CapeOptionParameter optionParameter)
        {
            return optionParameter != null
                ? optionParameter.StringValue
                : default(string);
        }

        /// <summary>
        /// Clone of option parameters
        /// </summary>
        public override object Clone()
        {
            if (_enumType == null)
                return new CapeOptionParameter(
                    this.ComponentName,
                    (IEnumerable<string>)this.OptionList,
                    this.Mode, this.DefaultValue, this.RestrictedToList)
                {
                    ComponentDescription = this.ComponentDescription,
                    value = this.value,
                    Dirty = true
                };
            else
                return new CapeOptionParameter(this.ComponentName, this._enumType, this.Mode, null, this.RestrictedToList)
                {
                    ComponentDescription = this.ComponentDescription,
                    value = this.value,
                    Dirty = true
                };
        }
       
    }
}
