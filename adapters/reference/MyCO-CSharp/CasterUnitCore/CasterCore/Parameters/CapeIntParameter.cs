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
    /// Int parameter, don't have an unit, can compare to int
    /// </summary>
    [Serializable]
    [ComVisible(true)]
    [Guid("2C57DC9F-1368-42eb-888F-5BC6ED7DDFA7")]
    [ComDefaultInterface(typeof(ICapeParameter))]
    public class CapeIntParameter :
        CapeParameterBase,
        ICapeIntegerParameterSpec, IComparable<int>,
        IComparable<ICapeIntegerParameterSpec>
    {
        #region Constructor
        private int _intVal;

        /// <summary>
        /// dafault name is "intParameter", and mode is CAPE_INPUT_OUTPUT
        /// </summary>
        public CapeIntParameter()
            : this("intParameter", CapeParamMode.CAPE_INPUT_OUTPUT)
        { }

        /// <summary>
        /// constructor of int parameter
        /// </summary>
        public CapeIntParameter(string name,
            CapeParamMode mode, int minVal = int.MinValue,
            int maxVal = int.MaxValue, int defaultVal = default(int))
            : base(name, CapeParamType.CAPE_INT, mode, UnitCategoryEnum.Dimensionless)
        {
            LowerBound = minVal;
            UpperBound = maxVal;
            DefaultValue = defaultVal;
            IntValue = DefaultValue;
        }

        #endregion

        #region CapeParameterBase

        /// <summary>
        /// Validate the parameter and reture the result
        /// </summary>
        /// <returns></returns>
        public override bool Validate()
        {
            string message = "";
            return Validate(ref message);
        }

        /// <summary>
        /// Validate the parameter and reture the result and error message
        /// </summary>
        public override bool Validate(ref string message)
        {
            bool isAvailable;
            isAvailable = Validate(IntValue, ref message);

            if (isAvailable)
                ValStatus = CapeValidationStatus.CAPE_VALID;
            else
                ValStatus = CapeValidationStatus.CAPE_INVALID;
            return isAvailable;
        }

        /// <summary>
        /// set to default value
        /// </summary>
        public override void Reset()
        {
            IntValue = DefaultValue;
        }

        /// <summary>
        /// get or set, can be set with int, string, ICapeIntegerParameterSpec
        /// </summary>
        public override object value
        {
            get { return IntValue; }
            set
            {
                int v;
                try
                {
                    if (value is ICapeIntegerParameterSpec)
                    {
                        v = (int)((ICapeParameter)value).value;
                    }
                    else
                    {
                        v = Convert.ToInt32(value);
                    }
                }
                catch (Exception e)
                {
                    throw new ECapeUnknownException(this,
                        $"value {value} is not int or string or ICapeIntegerParameterSpec",
                        e, "ICapeParameter");
                }

                IntValue = v;
            }
        }
        
        /// <summary>
        /// value with type
        /// </summary>
        public int IntValue
        {
            get => _intVal;
            set
            {
                if (_intVal == value)
                    return;
                _intVal = value;
                Dirty = true;
                ValStatus = CapeValidationStatus.CAPE_NOT_VALIDATED;
            }
        }

        #endregion

        #region ICapeIntegerParameterSpec

        /// <summary>
        /// check if the value to be set if avaliable
        /// </summary>
        public bool Validate(int value, ref string message)
        {
            bool isAvailable;
            if (value > UpperBound ||
                value < LowerBound)
            {
                message = "value is out of range";
                isAvailable = false;
            }
            else
            {
                message = "";
                isAvailable = true;
            }
            return isAvailable;
        }

        /// <summary>
        /// default value
        /// </summary>
        public int DefaultValue { get; set; }

        /// <summary>
        /// lower boundary
        /// </summary>
        public int LowerBound { get; set; }

        /// <summary>
        /// upper boundary
        /// </summary>
        public int UpperBound { get; set; }

        #endregion

        #region IComparable

        /// <summary>
        /// Compare the int parameter to a int
        /// </summary>
        public int CompareTo(int other)
        {
            return IntValue.CompareTo(other);
        }

        /// <summary>
        /// Compare between int parameters, return 1 if other is null
        /// </summary>
        public int CompareTo(ICapeIntegerParameterSpec other)
        {
            return CompareTo((object)other);
        }

        /// <summary>
        /// Compare between int parameters or string, return 1 if other is null
        /// </summary>
        public int CompareTo(object other)
        {
            switch (other)
            {
                case null:
                    return 1;
                case ICapeIntegerParameterSpec boolParam:
                    return CompareTo((int)((ICapeParameter)other).value);
                default:
                    int v;
                    try
                    {
                        v = Convert.ToInt32(other);
                    }
                    catch (Exception e)
                    {
                        throw new ECapeUnknownException(this,
                            $"value {other} can't convert to int",
                            e);
                    }
                    return CompareTo(v);
            }
        }

        ///// <summary>
        ///// Compare between int parameters
        ///// </summary>
        //public static bool operator ==(CapeIntParameter thisParameter, ICapeIntegerParameterSpec other)
        //{
        //    return EqualsHelper(thisParameter, other);
        //}

        ///// <summary>
        ///// Compare between int parameters
        ///// </summary>
        //public static bool operator !=(CapeIntParameter thisParameter, ICapeIntegerParameterSpec other)
        //{
        //    return !(thisParameter == other);
        //}

        ///// <summary>
        ///// Compare between int parameter and int
        ///// </summary>
        //public static bool operator ==(CapeIntParameter thisParameter, int other)
        //{
        //    return thisParameter != null && 
        //        thisParameter.CompareTo(other) == 0;
        //}

        ///// <summary>
        ///// Compare between int parameter and int
        ///// </summary>
        //public static bool operator !=(CapeIntParameter thisParameter, int other)
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
        //    else if (obj is int)
        //    {
        //        return CompareTo((int)obj) == 0;
        //    }
        //    else
        //    {
        //        return
        //            CompareTo(obj as ICapeIntegerParameterSpec) == 0;
        //    }
        //}

        ///// <summary>
        ///// use base.GetHashCode
        ///// </summary>
        //public override int GetHashCode()
        //{
        //    return base.GetHashCode();
        //}

        #endregion

        /// <summary>
        /// can be convert to int, if parameter is null, return 0
        /// </summary>
        public static explicit operator int(CapeIntParameter intParameter)
        {
            return intParameter != null
                ? intParameter.IntValue
                : default(int);
        }
        //explicit this method to clarify the usage, it should be used as a class, not a double

        /// <summary>
        /// Clone of int parameter
        /// </summary>
        public override object Clone()
        {
            return new CapeIntParameter(ComponentName, Mode,
                LowerBound, UpperBound, DefaultValue)
            {
                ComponentDescription = this.ComponentDescription,
                value = this.value,
                Dirty = true,
            };
        }
        
    }
}
