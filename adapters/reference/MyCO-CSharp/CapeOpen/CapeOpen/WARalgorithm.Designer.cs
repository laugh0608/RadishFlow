namespace CapeOpen
{
    partial class WARalgorithm
    {
        /// <summary>
        /// Required designer variable.
        /// </summary>
        private System.ComponentModel.IContainer components = null;

        /// <summary>
        /// Clean up any resources being used.
        /// </summary>
        /// <param name="disposing">true if managed resources should be disposed; otherwise, false.</param>
        protected override void Dispose(bool disposing)
        {
            foreach (object obj in p_Streams)
            {
                if (obj.GetType().IsCOMObject)
                {
                    System.Runtime.InteropServices.Marshal.FinalReleaseComObject(obj);
                }
            }
            foreach (object obj in p_InletStreams)
            {
                if (obj.GetType().IsCOMObject)
                {
                    System.Runtime.InteropServices.Marshal.FinalReleaseComObject(obj);
                }
            }
            foreach (object obj in p_OutletStreams)
            {
                if (obj.GetType().IsCOMObject)
                {
                    System.Runtime.InteropServices.Marshal.FinalReleaseComObject(obj);
                }
            }
            foreach (object obj in p_Units)
            {
                if (obj.GetType().IsCOMObject)
                {
                    System.Runtime.InteropServices.Marshal.FinalReleaseComObject(obj);
                }
            }
            //if (p_Monitoring.GetType().IsCOMObject)
            //{
            //    System.Runtime.InteropServices.Marshal.FinalReleaseComObject(p_Monitoring);
            //}
            if (disposing && (components != null))
            {
                components.Dispose();
            }
            base.Dispose(disposing);
        }

        #region Windows Form Designer generated code

        /// <summary>
        /// Required method for Designer support - do not modify
        /// the contents of this method with the code editor.
        /// </summary>
        private void InitializeComponent()
        {
            this.components = new System.ComponentModel.Container();
            System.ComponentModel.ComponentResourceManager resources = new System.ComponentModel.ComponentResourceManager(typeof(WARalgorithm));
            this.toolTip1 = new System.Windows.Forms.ToolTip(this.components);
            this.menuStrip1 = new System.Windows.Forms.MenuStrip();
            this.toolStripMenuItem1 = new System.Windows.Forms.ToolStripMenuItem();
            this.fileToolStripMenuItem = new System.Windows.Forms.ToolStripMenuItem();
            this.saveToolStripMenuItem = new System.Windows.Forms.ToolStripMenuItem();
            this.printToolStripMenuItem = new System.Windows.Forms.ToolStripMenuItem();
            this.toolStripSeparator1 = new System.Windows.Forms.ToolStripSeparator();
            this.exitToolStripMenuItem = new System.Windows.Forms.ToolStripMenuItem();
            this.tabControl1 = new System.Windows.Forms.TabControl();
            this.unitsAndStreamsTabPage = new System.Windows.Forms.TabPage();
            this.label5 = new System.Windows.Forms.Label();
            this.includeProductStreamsCheckBox = new System.Windows.Forms.CheckBox();
            this.energyTypeComboBox = new System.Windows.Forms.ComboBox();
            this.label4 = new System.Windows.Forms.Label();
            this.considerEnergyFlowsCheckBox = new System.Windows.Forms.CheckBox();
            this.checkedListBox1 = new System.Windows.Forms.CheckedListBox();
            this.label3 = new System.Windows.Forms.Label();
            this.feedAndInletStreamListBox = new System.Windows.Forms.ListBox();
            this.label2 = new System.Windows.Forms.Label();
            this.label1 = new System.Windows.Forms.Label();
            this.AvailableUnitOpsheckedListBox = new System.Windows.Forms.CheckedListBox();
            this.impactWeightsTabPage = new System.Windows.Forms.TabPage();
            this.trackBar8 = new System.Windows.Forms.TrackBar();
            this.numericUpDown8 = new System.Windows.Forms.NumericUpDown();
            this.label11 = new System.Windows.Forms.Label();
            this.trackBar7 = new System.Windows.Forms.TrackBar();
            this.numericUpDown7 = new System.Windows.Forms.NumericUpDown();
            this.label10 = new System.Windows.Forms.Label();
            this.trackBar6 = new System.Windows.Forms.TrackBar();
            this.numericUpDown6 = new System.Windows.Forms.NumericUpDown();
            this.label9 = new System.Windows.Forms.Label();
            this.trackBar5 = new System.Windows.Forms.TrackBar();
            this.numericUpDown5 = new System.Windows.Forms.NumericUpDown();
            this.label8 = new System.Windows.Forms.Label();
            this.trackBar4 = new System.Windows.Forms.TrackBar();
            this.numericUpDown4 = new System.Windows.Forms.NumericUpDown();
            this.label7 = new System.Windows.Forms.Label();
            this.trackBar3 = new System.Windows.Forms.TrackBar();
            this.numericUpDown3 = new System.Windows.Forms.NumericUpDown();
            this.label6 = new System.Windows.Forms.Label();
            this.trackBar2 = new System.Windows.Forms.TrackBar();
            this.numericUpDown2 = new System.Windows.Forms.NumericUpDown();
            this.label12 = new System.Windows.Forms.Label();
            this.label13 = new System.Windows.Forms.Label();
            this.trackBar1 = new System.Windows.Forms.TrackBar();
            this.label14 = new System.Windows.Forms.Label();
            this.label15 = new System.Windows.Forms.Label();
            this.numericUpDown1 = new System.Windows.Forms.NumericUpDown();
            this.label16 = new System.Windows.Forms.Label();
            this.resultsTabPage = new System.Windows.Forms.TabPage();
            this.textBox1 = new System.Windows.Forms.TextBox();
            this.printDialog1 = new System.Windows.Forms.PrintDialog();
            this.printDocument1 = new System.Drawing.Printing.PrintDocument();
            this.menuStrip1.SuspendLayout();
            this.tabControl1.SuspendLayout();
            this.unitsAndStreamsTabPage.SuspendLayout();
            this.impactWeightsTabPage.SuspendLayout();
            ((System.ComponentModel.ISupportInitialize)(this.trackBar8)).BeginInit();
            ((System.ComponentModel.ISupportInitialize)(this.numericUpDown8)).BeginInit();
            ((System.ComponentModel.ISupportInitialize)(this.trackBar7)).BeginInit();
            ((System.ComponentModel.ISupportInitialize)(this.numericUpDown7)).BeginInit();
            ((System.ComponentModel.ISupportInitialize)(this.trackBar6)).BeginInit();
            ((System.ComponentModel.ISupportInitialize)(this.numericUpDown6)).BeginInit();
            ((System.ComponentModel.ISupportInitialize)(this.trackBar5)).BeginInit();
            ((System.ComponentModel.ISupportInitialize)(this.numericUpDown5)).BeginInit();
            ((System.ComponentModel.ISupportInitialize)(this.trackBar4)).BeginInit();
            ((System.ComponentModel.ISupportInitialize)(this.numericUpDown4)).BeginInit();
            ((System.ComponentModel.ISupportInitialize)(this.trackBar3)).BeginInit();
            ((System.ComponentModel.ISupportInitialize)(this.numericUpDown3)).BeginInit();
            ((System.ComponentModel.ISupportInitialize)(this.trackBar2)).BeginInit();
            ((System.ComponentModel.ISupportInitialize)(this.numericUpDown2)).BeginInit();
            ((System.ComponentModel.ISupportInitialize)(this.trackBar1)).BeginInit();
            ((System.ComponentModel.ISupportInitialize)(this.numericUpDown1)).BeginInit();
            this.resultsTabPage.SuspendLayout();
            this.SuspendLayout();
            // 
            // menuStrip1
            // 
            this.menuStrip1.Items.AddRange(new System.Windows.Forms.ToolStripItem[] {
            this.toolStripMenuItem1,
            this.fileToolStripMenuItem});
            this.menuStrip1.Location = new System.Drawing.Point(0, 0);
            this.menuStrip1.Name = "menuStrip1";
            this.menuStrip1.Size = new System.Drawing.Size(919, 24);
            this.menuStrip1.TabIndex = 0;
            this.menuStrip1.Text = "menuStrip1";
            // 
            // toolStripMenuItem1
            // 
            this.toolStripMenuItem1.Name = "toolStripMenuItem1";
            this.toolStripMenuItem1.Size = new System.Drawing.Size(12, 20);
            // 
            // fileToolStripMenuItem
            // 
            this.fileToolStripMenuItem.DropDownItems.AddRange(new System.Windows.Forms.ToolStripItem[] {
            this.saveToolStripMenuItem,
            this.printToolStripMenuItem,
            this.toolStripSeparator1,
            this.exitToolStripMenuItem});
            this.fileToolStripMenuItem.Name = "fileToolStripMenuItem";
            this.fileToolStripMenuItem.Size = new System.Drawing.Size(37, 20);
            this.fileToolStripMenuItem.Text = "File";
            // 
            // saveToolStripMenuItem
            // 
            this.saveToolStripMenuItem.Name = "saveToolStripMenuItem";
            this.saveToolStripMenuItem.Size = new System.Drawing.Size(108, 22);
            this.saveToolStripMenuItem.Text = "Save...";
            this.saveToolStripMenuItem.Click += new System.EventHandler(this.saveToolStripMenuItem_Click);
            // 
            // printToolStripMenuItem
            // 
            this.printToolStripMenuItem.Name = "printToolStripMenuItem";
            this.printToolStripMenuItem.Size = new System.Drawing.Size(108, 22);
            this.printToolStripMenuItem.Text = "Print...";
            this.printToolStripMenuItem.Click += new System.EventHandler(this.printToolStripMenuItem_Click);
            // 
            // toolStripSeparator1
            // 
            this.toolStripSeparator1.Name = "toolStripSeparator1";
            this.toolStripSeparator1.Size = new System.Drawing.Size(105, 6);
            // 
            // exitToolStripMenuItem
            // 
            this.exitToolStripMenuItem.Name = "exitToolStripMenuItem";
            this.exitToolStripMenuItem.Size = new System.Drawing.Size(108, 22);
            this.exitToolStripMenuItem.Text = "Exit";
            this.exitToolStripMenuItem.Click += new System.EventHandler(this.exitToolStripMenuItem_Click);
            // 
            // tabControl1
            // 
            this.tabControl1.Controls.Add(this.unitsAndStreamsTabPage);
            this.tabControl1.Controls.Add(this.impactWeightsTabPage);
            this.tabControl1.Controls.Add(this.resultsTabPage);
            this.tabControl1.Dock = System.Windows.Forms.DockStyle.Fill;
            this.tabControl1.Location = new System.Drawing.Point(0, 24);
            this.tabControl1.Name = "tabControl1";
            this.tabControl1.SelectedIndex = 0;
            this.tabControl1.Size = new System.Drawing.Size(919, 523);
            this.tabControl1.TabIndex = 1;
            // 
            // unitsAndStreamsTabPage
            // 
            this.unitsAndStreamsTabPage.Controls.Add(this.label5);
            this.unitsAndStreamsTabPage.Controls.Add(this.includeProductStreamsCheckBox);
            this.unitsAndStreamsTabPage.Controls.Add(this.energyTypeComboBox);
            this.unitsAndStreamsTabPage.Controls.Add(this.label4);
            this.unitsAndStreamsTabPage.Controls.Add(this.considerEnergyFlowsCheckBox);
            this.unitsAndStreamsTabPage.Controls.Add(this.checkedListBox1);
            this.unitsAndStreamsTabPage.Controls.Add(this.label3);
            this.unitsAndStreamsTabPage.Controls.Add(this.feedAndInletStreamListBox);
            this.unitsAndStreamsTabPage.Controls.Add(this.label2);
            this.unitsAndStreamsTabPage.Controls.Add(this.label1);
            this.unitsAndStreamsTabPage.Controls.Add(this.AvailableUnitOpsheckedListBox);
            this.unitsAndStreamsTabPage.Location = new System.Drawing.Point(4, 22);
            this.unitsAndStreamsTabPage.Name = "unitsAndStreamsTabPage";
            this.unitsAndStreamsTabPage.Padding = new System.Windows.Forms.Padding(3);
            this.unitsAndStreamsTabPage.Size = new System.Drawing.Size(911, 497);
            this.unitsAndStreamsTabPage.TabIndex = 0;
            this.unitsAndStreamsTabPage.Text = "Units and Streams";
            this.unitsAndStreamsTabPage.UseVisualStyleBackColor = true;
            // 
            // label5
            // 
            this.label5.AutoSize = true;
            this.label5.Location = new System.Drawing.Point(37, 392);
            this.label5.Name = "label5";
            this.label5.Size = new System.Drawing.Size(35, 13);
            this.label5.TabIndex = 10;
            this.label5.Text = "label5";
            // 
            // includeProductStreamsCheckBox
            // 
            this.includeProductStreamsCheckBox.AutoSize = true;
            this.includeProductStreamsCheckBox.Location = new System.Drawing.Point(506, 434);
            this.includeProductStreamsCheckBox.Name = "includeProductStreamsCheckBox";
            this.includeProductStreamsCheckBox.Size = new System.Drawing.Size(142, 17);
            this.includeProductStreamsCheckBox.TabIndex = 9;
            this.includeProductStreamsCheckBox.Text = "Include Product Streams";
            this.includeProductStreamsCheckBox.UseVisualStyleBackColor = true;
            this.includeProductStreamsCheckBox.CheckedChanged += new System.EventHandler(this.includeProductStreamsCheckBox_CheckedChanged);
            // 
            // energyTypeComboBox
            // 
            this.energyTypeComboBox.FormattingEnabled = true;
            this.energyTypeComboBox.Items.AddRange(new object[] {
            "Coal",
            "Natural Gas",
            "Oil"});
            this.energyTypeComboBox.Location = new System.Drawing.Point(281, 431);
            this.energyTypeComboBox.Name = "energyTypeComboBox";
            this.energyTypeComboBox.Size = new System.Drawing.Size(121, 21);
            this.energyTypeComboBox.TabIndex = 8;
            this.energyTypeComboBox.Text = "Coal";
            this.energyTypeComboBox.SelectedIndexChanged += new System.EventHandler(this.energyTypeComboBox_SelectedIndexChanged);
            // 
            // label4
            // 
            this.label4.AutoSize = true;
            this.label4.Location = new System.Drawing.Point(204, 431);
            this.label4.Name = "label4";
            this.label4.Size = new System.Drawing.Size(70, 13);
            this.label4.TabIndex = 7;
            this.label4.Text = "Energy Type:";
            // 
            // considerEnergyFlowsCheckBox
            // 
            this.considerEnergyFlowsCheckBox.AutoSize = true;
            this.considerEnergyFlowsCheckBox.Location = new System.Drawing.Point(36, 431);
            this.considerEnergyFlowsCheckBox.Name = "considerEnergyFlowsCheckBox";
            this.considerEnergyFlowsCheckBox.Size = new System.Drawing.Size(133, 17);
            this.considerEnergyFlowsCheckBox.TabIndex = 6;
            this.considerEnergyFlowsCheckBox.Text = "Consider Energy Flows";
            this.considerEnergyFlowsCheckBox.UseVisualStyleBackColor = true;
            this.considerEnergyFlowsCheckBox.CheckedChanged += new System.EventHandler(this.considerEnergyFlowsCheckBox_CheckedChanged);
            // 
            // checkedListBox1
            // 
            this.checkedListBox1.FormattingEnabled = true;
            this.checkedListBox1.Location = new System.Drawing.Point(506, 70);
            this.checkedListBox1.Name = "checkedListBox1";
            this.checkedListBox1.Size = new System.Drawing.Size(197, 289);
            this.checkedListBox1.TabIndex = 5;
            this.checkedListBox1.ItemCheck += new System.Windows.Forms.ItemCheckEventHandler(this.checkedListBox1_ItemCheck);
            // 
            // label3
            // 
            this.label3.AutoSize = true;
            this.label3.Location = new System.Drawing.Point(503, 37);
            this.label3.Name = "label3";
            this.label3.Size = new System.Drawing.Size(239, 13);
            this.label3.TabIndex = 4;
            this.label3.Text = "Outlet/Product Streams (Check Product Streams)";
            // 
            // feedAndInletStreamListBox
            // 
            this.feedAndInletStreamListBox.FormattingEnabled = true;
            this.feedAndInletStreamListBox.Location = new System.Drawing.Point(259, 70);
            this.feedAndInletStreamListBox.Name = "feedAndInletStreamListBox";
            this.feedAndInletStreamListBox.Size = new System.Drawing.Size(197, 290);
            this.feedAndInletStreamListBox.TabIndex = 3;
            // 
            // label2
            // 
            this.label2.AutoSize = true;
            this.label2.Location = new System.Drawing.Point(256, 37);
            this.label2.Name = "label2";
            this.label2.Size = new System.Drawing.Size(97, 13);
            this.label2.TabIndex = 2;
            this.label2.Text = "Feed/Inlet Streams";
            // 
            // label1
            // 
            this.label1.AutoSize = true;
            this.label1.Location = new System.Drawing.Point(37, 37);
            this.label1.Name = "label1";
            this.label1.Size = new System.Drawing.Size(126, 13);
            this.label1.TabIndex = 1;
            this.label1.Text = "Available Unit Operations";
            // 
            // AvailableUnitOpsheckedListBox
            // 
            this.AvailableUnitOpsheckedListBox.FormattingEnabled = true;
            this.AvailableUnitOpsheckedListBox.Location = new System.Drawing.Point(37, 70);
            this.AvailableUnitOpsheckedListBox.Name = "AvailableUnitOpsheckedListBox";
            this.AvailableUnitOpsheckedListBox.Size = new System.Drawing.Size(173, 289);
            this.AvailableUnitOpsheckedListBox.TabIndex = 0;
            this.AvailableUnitOpsheckedListBox.ItemCheck += new System.Windows.Forms.ItemCheckEventHandler(this.AvailableUnitOpsheckedListBox_ItemCheck);
            // 
            // impactWeightsTabPage
            // 
            this.impactWeightsTabPage.Controls.Add(this.trackBar8);
            this.impactWeightsTabPage.Controls.Add(this.numericUpDown8);
            this.impactWeightsTabPage.Controls.Add(this.label11);
            this.impactWeightsTabPage.Controls.Add(this.trackBar7);
            this.impactWeightsTabPage.Controls.Add(this.numericUpDown7);
            this.impactWeightsTabPage.Controls.Add(this.label10);
            this.impactWeightsTabPage.Controls.Add(this.trackBar6);
            this.impactWeightsTabPage.Controls.Add(this.numericUpDown6);
            this.impactWeightsTabPage.Controls.Add(this.label9);
            this.impactWeightsTabPage.Controls.Add(this.trackBar5);
            this.impactWeightsTabPage.Controls.Add(this.numericUpDown5);
            this.impactWeightsTabPage.Controls.Add(this.label8);
            this.impactWeightsTabPage.Controls.Add(this.trackBar4);
            this.impactWeightsTabPage.Controls.Add(this.numericUpDown4);
            this.impactWeightsTabPage.Controls.Add(this.label7);
            this.impactWeightsTabPage.Controls.Add(this.trackBar3);
            this.impactWeightsTabPage.Controls.Add(this.numericUpDown3);
            this.impactWeightsTabPage.Controls.Add(this.label6);
            this.impactWeightsTabPage.Controls.Add(this.trackBar2);
            this.impactWeightsTabPage.Controls.Add(this.numericUpDown2);
            this.impactWeightsTabPage.Controls.Add(this.label12);
            this.impactWeightsTabPage.Controls.Add(this.label13);
            this.impactWeightsTabPage.Controls.Add(this.trackBar1);
            this.impactWeightsTabPage.Controls.Add(this.label14);
            this.impactWeightsTabPage.Controls.Add(this.label15);
            this.impactWeightsTabPage.Controls.Add(this.numericUpDown1);
            this.impactWeightsTabPage.Controls.Add(this.label16);
            this.impactWeightsTabPage.Location = new System.Drawing.Point(4, 22);
            this.impactWeightsTabPage.Name = "impactWeightsTabPage";
            this.impactWeightsTabPage.Padding = new System.Windows.Forms.Padding(3);
            this.impactWeightsTabPage.Size = new System.Drawing.Size(911, 497);
            this.impactWeightsTabPage.TabIndex = 1;
            this.impactWeightsTabPage.Text = "Impact Weights";
            this.impactWeightsTabPage.UseVisualStyleBackColor = true;
            // 
            // trackBar8
            // 
            this.trackBar8.Location = new System.Drawing.Point(435, 412);
            this.trackBar8.Name = "trackBar8";
            this.trackBar8.Size = new System.Drawing.Size(239, 45);
            this.trackBar8.TabIndex = 53;
            this.trackBar8.Value = 1;
            // 
            // numericUpDown8
            // 
            this.numericUpDown8.DecimalPlaces = 1;
            this.numericUpDown8.Increment = new decimal(new int[] {
            1,
            0,
            0,
            65536});
            this.numericUpDown8.Location = new System.Drawing.Point(268, 423);
            this.numericUpDown8.Maximum = new decimal(new int[] {
            10,
            0,
            0,
            0});
            this.numericUpDown8.Name = "numericUpDown8";
            this.numericUpDown8.Size = new System.Drawing.Size(120, 20);
            this.numericUpDown8.TabIndex = 52;
            this.numericUpDown8.Value = new decimal(new int[] {
            1,
            0,
            0,
            0});
            // 
            // label11
            // 
            this.label11.AutoSize = true;
            this.label11.Location = new System.Drawing.Point(76, 425);
            this.label11.Name = "label11";
            this.label11.Size = new System.Drawing.Size(64, 13);
            this.label11.TabIndex = 51;
            this.label11.Text = "Acidification";
            // 
            // trackBar7
            // 
            this.trackBar7.Location = new System.Drawing.Point(435, 361);
            this.trackBar7.Name = "trackBar7";
            this.trackBar7.Size = new System.Drawing.Size(239, 45);
            this.trackBar7.TabIndex = 50;
            this.trackBar7.Value = 1;
            // 
            // numericUpDown7
            // 
            this.numericUpDown7.DecimalPlaces = 1;
            this.numericUpDown7.Increment = new decimal(new int[] {
            1,
            0,
            0,
            65536});
            this.numericUpDown7.Location = new System.Drawing.Point(268, 372);
            this.numericUpDown7.Maximum = new decimal(new int[] {
            10,
            0,
            0,
            0});
            this.numericUpDown7.Name = "numericUpDown7";
            this.numericUpDown7.Size = new System.Drawing.Size(120, 20);
            this.numericUpDown7.TabIndex = 49;
            this.numericUpDown7.Value = new decimal(new int[] {
            1,
            0,
            0,
            0});
            // 
            // label10
            // 
            this.label10.AutoSize = true;
            this.label10.Location = new System.Drawing.Point(76, 374);
            this.label10.Name = "label10";
            this.label10.Size = new System.Drawing.Size(124, 13);
            this.label10.TabIndex = 48;
            this.label10.Text = "Photochemical Oxidation";
            // 
            // trackBar6
            // 
            this.trackBar6.Location = new System.Drawing.Point(435, 310);
            this.trackBar6.Name = "trackBar6";
            this.trackBar6.Size = new System.Drawing.Size(239, 45);
            this.trackBar6.TabIndex = 47;
            this.trackBar6.Value = 1;
            // 
            // numericUpDown6
            // 
            this.numericUpDown6.DecimalPlaces = 1;
            this.numericUpDown6.Increment = new decimal(new int[] {
            1,
            0,
            0,
            65536});
            this.numericUpDown6.Location = new System.Drawing.Point(268, 321);
            this.numericUpDown6.Maximum = new decimal(new int[] {
            10,
            0,
            0,
            0});
            this.numericUpDown6.Name = "numericUpDown6";
            this.numericUpDown6.Size = new System.Drawing.Size(120, 20);
            this.numericUpDown6.TabIndex = 46;
            this.numericUpDown6.Value = new decimal(new int[] {
            1,
            0,
            0,
            0});
            // 
            // label9
            // 
            this.label9.AutoSize = true;
            this.label9.Location = new System.Drawing.Point(76, 323);
            this.label9.Name = "label9";
            this.label9.Size = new System.Drawing.Size(84, 13);
            this.label9.TabIndex = 45;
            this.label9.Text = "Ozone depletion";
            // 
            // trackBar5
            // 
            this.trackBar5.Location = new System.Drawing.Point(435, 259);
            this.trackBar5.Name = "trackBar5";
            this.trackBar5.Size = new System.Drawing.Size(239, 45);
            this.trackBar5.TabIndex = 44;
            this.trackBar5.Value = 1;
            // 
            // numericUpDown5
            // 
            this.numericUpDown5.DecimalPlaces = 1;
            this.numericUpDown5.Increment = new decimal(new int[] {
            1,
            0,
            0,
            65536});
            this.numericUpDown5.Location = new System.Drawing.Point(268, 270);
            this.numericUpDown5.Maximum = new decimal(new int[] {
            10,
            0,
            0,
            0});
            this.numericUpDown5.Name = "numericUpDown5";
            this.numericUpDown5.Size = new System.Drawing.Size(120, 20);
            this.numericUpDown5.TabIndex = 43;
            this.numericUpDown5.Value = new decimal(new int[] {
            1,
            0,
            0,
            0});
            // 
            // label8
            // 
            this.label8.AutoSize = true;
            this.label8.Location = new System.Drawing.Point(76, 272);
            this.label8.Name = "label8";
            this.label8.Size = new System.Drawing.Size(82, 13);
            this.label8.TabIndex = 42;
            this.label8.Text = "Global Warming";
            // 
            // trackBar4
            // 
            this.trackBar4.Location = new System.Drawing.Point(435, 208);
            this.trackBar4.Name = "trackBar4";
            this.trackBar4.Size = new System.Drawing.Size(239, 45);
            this.trackBar4.TabIndex = 41;
            this.trackBar4.Value = 1;
            // 
            // numericUpDown4
            // 
            this.numericUpDown4.DecimalPlaces = 1;
            this.numericUpDown4.Increment = new decimal(new int[] {
            1,
            0,
            0,
            65536});
            this.numericUpDown4.Location = new System.Drawing.Point(268, 219);
            this.numericUpDown4.Maximum = new decimal(new int[] {
            10,
            0,
            0,
            0});
            this.numericUpDown4.Name = "numericUpDown4";
            this.numericUpDown4.Size = new System.Drawing.Size(120, 20);
            this.numericUpDown4.TabIndex = 40;
            this.numericUpDown4.Value = new decimal(new int[] {
            1,
            0,
            0,
            0});
            // 
            // label7
            // 
            this.label7.AutoSize = true;
            this.label7.Location = new System.Drawing.Point(76, 221);
            this.label7.Name = "label7";
            this.label7.Size = new System.Drawing.Size(92, 13);
            this.label7.TabIndex = 39;
            this.label7.Text = "Terrestrial Toxicity";
            // 
            // trackBar3
            // 
            this.trackBar3.Location = new System.Drawing.Point(435, 157);
            this.trackBar3.Name = "trackBar3";
            this.trackBar3.Size = new System.Drawing.Size(239, 45);
            this.trackBar3.TabIndex = 38;
            this.trackBar3.Value = 1;
            // 
            // numericUpDown3
            // 
            this.numericUpDown3.DecimalPlaces = 1;
            this.numericUpDown3.Increment = new decimal(new int[] {
            1,
            0,
            0,
            65536});
            this.numericUpDown3.Location = new System.Drawing.Point(268, 168);
            this.numericUpDown3.Maximum = new decimal(new int[] {
            10,
            0,
            0,
            0});
            this.numericUpDown3.Name = "numericUpDown3";
            this.numericUpDown3.Size = new System.Drawing.Size(120, 20);
            this.numericUpDown3.TabIndex = 37;
            this.numericUpDown3.Value = new decimal(new int[] {
            1,
            0,
            0,
            0});
            // 
            // label6
            // 
            this.label6.AutoSize = true;
            this.label6.Location = new System.Drawing.Point(76, 170);
            this.label6.Name = "label6";
            this.label6.Size = new System.Drawing.Size(82, 13);
            this.label6.TabIndex = 36;
            this.label6.Text = "Aquatic Toxicity";
            // 
            // trackBar2
            // 
            this.trackBar2.Location = new System.Drawing.Point(435, 106);
            this.trackBar2.Name = "trackBar2";
            this.trackBar2.Size = new System.Drawing.Size(239, 45);
            this.trackBar2.TabIndex = 35;
            this.trackBar2.Value = 1;
            // 
            // numericUpDown2
            // 
            this.numericUpDown2.DecimalPlaces = 1;
            this.numericUpDown2.Increment = new decimal(new int[] {
            1,
            0,
            0,
            65536});
            this.numericUpDown2.Location = new System.Drawing.Point(268, 117);
            this.numericUpDown2.Maximum = new decimal(new int[] {
            10,
            0,
            0,
            0});
            this.numericUpDown2.Name = "numericUpDown2";
            this.numericUpDown2.Size = new System.Drawing.Size(120, 20);
            this.numericUpDown2.TabIndex = 34;
            this.numericUpDown2.Value = new decimal(new int[] {
            1,
            0,
            0,
            0});
            // 
            // label12
            // 
            this.label12.AutoSize = true;
            this.label12.Location = new System.Drawing.Point(76, 119);
            this.label12.Name = "label12";
            this.label12.Size = new System.Drawing.Size(170, 13);
            this.label12.TabIndex = 33;
            this.label12.Text = "Human Toxicity: Dermal/Inhalation";
            // 
            // label13
            // 
            this.label13.AutoSize = true;
            this.label13.Location = new System.Drawing.Point(495, 39);
            this.label13.Name = "label13";
            this.label13.Size = new System.Drawing.Size(112, 13);
            this.label13.TabIndex = 32;
            this.label13.Text = "User Assigned Weight";
            // 
            // trackBar1
            // 
            this.trackBar1.Location = new System.Drawing.Point(435, 55);
            this.trackBar1.Name = "trackBar1";
            this.trackBar1.Size = new System.Drawing.Size(239, 45);
            this.trackBar1.TabIndex = 31;
            this.trackBar1.Value = 1;
            // 
            // label14
            // 
            this.label14.AutoSize = true;
            this.label14.Location = new System.Drawing.Point(655, 39);
            this.label14.Name = "label14";
            this.label14.Size = new System.Drawing.Size(29, 13);
            this.label14.TabIndex = 30;
            this.label14.Text = "High";
            // 
            // label15
            // 
            this.label15.AutoSize = true;
            this.label15.Location = new System.Drawing.Point(415, 39);
            this.label15.Name = "label15";
            this.label15.Size = new System.Drawing.Size(27, 13);
            this.label15.TabIndex = 29;
            this.label15.Text = "Low";
            // 
            // numericUpDown1
            // 
            this.numericUpDown1.DecimalPlaces = 1;
            this.numericUpDown1.Increment = new decimal(new int[] {
            1,
            0,
            0,
            65536});
            this.numericUpDown1.Location = new System.Drawing.Point(268, 66);
            this.numericUpDown1.Maximum = new decimal(new int[] {
            10,
            0,
            0,
            0});
            this.numericUpDown1.Name = "numericUpDown1";
            this.numericUpDown1.Size = new System.Drawing.Size(120, 20);
            this.numericUpDown1.TabIndex = 28;
            this.numericUpDown1.Value = new decimal(new int[] {
            1,
            0,
            0,
            0});
            // 
            // label16
            // 
            this.label16.AutoSize = true;
            this.label16.Location = new System.Drawing.Point(76, 68);
            this.label16.Name = "label16";
            this.label16.Size = new System.Drawing.Size(129, 13);
            this.label16.TabIndex = 27;
            this.label16.Text = "Human Toxicity: Ingestion";
            // 
            // resultsTabPage
            // 
            this.resultsTabPage.Controls.Add(this.textBox1);
            this.resultsTabPage.Location = new System.Drawing.Point(4, 22);
            this.resultsTabPage.Name = "resultsTabPage";
            this.resultsTabPage.Size = new System.Drawing.Size(911, 497);
            this.resultsTabPage.TabIndex = 2;
            this.resultsTabPage.Text = "Results";
            this.resultsTabPage.UseVisualStyleBackColor = true;
            // 
            // textBox1
            // 
            this.textBox1.Dock = System.Windows.Forms.DockStyle.Fill;
            this.textBox1.Font = new System.Drawing.Font("Courier New", 8.25F, System.Drawing.FontStyle.Regular, System.Drawing.GraphicsUnit.Point, ((byte)(0)));
            this.textBox1.Location = new System.Drawing.Point(0, 0);
            this.textBox1.Multiline = true;
            this.textBox1.Name = "textBox1";
            this.textBox1.ScrollBars = System.Windows.Forms.ScrollBars.Both;
            this.textBox1.Size = new System.Drawing.Size(911, 497);
            this.textBox1.TabIndex = 0;
            // 
            // printDialog1
            // 
            this.printDialog1.UseEXDialog = true;
            // 
            // printDocument1
            // 
            this.printDocument1.PrintPage += new System.Drawing.Printing.PrintPageEventHandler(this.printDocument1_PrintPage);
            // 
            // WARalgorithm
            // 
            this.AutoScaleDimensions = new System.Drawing.SizeF(6F, 13F);
            this.AutoScaleMode = System.Windows.Forms.AutoScaleMode.Font;
            this.ClientSize = new System.Drawing.Size(919, 547);
            this.Controls.Add(this.tabControl1);
            this.Controls.Add(this.menuStrip1);
            this.Icon = ((System.Drawing.Icon)(resources.GetObject("$this.Icon")));
            this.MainMenuStrip = this.menuStrip1;
            this.Name = "WARalgorithm";
            this.Text = "WARalgorithm";
            this.menuStrip1.ResumeLayout(false);
            this.menuStrip1.PerformLayout();
            this.tabControl1.ResumeLayout(false);
            this.unitsAndStreamsTabPage.ResumeLayout(false);
            this.unitsAndStreamsTabPage.PerformLayout();
            this.impactWeightsTabPage.ResumeLayout(false);
            this.impactWeightsTabPage.PerformLayout();
            ((System.ComponentModel.ISupportInitialize)(this.trackBar8)).EndInit();
            ((System.ComponentModel.ISupportInitialize)(this.numericUpDown8)).EndInit();
            ((System.ComponentModel.ISupportInitialize)(this.trackBar7)).EndInit();
            ((System.ComponentModel.ISupportInitialize)(this.numericUpDown7)).EndInit();
            ((System.ComponentModel.ISupportInitialize)(this.trackBar6)).EndInit();
            ((System.ComponentModel.ISupportInitialize)(this.numericUpDown6)).EndInit();
            ((System.ComponentModel.ISupportInitialize)(this.trackBar5)).EndInit();
            ((System.ComponentModel.ISupportInitialize)(this.numericUpDown5)).EndInit();
            ((System.ComponentModel.ISupportInitialize)(this.trackBar4)).EndInit();
            ((System.ComponentModel.ISupportInitialize)(this.numericUpDown4)).EndInit();
            ((System.ComponentModel.ISupportInitialize)(this.trackBar3)).EndInit();
            ((System.ComponentModel.ISupportInitialize)(this.numericUpDown3)).EndInit();
            ((System.ComponentModel.ISupportInitialize)(this.trackBar2)).EndInit();
            ((System.ComponentModel.ISupportInitialize)(this.numericUpDown2)).EndInit();
            ((System.ComponentModel.ISupportInitialize)(this.trackBar1)).EndInit();
            ((System.ComponentModel.ISupportInitialize)(this.numericUpDown1)).EndInit();
            this.resultsTabPage.ResumeLayout(false);
            this.resultsTabPage.PerformLayout();
            this.ResumeLayout(false);
            this.PerformLayout();

        }

        #endregion

        private System.Windows.Forms.ToolTip toolTip1;
        private System.Windows.Forms.MenuStrip menuStrip1;
        private System.Windows.Forms.ToolStripMenuItem toolStripMenuItem1;
        private System.Windows.Forms.ToolStripMenuItem fileToolStripMenuItem;
        private System.Windows.Forms.ToolStripMenuItem saveToolStripMenuItem;
        private System.Windows.Forms.ToolStripMenuItem printToolStripMenuItem;
        private System.Windows.Forms.ToolStripSeparator toolStripSeparator1;
        private System.Windows.Forms.ToolStripMenuItem exitToolStripMenuItem;
        private System.Windows.Forms.TabControl tabControl1;
        private System.Windows.Forms.TabPage unitsAndStreamsTabPage;
        private System.Windows.Forms.ComboBox energyTypeComboBox;
        private System.Windows.Forms.Label label4;
        private System.Windows.Forms.CheckBox considerEnergyFlowsCheckBox;
        private System.Windows.Forms.CheckedListBox checkedListBox1;
        private System.Windows.Forms.Label label3;
        private System.Windows.Forms.ListBox feedAndInletStreamListBox;
        private System.Windows.Forms.Label label2;
        private System.Windows.Forms.Label label1;
        private System.Windows.Forms.CheckedListBox AvailableUnitOpsheckedListBox;
        private System.Windows.Forms.TabPage impactWeightsTabPage;
        private System.Windows.Forms.TabPage resultsTabPage;
        private System.Windows.Forms.CheckBox includeProductStreamsCheckBox;
        private System.Windows.Forms.Label label5;
        private System.Windows.Forms.TrackBar trackBar8;
        private System.Windows.Forms.NumericUpDown numericUpDown8;
        private System.Windows.Forms.Label label11;
        private System.Windows.Forms.TrackBar trackBar7;
        private System.Windows.Forms.NumericUpDown numericUpDown7;
        private System.Windows.Forms.Label label10;
        private System.Windows.Forms.TrackBar trackBar6;
        private System.Windows.Forms.NumericUpDown numericUpDown6;
        private System.Windows.Forms.Label label9;
        private System.Windows.Forms.TrackBar trackBar5;
        private System.Windows.Forms.NumericUpDown numericUpDown5;
        private System.Windows.Forms.Label label8;
        private System.Windows.Forms.TrackBar trackBar4;
        private System.Windows.Forms.NumericUpDown numericUpDown4;
        private System.Windows.Forms.Label label7;
        private System.Windows.Forms.TrackBar trackBar3;
        private System.Windows.Forms.NumericUpDown numericUpDown3;
        private System.Windows.Forms.Label label6;
        private System.Windows.Forms.TrackBar trackBar2;
        private System.Windows.Forms.NumericUpDown numericUpDown2;
        private System.Windows.Forms.Label label12;
        private System.Windows.Forms.Label label13;
        private System.Windows.Forms.TrackBar trackBar1;
        private System.Windows.Forms.Label label14;
        private System.Windows.Forms.Label label15;
        private System.Windows.Forms.NumericUpDown numericUpDown1;
        private System.Windows.Forms.Label label16;
        private System.Windows.Forms.TextBox textBox1;
        private System.Windows.Forms.PrintDialog printDialog1;
        private System.Drawing.Printing.PrintDocument printDocument1;
    }
}