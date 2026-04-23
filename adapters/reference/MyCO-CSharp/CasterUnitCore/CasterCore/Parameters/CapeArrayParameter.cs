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
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.InteropServices;
using CAPEOPEN;

namespace CasterCore
{
    /// <summary>
    /// A one dimensional matrix parameter, implement ICapeArrayParameterSpec.
    /// Can not work for any simulator, trying to make it work
    /// </summary>
    [Serializable]
    [ComVisible(true)]
    [Guid("8A15FAD5-422A-4274-B9E7-42122C9EEFDA")]
    [ComDefaultInterface(typeof(ICapeParameter))]
    public class CapeArrayParameter
        : CapeParameterBase, ICapeArrayParameterSpec,
        ICollection<ICapeParameter>
    {
        private List<ICapeParameter> _array;

        #region Constructor

        /// <summary>
        /// default constructor
        /// </summary>
        public CapeArrayParameter()
            : this("arrayParameter", CapeParamMode.CAPE_INPUT_OUTPUT)
        { }

        /// <summary>
        /// constructor of array parameter
        /// </summary>
        public CapeArrayParameter(string name, CapeParamMode mode)
            : base(name, CapeParamType.CAPE_ARRAY, mode, UnitCategoryEnum.Dimensionless)
        {
            _array = new List<ICapeParameter>();
            ValStatus = CapeValidationStatus.CAPE_NOT_VALIDATED;
            IsReadOnly = false;
        }
        #endregion

        #region CapeParameterBase

        /// <summary>
        /// Check for valid status for each item
        /// </summary>
        public override bool Validate(ref string message)
        {
            bool[] isAvailable;
            object messages = null;
            if (Type == CapeParamType.CAPE_ARRAY)
                isAvailable = (bool[])Validate(_array, ref messages);
            else
                return false;

            for (int i = 0; i < isAvailable.Rank; i++)
            {
                if (isAvailable[i] == false)
                {
                    ValStatus = CapeValidationStatus.CAPE_INVALID;
                    return false;
                }
            }
            ValStatus = CapeValidationStatus.CAPE_VALID;
            return true;
        }

        /// <summary>
        /// is all item valid
        /// </summary>
        public override bool Validate()
        {
            string message = null;
            return Validate(ref message);
        }

        /// <summary>
        /// Clear All
        /// </summary>
        public override void Reset()
        {
            _array.Clear();
            ValStatus = CapeValidationStatus.CAPE_NOT_VALIDATED;
            Dirty = true;
        }

        /// <summary>
        /// return value, doesn't work, dont know why
        /// </summary>
        public override object value
        {
            get { return ItemsSpecifications; }
            set
            {
                var v = value as IEnumerable<CapeParameterBase>;
                if (v == null)
                    return;
                _array.Clear();
                var temp = v as CapeParameterBase[] ?? v.ToArray();
                _array.Capacity = temp.Count();
                foreach (var element in temp)
                {
                    _array.Add(element);
                }
                ValStatus = CapeValidationStatus.CAPE_NOT_VALIDATED;
            }
        }

        #endregion

        #region ICapeArrayParameterSpec

        /// <summary>
        /// Validate a value
        /// </summary>
        public object Validate(object value, ref object message)
        {
            //这里的value应该是内部指定的存储类型
            var v = value as IEnumerable<CapeParameterBase>;
            //这里能正常转换吗？？？可以用单元测试试一下
            //这样写能不能获取到二维数组？？？？?????????????????
            if (v == null)
            {
                message = "value is not CapeArrayParameter";
                return false;
            }
            bool[] validList = new bool[v.Count()];
            message = new string[v.Count()];
            string[] messageRef = message as string[];
            for (int i = 0; i < v.Count(); i++)
            {
                validList[i] = v.ElementAtOrDefault(i).Validate(ref messageRef[i]);
            }
            return validList;
        }

        /// <summary>
        /// return numbers of dimensions, array only have 1 dimension
        /// </summary>
        public int NumDimensions { get { return 1; } }

        /// <summary>
        /// return numbers of elements
        /// </summary>
        public object Size
        {
            get
            {
                int[] sizeList = new int[1];
                sizeList[0] = _array.Count;
                return sizeList;
            }
        }

        /// <summary>
        /// return value as object array
        /// </summary>
        public object ItemsSpecifications
        {
            get
            {
                object[] spec = new object[1];
                spec[0] = _array.ToArray();
                return spec;
            }
        }

        #endregion

        #region Implementation of IEnumerable

        /// <summary>
        /// return its inner List enumerator
        /// </summary>
        /// <returns></returns>
        public IEnumerator<ICapeParameter> GetEnumerator()
        {
            return _array.GetEnumerator();
        }

        IEnumerator IEnumerable.GetEnumerator()
        {
            return GetEnumerator();
        }

        #endregion

        #region ICollection<ICapeParameter>

        /// <summary>
        /// Add an parameter
        /// </summary>
        /// <param name="item"></param>
        public void Add(ICapeParameter item)
        {
            if (IsReadOnly) return;
            _array.Add(item);
        }

        /// <summary>
        /// clear all
        /// </summary>
        public void Clear()
        {
            if (IsReadOnly) return;
            _array.Clear();
        }

        /// <summary>
        /// Is a parameter is in this array
        /// </summary>
        /// <param name="item"></param>
        /// <returns></returns>
        public bool Contains(ICapeParameter item)
        {
            return _array.Contains(item);
        }

        /// <summary>
        /// copy to an array
        /// </summary>
        /// <param name="array"></param>
        /// <param name="arrayIndex"></param>
        public void CopyTo(ICapeParameter[] array, int arrayIndex)
        {
            _array.CopyTo(array, arrayIndex);
        }

        /// <summary>
        /// Remove an item
        /// </summary>
        /// <param name="item"></param>
        /// <returns></returns>
        public bool Remove(ICapeParameter item)
        {
            if (IsReadOnly) return false;
            return _array.Remove(item);
        }

        /// <summary>
        /// Number of array item
        /// </summary>
        public int Count { get { return _array.Count; } }

        /// <summary>
        /// is the array changable, always false
        /// </summary>
        public bool IsReadOnly { get; protected set; }

        #endregion

        /// <summary>
        /// Clone of one dimensional matrix,
        /// not implemented.
        /// </summary>
        public override object Clone()
        {
            throw new NotImplementedException();
        }
    }
}

