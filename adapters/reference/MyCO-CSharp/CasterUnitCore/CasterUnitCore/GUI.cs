using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using CasterUnitBase;
using CasterUnitBase.UI;

namespace CasterUnitBase
{
    public class GUI
    {
        private Calculator calculator;

        public GUI(Calculator calculator)
        {
            this.calculator = calculator;
        }

        public void ShowGUI()
        {
            MainWindow mainWindow = new MainWindow();
            mainWindow.ShowDialog();
        }
    }
}
