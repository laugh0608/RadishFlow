using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Text;
using System.Threading.Tasks;
using CasterUnitCore.Annotations;
using CAPEOPEN;
using CasterCore;

namespace CasterUnitCore
{
    public class ParameterModel : INotifyPropertyChanged
    {
        public CapeParameterBase Parameter { get; set; }
        public CapeParamType ParamType { get { return Parameter.Type; } }

        public ParameterModel(CapeParameterBase param)
        {
            Parameter = param;
        }


        #region GUI-relate

        /// <summary>
        /// Name shown in GUI
        /// </summary>
        public string Name { get { return Parameter.ComponentName; } }

        /// <summary>
        /// value shown in GUI, will convert to CurrentUnit
        /// </summary>
        public string CurrentValue
        {
            get
            {
                if (ParamType == CapeParamType.CAPE_REAL)
                {
                    CapeRealParameter realParam = Parameter as CapeRealParameter;
                    return Units.UnitConvert(CurrentUnit, realParam.DoubleValue,
                        Units.GetSIUnit(realParam.CurrentUnitCategory),
                        realParam.CurrentUnitCategory).ToString();
                }
                else
                {
                    return Parameter.value.ToString();
                }
            }
            set
            {
                if (ParamType == CapeParamType.CAPE_REAL)
                {
                    try
                    {
                        CapeRealParameter realParam = Parameter as CapeRealParameter;
                        realParam.value = Units.UnitConvert(
                            Units.GetSIUnit(realParam.CurrentUnitCategory),
                            Convert.ToDouble(value), CurrentUnit, realParam.CurrentUnitCategory);
                    }
                    catch (Exception)
                    {
                        CapeRealParameter realParam = Parameter as CapeRealParameter;
                        realParam.value = double.NaN;
                    }
                }
                else if (ParamType == CapeParamType.CAPE_INT)
                {
                    try
                    {
                        CapeIntParameter intParam = Parameter as CapeIntParameter;
                        intParam.value = Convert.ToInt32(value);
                    }
                    catch (Exception)
                    {
                        CapeIntParameter intParam = Parameter as CapeIntParameter;
                        intParam.value = default(int);
                    }

                }
                else
                    Parameter.value = value;
            }
        }

        /// <summary>
        /// Unit shown in GUI, when unit is changed and mode is CAPE_INPUT, the value of parameter will change
        /// if you dont want the actual value changes with unit, use CAPE_INPUT_OUTPUT to avoid actual value change
        /// </summary>
        public string CurrentUnit
        {
            get { return ((CapeRealParameter)Parameter).CurrentUnit; }
            set
            {
                CapeRealParameter realParam = Parameter as CapeRealParameter;
                ((CapeRealParameter)Parameter).CurrentUnit = value;
                if (realParam.Mode == CapeParamMode.CAPE_INPUT)
                    realParam.value = Units.UnitConvert(Units.GetSIUnit(realParam.CurrentUnitCategory),
                                        Convert.ToDouble(realParam.value), CurrentUnit, realParam.CurrentUnitCategory);
                else     //if parameter is input, the display value won't change, otherwise, the display value change
                    OnPropertyChanged(nameof(CurrentValue));
            }
        }            //当前单位，默认SI
        /// <summary>
        /// unit list, get from Units class
        /// </summary>
        public IEnumerable<string> CurrentUnitsList
        {
            get
            {
                CapeRealParameter realParam = Parameter as CapeRealParameter;
                IEnumerable<string> a = Units.GetUnitList(realParam.CurrentUnitCategory);
                return Units.GetUnitList(realParam.CurrentUnitCategory);
            }
        }

        /// <summary>
        /// Options shown in GUI
        /// </summary>
        public IEnumerable<string> OptionList
        {
            get
            {
                if (ParamType == CapeParamType.CAPE_BOOLEAN)
                    return new[] { "True", "False" };
                else if (ParamType == CapeParamType.CAPE_OPTION)
                    return ((CapeOptionParameter)Parameter).OptionList as string[];
                else
                    return null;
            }
        }

        #endregion

        public event PropertyChangedEventHandler PropertyChanged;

        [NotifyPropertyChangedInvocator]
        protected virtual void OnPropertyChanged([CallerMemberName] string propertyName = null)
        {
            var handler = PropertyChanged;
            if (handler != null) handler(this, new PropertyChangedEventArgs(propertyName));
        }
    }
}
