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

#pragma warning disable CA1065

namespace CasterCore
{
    /// <summary>
    /// a function used to filter parameters, same as Func<ICapeIdentification,bool>;
    /// </summary>
    public delegate bool ValidateFunc(ICapeIdentification item);

    /// <summary>
    /// This class contains a collection of ICapeIdentification, with a key default set to its name, and can also use index to locate item
    /// </summary>
    [Serializable]
    [ComVisible(true)]
    [Guid("7D9B4745-1510-4BCD-B7CF-87DE6796AD23")]
    [ComDefaultInterface(typeof(ICapeCollection))]
    public class CapeCollection :
        CapeOpenBaseObject, ICapeCollection, ICollection<CapeCollectionPair>, ICloneable
    {
        /// <summary>
        /// contains items with a key
        /// </summary>
        protected SortedDictionary<string, ICapeIdentification> _items = new SortedDictionary<string, ICapeIdentification>();
        //the value is keeped both in keys and index, makes it possible to access through index or its name
        /// <summary>
        /// Validation for insert value
        /// </summary>
        protected ValidateFunc constraint = null;


        #region Constructor

        /// <summary>
        /// Constructor of CapeCollection
        /// </summary>
        /// <param name="name">the name of this collection</param>
        /// <param name="description">discription of this collection</param>
        /// <param name="constraint">a Func&lt;ICapeIdentification,bool&gt; delegate, used to validate items.</param>
        /// <param name="canRename">Is the collection can be renamed.</param>
        public CapeCollection(string name = null, string description = "",
            /*Func<ICapeIdentification, bool>*/
            ValidateFunc constraint = null, bool canRename = false)
            : base(name, description, canRename)
        {
            IsReadOnly = false;
            this.constraint = constraint;
        }

        #endregion

        #region ICapeCollection

        /// <summary>
        /// if id is int, it starts from one. According to CO
        /// if id is string, same as this[id]
        /// </summary>
        object ICapeCollection.Item(object id)
        {
            try
            {
                if (id is string)
                    return this[(string)id];
                else if (id is int)
                    return this[(int)id - 1];
                else
                    throw new ECapeUnknownException(this, "index can only be int or string");
            }
            catch (ArgumentOutOfRangeException e)
            {
                throw new ECapeUnknownException(this,
                    $"Index not avaliable. Count: {Count}, Wanted {id}",
                    e);
            }
        }

        /// <summary>
        /// return value by index, index starts from zero
        /// </summary>
        /// <paramCollection name="index">Start from zero</paramCollection>
        public ICapeIdentification this[int index]
        {
            get
            {
                try
                {
                    return this._items.ElementAt(index).Value;
                }
                catch (ArgumentOutOfRangeException e)
                {
                    throw new ECapeUnknownException(this,
                        $"Index not avaliable, start from zero. Count: {Count}, Wanted {index}",
                        e);
                }
            }
            set
            {
                try
                {
                    var pair = this._items.ElementAt(index);
                    this._items[pair.Key] = value;
                }
                catch (ArgumentOutOfRangeException e)
                {
                    throw new ECapeUnknownException(this,
                        $"Index not avaliable, start from zero.. Count: {Count}, Wanted {index}",
                        e);
                }
            }
        }

        /// <summary>
        /// return value by name
        /// </summary>
        public ICapeIdentification this[string key]
        {
            get
            {
                try
                {
                    return this._items[key];
                }
                catch (KeyNotFoundException e)
                {
                    throw new ECapeUnknownException(this,
                        $"Key not avaliable. Wanted: {key}",
                        e);
                }
            }
            set
            {
                if (!_items.ContainsKey(key))
                    Add(key, value);
                else
                    this._items[key] = value;
            }
        }

        /// <summary>
        /// return CapeCollectionPair by index, start from one
        /// </summary>
        public CapeCollectionPair GetItem(int index)
        {
            if (index < 0 || index >= _items.Count)
                throw new ECapeUnknownException(this,
                        $"Index not avaliable, start from zero. Count: {Count}, Wanted {index}",
                        null, "ICapeCollection");
            return new CapeCollectionPair(_items.ElementAt(index).Key, _items.ElementAt(index).Value);
        }

        int ICapeCollection.Count() => _items.Count;

        #endregion

        #region IEnumerable

        /// <summary>
        /// Enumerator of CapeCollection, ordered by insertion sequence
        /// </summary>
        public IEnumerator<CapeCollectionPair> GetEnumerator()
        {
            return _items.Select(pair => new CapeCollectionPair(pair.Key, pair.Value)).GetEnumerator();
        }

        IEnumerator IEnumerable.GetEnumerator()
        {
            return GetEnumerator();
        }

        #endregion

        #region ICollection<CapeCollectionPair>

        /// <summary>
        /// Add an item with key and value
        /// </summary>
        public void Add(CapeCollectionPair item) => Add(item.Key, item.Value);

        /// <summary>
        /// Add an item, the default key is ComponentName
        /// </summary>
        public void Add(CapeOpenBaseObject item)
        {
            string autokey = item.ComponentName;
            Add(autokey, item);
        }

        /// <summary>
        /// Add an item with a key
        /// </summary>
        public void Add(string key, ICapeIdentification item)
        {
            if (constraint != null && !constraint(item))
                throw new ECapeUnknownException(this,
                    $"<{key}> is not a valid item in collection <{ComponentName}>. Due to <{constraint.ToString()}>");
            if (_items.ContainsValue(item))
                throw new ECapeUnknownException(this,
                    $"This Collection already has the same item: <{item.ComponentName}>.");
            if (_items.ContainsKey(key))
                throw new ECapeUnknownException(this,
                    $"This collection already has an item with the same key: <{key}>.");
            _items.Add(key, item);
        }

        /// <summary>
        /// Wipe all items
        /// </summary>
        public void Clear()
        {
            _items.Clear();
        }

        /// <summary>
        /// contains an item with the same key
        /// </summary>
        public bool Contains(CapeCollectionPair item)
            => _items.ContainsKey(item.Key) && _items[item.Key] == item.Value;

        /// <summary>
        /// whether CapeCollection contains this item
        /// </summary>
        public bool Contains(ICapeIdentification item)
            => _items.ContainsValue(item);

        /// <summary>
        /// whether CapeCollection contains item with this key
        /// </summary>
        public bool Contains(string key)
            => _items.ContainsKey(key);

        /// <summary>
        /// Copy the dictionary to an array of CapeCollectionPair, ordered by insertion sequence 
        /// </summary>
        public void CopyTo(CapeCollectionPair[] array, int arrayIndex)
        {
            var sortedList = new CapeCollectionPair[Count];
            for (int i = 0; i < Count; i++)
            {
                var pair = _items.ElementAt(i);
                sortedList[i] = new CapeCollectionPair(pair.Key, pair.Value);
            }
            sortedList.CopyTo(array, arrayIndex);
        }

        /// <summary>
        /// Remove item with key and value
        /// </summary>
        /// <param name="item"></param>
        /// <returns></returns>
        public bool Remove(CapeCollectionPair item)
            => Contains(item) ?
                Remove(item.Key) :
                false;

        /// <summary>
        /// Remove item with this key
        /// </summary>
        public bool Remove(string key)
        {
            if (!Contains(key)) return false;
            _items.Remove(key);
            return true;
        }
        /// <summary>
        /// Remove the same item, should be reference equal
        /// </summary>
        public bool Remove(ICapeIdentification item)
        {
            string key = (from p in _items
                          where p.Value.Equals(item)
                          select p.Key).SingleOrDefault();
            if (key == null) return false;
            _items.Remove(key);
            return true;
        }

        /// <summary>
        /// Number of this collection
        /// </summary>
        public int Count => _items.Count;

        /// <summary>
        /// Is the collection immutable, always false
        /// </summary>
        public bool IsReadOnly { get; protected set; }

        /// <summary>
        /// return item keys, in inserting order
        /// </summary>
        public string[] Keys => _items.Select(p => p.Key).ToArray();

        /// <summary>
        /// Return item array, without key
        /// </summary>
        public ICapeIdentification[] Values
        {
            get
            {
                return _items.Select(p => p.Value).ToArray();
            }
        }

        #endregion

        #region ICloneable

        /// <summary>
        /// if the item inside the CapeCollection is not CapeOpenBaseObject or never implement Clone(), the method will fail
        /// </summary>
        public object Clone()
        {
            CapeCollection newCollection = new CapeCollection(this.ComponentName,
                this.ComponentDescription, this.constraint, this.CanRename);
            foreach (var pair in _items)
            {
                newCollection.Add(pair.Key, pair.Value);
            }
            return newCollection;
        }

        #endregion
    }
}

#pragma warning restore