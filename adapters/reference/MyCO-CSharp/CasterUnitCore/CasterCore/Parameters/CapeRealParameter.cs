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
    /// Real parameter, have a unit, can compare to double
    /// </summary>
    [Serializable]
    [ComVisible(true)]
    [Guid("77E39C43-046B-4b1f-9EE0-AA9EFC55D2EE")]
    [ComDefaultInterface(typeof(ICapeParameter))]
    public class CapeRealParameter :
        CapeParameterBase,
        ICapeRealParameterSpec, IComparable<double>,
        IComparable<ICapeRealParameterSpec>
    {
        private double _dblVal;        //Current value

        /// <summary>
        /// represent unit category, like temperature
        /// </summary>
        public UnitCategoryEnum CurrentUnitCategory { get; set; }

        /// <summary>
        /// In default GUI, when unit is changed and mode is CAPE_INPUT, the value of parameter will change
        /// if you dont want the actual value changes with unit, use CAPE_INPUT_OUTPUT to avoid actual value change
        /// </summary>
        public string CurrentUnit { get; set; }

        #region Constructor

        /// <summary>
        /// default name is "realParameter"
        /// </summary>
        public CapeRealParameter()
            : this("realParameter", UnitCategoryEnum.Dimensionless,
                  CapeParamMode.CAPE_INPUT_OUTPUT)
        { }

        /// <summary>
        /// real parameter
        /// </summary>
        /// <paramCollection name="unitCategory">unit type</paramCollection>
        /// <paramCollection name="mode">if set to Input, the value won't change with unit in default window</paramCollection>
        public CapeRealParameter(string name, UnitCategoryEnum unitCategory,
            CapeParamMode mode, double minVal = double.MinValue,
            double maxVal = double.MaxValue, double defaultVal = double.NaN)
            : base(name, CapeParamType.CAPE_REAL, mode, unitCategory)
        {
            this.LowerBound = minVal;
            this.UpperBound = maxVal;
            this.DefaultValue = defaultVal;
            DoubleValue = DefaultValue;
            CurrentUnitCategory = unitCategory;
            CurrentUnit = Units.GetSIUnit(unitCategory);
        }

        #endregion

        #region ICapeParameter

        /// <summary>
        /// display value, can be set to double, string, ICapeRealParameterSpec
        /// Using the property will call boxing procedure, performance loss
        /// </summary>
        public override object value
        {
            get => DoubleValue;
            set
            {
                double v;
                try
                {
                    if (value is ICapeRealParameterSpec)
                    {
                        v = (double)((ICapeParameter)value).value;
                    }
                    else
                    {
                        v = Convert.ToDouble(value); //Should throw a exception
                    }
                }
                catch (Exception)
                {
                    throw new ECapeUnknownException(this,
                        $"value {value} is not double or string or ICapeRealParameterSpec");
                }

                DoubleValue = v;
            }
        }

        /// <summary>
        /// value with type, user-input value, with user-defined unit
        /// </summary>
        public double DoubleValue
        {
            get => _dblVal;
            set
            {
                if (_dblVal == value)
                    return;
                _dblVal = value;
                Dirty = true;
                ValStatus = CapeValidationStatus.CAPE_NOT_VALIDATED;
            }
        }

        /// <summary>
        /// Convert current value and unit to SI value
        /// </summary>
        public double SIValue
        {
            get { return Units.ConvertToSI(DoubleValue, CurrentUnit, CurrentUnitCategory); }
            set { this.DoubleValue = Units.ConvertFromSI(CurrentUnit, value, CurrentUnitCategory); }
        }

        /// <summary>
        /// Set to its default value
        /// </summary>
        public override void Reset()
        {
            DoubleValue = DefaultValue;
        }

        /// <summary>
        /// whether the value is a number and inside range
        /// </summary>
        public override bool Validate()
        {
            string message = "";
            return Validate(ref message);
        }

        /// <summary>
        /// return the result and an error message
        /// </summary>
        public override bool Validate(ref string message)
        {
            bool isAvailable;
            isAvailable = Validate(this.DoubleValue, ref message);

            if (isAvailable)
                ValStatus = CapeValidationStatus.CAPE_VALID;
            else
                ValStatus = CapeValidationStatus.CAPE_INVALID;
            return isAvailable;
        }

        #endregion

        #region ICapeRealParameterSpec

        /// <summary>
        /// default value
        /// </summary>
        public double DefaultValue { get; set; }

        /// <summary>
        /// lower boundary
        /// </summary>
        public double LowerBound { get; set; }

        /// <summary>
        /// upper boundary
        /// </summary>
        public double UpperBound { get; set; }

        /// <summary>
        /// whether the value is a number and inside range
        /// </summary>
        public bool Validate(double value, ref string message)
        {
            bool isAvailable = false;
            if (double.IsNaN(value))
            {
                message = "value is NaN";
                isAvailable = false;
            }
            else if (double.IsInfinity(value))
            {
                message = "value is Infinity";
                isAvailable = false;
            }
            else if (!double.IsNaN(UpperBound) && value > this.UpperBound ||
                !double.IsNaN(LowerBound) && value < this.LowerBound)
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

        #endregion

        #region IComparable

        /// <summary>
        /// Compare the double parameter to a double
        /// </summary>
        public int CompareTo(double other)
        {
            return DoubleValue.CompareTo(other);
        }

        /// <summary>
        /// Compare between double parameters, return 1 if other is null
        /// </summary>
        public int CompareTo(ICapeRealParameterSpec other)
        {
            return CompareTo((object)other);
        }

        /// <summary>
        /// Compare with any object can be convert to double, return 1 if other is null
        /// </summary>
        public int CompareTo(object other)
        {
            switch (other)
            {
                case null:
                    return 1;
                case ICapeRealParameterSpec boolParam:
                    return CompareTo((double)((ICapeParameter)other).value);
                default:
                    double v;
                    try
                    {
                        v = Convert.ToDouble(other);
                    }
                    catch (Exception e)
                    {
                        throw new ECapeUnknownException(this,
                            $"value {other} can't convert to double",
                            e);
                    }
                    return CompareTo(v);
            }
        }

        ///// <summary>
        ///// Compare between double parameters
        ///// </summary>
        //public static bool operator ==(CapeRealParameter thisParameter, ICapeRealParameterSpec other)
        //{
        //    return EqualsHelper(thisParameter, other);
        //}

        ///// <summary>
        ///// Compare between double parameters
        ///// </summary>
        //public static bool operator !=(CapeRealParameter thisParameter, ICapeRealParameterSpec other)
        //{
        //    return !(thisParameter == other);
        //}

        ///// <summary>
        ///// Compare between int parameter and int
        ///// </summary>
        //public static bool operator ==(CapeRealParameter thisParameter, int other)
        //{
        //    return thisParameter != null &&
        //        thisParameter.CompareTo(other) == 0;
        //}

        ///// <summary>
        ///// Compare between int parameter and int
        ///// </summary>
        //public static bool operator !=(CapeRealParameter thisParameter, int other)
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
        //            CompareTo(obj as ICapeRealParameterSpec) == 0;
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
        /// can convert to double, if parameter is null, return double.NaN
        /// </summary>
        public static explicit operator double(CapeRealParameter realParameter)
        {
            return realParameter != null
                ? (double)(realParameter.value)
                : double.NaN;
        }
        //explicit this method to clarify the usage, it should be used as a class, not a double

        /// <summary>
        /// Clone of real parameter, include its unit
        /// </summary>
        public override object Clone()
        {
            return new CapeRealParameter(this.ComponentName, this.CurrentUnitCategory, this.Mode,
                this.LowerBound, this.UpperBound, this.DefaultValue)
            {
                ComponentDescription = this.ComponentDescription,
                CurrentUnit = this.CurrentUnit,
                value = this.value,
                Dirty = true,
            };
        }

        /// <summary>
        /// return "{ComponentName}: {value} {CurrentUnit}"
        /// </summary>
        public override string ToString()
        {
            return $"{ComponentName}: {value} {CurrentUnit}";
        }
    }
}
