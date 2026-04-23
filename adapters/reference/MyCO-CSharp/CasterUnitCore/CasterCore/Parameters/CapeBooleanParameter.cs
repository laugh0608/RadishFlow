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
    /// bool parameter, implemented ICapeBooleanParameterSpec, can compare to bool, can be implicit convert to bool
    /// </summary>
    [Serializable]
    [ComVisible(true)]
    [Guid("8B8BC504-EEB5-4a13-B016-9614543E4536")]
    [ComDefaultInterface(typeof(ICapeParameter))]
    public class CapeBooleanParameter
        : CapeParameterBase, ICapeBooleanParameterSpec,
          IComparable<bool>, IComparable<ICapeBooleanParameterSpec>
    {
        private bool _boolVal;

        #region Constructor

        /// <summary>
        /// default value is false, default name is "booleanParameter"
        /// </summary>
        public CapeBooleanParameter()
            : this("booleanParameter", CapeParamMode.CAPE_INPUT_OUTPUT, false)
        { }

        /// <summary>
        /// constructor of bool parameter
        /// </summary>
        public CapeBooleanParameter(string name, CapeParamMode mode, bool defVal = false)
            : base(name, CapeParamType.CAPE_BOOLEAN, mode, UnitCategoryEnum.Dimensionless)
        {
            DefaultValue = defVal;
            BoolValue = DefaultValue;
        }

        #endregion

        #region CapeParameterBase

        /// <summary>
        /// bool paramCollection is always valid
        /// </summary>
        /// <returns></returns>
        public override bool Validate()
        {
            string message = "";
            return Validate(ref message);
        }

        /// <summary>
        /// bool paramCollection is always valid
        /// </summary>
        public override bool Validate(ref string message)
        {
            ValStatus = CapeValidationStatus.CAPE_VALID;
            message = "";
            return true;
        }

        /// <summary>
        /// reset to the default value, and status set to CAPE_NOT_VALIDATED
        /// </summary>
        public override void Reset()
        {
            BoolValue = DefaultValue;
        }

        /// <summary>
        /// return bool value, can be set with string or bool
        /// </summary>
        public override object value
        {
            get { return BoolValue; }
            set
            {
                bool v;
                try
                {
                    v = Convert.ToBoolean(value);
                }
                catch (Exception e)
                {
                    throw new ECapeUnknownException(this,
                        $"value {value} can not be convert to bool.",
                        e, "ICapeParameter");
                }

                BoolValue = v;
            }
        }

        /// <summary>
        /// value with type
        /// </summary>
        public bool BoolValue
        {
            get => _boolVal; set
            {
                if (_boolVal == value)
                    return;
                _boolVal = value;
                Dirty = true;
                ValStatus = CapeValidationStatus.CAPE_NOT_VALIDATED;
            }
        }

        #endregion

        #region ICapeBooleanParameterSpec

        /// <summary>
        /// validate if the value to be set if avaliable
        /// </summary>
        public bool Validate(bool validateValue, ref string message)
        {
            message = "";
            return true;
        }

        /// <summary>
        /// Default value
        /// </summary>
        public bool DefaultValue { get; set; }

        #endregion

        #region IComparable

        /// <summary>
        /// Compare the bool parameter to a bool
        /// </summary>
        public int CompareTo(bool other)
        {
            return BoolValue.CompareTo(other);
        }

        /// <summary>
        /// Compare between bool parameters, return 1 if other is null
        /// </summary>
        public int CompareTo(ICapeBooleanParameterSpec other)
        {
            return CompareTo((object)other);
        }

        /// <summary>
        /// Compare between bool parameters or string, return 1 if other is null
        /// </summary>
        public int CompareTo(object other)
        {
            switch (other)
            {
                case null:
                    return 1;
                case ICapeBooleanParameterSpec boolParam:
                    return CompareTo((bool)((ICapeParameter)other).value);
                default:
                    bool v;
                    try
                    {
                        v = Convert.ToBoolean(other);
                    }
                    catch (Exception e)
                    {
                        throw new ECapeUnknownException(this,
                            $"value {other} can't convert to bool",
                            e);
                    }
                    return CompareTo(v);
            }
        }

        ///// <summary>
        ///// Compare between bool parameters
        ///// </summary>
        //public static bool operator ==(CapeBooleanParameter thisParameter, object other)
        //{
        //    return EqualsHelper(thisParameter, other);
        //}

        ///// <summary>
        ///// Compare between bool parameters
        ///// </summary>
        //public static bool operator !=(CapeBooleanParameter thisParameter, object other)
        //{
        //    return !(thisParameter == other);
        //}

        ///// <summary>
        ///// Compare between bool parameter and bool
        ///// </summary>
        //public static bool operator ==(CapeBooleanParameter thisParameter, bool other)
        //{
        //    return !thisParameter.Equals(null) &&
        //        thisParameter.CompareTo(other) == 0;
        //}

        ///// <summary>
        ///// Compare between bool parameter and bool
        ///// </summary>
        //public static bool operator !=(CapeBooleanParameter thisParameter, bool other)
        //{
        //    return !(thisParameter == other);
        //}

        ///// <summary>
        ///// Actual implement of ==, boxing procedure might have performance loss
        ///// </summary>
        //public override bool Equals(object obj)
        //{
        //    if (object.ReferenceEquals(this, obj))
        //        return true;
        //    else if (obj is bool)
        //    {
        //        return CompareTo((bool)obj) == 0;
        //    }
        //    else
        //    {
        //        return
        //            CompareTo(obj as ICapeBooleanParameterSpec) == 0;
        //    }
        //}

        ///// <summary>
        ///// use base.GetHashCode,
        ///// two bool parameters may be equal, but will return different hashcode.
        ///// </summary>
        //public override int GetHashCode()
        //{
        //    return base.GetHashCode();
        //}

        #endregion

        /// <summary>
        /// can be used as bool, if parameter is null, return false
        /// </summary>
        public static explicit operator bool(CapeBooleanParameter boolParameter)
        {
            return boolParameter != null
                ? boolParameter.BoolValue
                : default(bool);
        }
        //explicit this method to clarify the usage, it should be used as a class, not a double

        /// <summary>
        /// Clone of bool parameter
        /// </summary>
        public override object Clone()
        {
            return new CapeBooleanParameter(ComponentName, Mode, DefaultValue)
            {
                ComponentDescription = this.ComponentDescription,
                BoolValue = this.BoolValue,
                Dirty = true
            };
        }
        
    }
}
