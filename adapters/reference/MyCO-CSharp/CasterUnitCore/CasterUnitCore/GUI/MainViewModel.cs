using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using CasterCore;

namespace CasterUnitCore
{
    public class MainViewModel
    {
        public ObservableCollection<ParameterModel> Parameters { get; set; }
        public ObservableCollection<ParameterModel> Results { get; set; }

        public void SetBinding(CapeCollection parameters, CapeCollection results)
        {
            this.Parameters = new ObservableCollection<ParameterModel>();
            this.Results = new ObservableCollection<ParameterModel>();
            foreach (var parameter in parameters)
            {
                this.Parameters.Add(new ParameterModel(parameter.Value as CapeParameterBase));
            }
            foreach (var result in results)
            {
                this.Results.Add(new ParameterModel(result.Value as CapeParameterBase));
            }
        }
    }
}
