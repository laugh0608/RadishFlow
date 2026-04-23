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

namespace CasterCore
{
    /// <summary>
    /// Similar to KeyValuePair of Dictionary, the class is used to return pairs in CapeCollection, index is useless if not in collection
    /// </summary>
    [Serializable]
    public struct CapeCollectionPair
    {
        public string Key;
        public ICapeIdentification Value;

        public CapeCollectionPair(string key, ICapeIdentification value)
        {
            if (key == null)
                throw new ECapeUnknownException(
                    "key in CapeCollectionPair cannot be null.");
            Key = key;
            Value = value;
        }

    }
}
