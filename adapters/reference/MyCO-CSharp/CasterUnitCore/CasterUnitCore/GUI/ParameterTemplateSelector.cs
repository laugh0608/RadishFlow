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

using System.Windows;
using System.Windows.Controls;
using CAPEOPEN;

namespace CasterUnitCore
{
    class ParameterTemplateSelector : DataTemplateSelector
    {
        public DataTemplate RealTemplate { get; set; }
        public DataTemplate OptionTemplate { get; set; }
        public DataTemplate IntTemplate { get; set; }

        public override DataTemplate SelectTemplate(object item, DependencyObject container)
        {
            ParameterModel pm=item as ParameterModel;

            if (pm.ParamType == CapeParamType.CAPE_REAL)
                return RealTemplate;
            if (pm.ParamType == CapeParamType.CAPE_INT)
                return IntTemplate;
            if (pm.ParamType ==CapeParamType.CAPE_BOOLEAN
                     || pm.ParamType ==CapeParamType.CAPE_OPTION)
                return OptionTemplate;
            else
                return base.SelectTemplate(item, container);
        }

    }
}
