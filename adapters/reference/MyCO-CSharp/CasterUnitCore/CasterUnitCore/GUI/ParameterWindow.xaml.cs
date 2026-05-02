using System;
using System.Linq;
using System.Reflection;
using System.Windows;
using System.Windows.Controls;
using CasterCore;

namespace CasterUnitCore
{
    /// <summary>
    /// An auto Parameters window, made by wpf
    /// </summary>
    public partial class ParameterWindow : Window
    {
        ///// <summary>
        ///// Parameters from calculator, should be a copy
        ///// </summary>
        //public CapeCollection Parameters;
        ///// <summary>
        ///// Results from calculator, should be a copy
        ///// </summary>
        //public CapeCollection Results;
        public MainViewModel ViewModel;
        /// <summary>
        /// 
        /// </summary>
        /// <paramCollection name="unit">reference to unit, just to get its name for now</paramCollection>
        /// <paramCollection name="Parameters">Parameters, better be a clone</paramCollection>
        /// <paramCollection name="results">results</paramCollection>
        public ParameterWindow(CasterUnitOperationBase unit, CapeCollection parameters, CapeCollection results)
        {
            InitializeComponent();
            treeViewItem.Header = unit.ComponentName;
            this.ViewModel=new MainViewModel();
            this.ViewModel.SetBinding(parameters,results);
            this.Paramlst.ItemsSource = ViewModel.Parameters.ToArray();
            this.Resultlst.ItemsSource = ViewModel.Results;
            treeViewItem.Focus();
        }

        private void ButtonOK(object sender, RoutedEventArgs e)
        {
            DialogResult = true;
            Close();
        }

        private void ButtonCancel(object sender, RoutedEventArgs e)
        {
            DialogResult = false;
            Close();
        }

        private void TreeView_OnSelectedItemChanged(object sender, RoutedPropertyChangedEventArgs<object> e)
        {
            string select = "";
            object item = ((TreeView)sender).SelectedItem;
            if (item is TreeViewItem
                || item is StackPanel && item == paramTreeItem)
                select = "paramTreeItem";
            else if (item is StackPanel && item == resultTreeItem)
                select = "resultTreeItem";

            if (select == "paramTreeItem")
            {
                Paramlst.Visibility = Visibility.Visible;
                Resultlst.Visibility = Visibility.Collapsed;
            }
            else if (select == "resultTreeItem")
            {
                Paramlst.Visibility = Visibility.Collapsed;
                Resultlst.Visibility = Visibility.Visible;
            }
            else
            {
                Paramlst.Visibility = Visibility.Collapsed;
                Resultlst.Visibility = Visibility.Collapsed;
            }
        }

    }
}
