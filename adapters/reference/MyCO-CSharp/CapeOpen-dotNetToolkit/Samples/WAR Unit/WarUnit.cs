#region Copyright (c) 2005-2013, Mose, All rights reserved.
/*
// Copyright (c) 2005-2013, Mose Srl (http://www.mose.units.it/)
// Original capeopentoolkit.net Source Code: Copyright (c) 2005, Marco Carone, Marco Parenzan (e-mail: marco.carone@yahoo.it; marco.parenzan@libero.it)
// All rights reserved.
//  
// Redistribution and use in source and binary forms, with or without modification, are permitted 
// provided that the following conditions are met: 
//  
// (1) Redistributions of source code must retain the above copyright notice, this list of 
// conditions and the following disclaimer. 
// (2) Redistributions in binary form must reproduce the above copyright notice, this list of 
// conditions and the following disclaimer in the documentation and/or other materials 
// provided with the distribution. 
// (3) Neither the name of the Mose nor the names of its contributors may be used 
// to endorse or promote products derived from this software without specific prior 
// written permission.
//      
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS 
// OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY 
// AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR 
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL 
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, 
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER 
// IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT 
// OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
// -------------------------------------------------------------------------
//
// Original capeopentoolkit.net Source Code: Copyright (c) 2005, Marco Carone, Marco Parenzan (e-mail: marco.carone@yahoo.it; marco.parenzan@libero.it)
// 
// Mose is a registered trademark of Mose Srl.
// 
// For portions of this software, the some additional copyright notices may apply 
// which can either be found in the license.txt file included in the source distribution
// or following this notice. 
//
*/
#endregion

namespace CapeOpenWar 
{
    using System;
    using System.Collections.Generic;
    using System.Text;
    using Mose.CapeOpenToolkit.UnitProxy;
    using System.Runtime.InteropServices;
    using System.IO;
    
    
    public partial class WarUnit : Mose.CapeOpenToolkit.UnitProxy.CapeOpenUnitSimple {
        
        protected override void OnCalculate() 
        {
            double zTemperature = InputPort.Temperature;
            double zPressure = InputPort.Pressure;
            double zEnthalpy = InputPort.Enthalpy;
            double zTotalFlow = InputPort.TotalFlow;
            double[] zFraction = InputPort.Fraction;

            // totalflow ii inmultit cu 3.6 pt. ca CO il citeste in mol/s si tie iti trebuie in kmol/hr
            double zTotmod = zTotalFlow * 3.6;


            string[] zIds = InputPort.Ids;

            string[] zPropertyValue = new string[1];
            zPropertyValue[0] = "casRegistryNumber";

            string[] zPropVal = null;
            object[] zObj;

            zObj = (object[])InputPort.ConnectedObject.GetComponentConstant(zPropertyValue, zIds);
            zPropVal = new string[zObj.GetLength(0)];
            for (int i = 0; i < zObj.GetLength(0); i++)
            {
                zPropVal[i] = (string)zObj[i];
            }

            //Volume
            InputPort.CalculateProperty(CapeOpenThermoMaterialPropertyType.Volume, CapeOpenPhaseType.Overall, CapeOpenCalculationType.Pure);
            double zVolume = InputPort.Volume;

            Dictionary<string, double> zComponentsList = new Dictionary<string, double>();
            DataAccessHelper zAccessoDati = new DataAccessHelper();

            foreach (string zCas in zPropVal)
            {
                double zSumIndexNorm = 0;
                try
                {
                    zSumIndexNorm = zAccessoDati.GetSumIndexNorm(zCas);
                }
                catch
                {
                    return;
                }
                zComponentsList.Add(zCas, zSumIndexNorm);
            }

            double zInCP;
            double zInEP;
            double zOutCP;
            double zOutEP;

            double MM;

            MM = 0;

            double[] zMolecularWeights;
            zObj = (object[])InputPort.ConnectedObject.GetComponentConstant("molecularWeight", zIds);
            zMolecularWeights = new double[zObj.GetLength(0)];

            for (int i = 0; i < zObj.GetLength(0); i++)
            {
                zMolecularWeights[i] = (double)zObj[i];
            }

            //calculate of molecular weight of the stream
            for (int i = 0; i < InputPort.ComponentsCount; i++)
            {
                MM = MM + zFraction[i] * zMolecularWeights[i];
            }

            double[] zFractionMasses = new double[InputPort.ComponentsCount];

            for (int i = 0; i < InputPort.ComponentsCount; i++)
            {
                zFractionMasses[i] = zFraction[i] * zMolecularWeights[i] / MM;
            }



            StreamWriter zWriter;

            if (ProcessTypeValue == 0.0)
            {//for process_type=0 calc impact
                if (CapePositionValue == 0.0)
                {//    ' if the stream is the begin of process
                    zInCP = 0;
                    double zIndividual0;
                    for (int r0 = 0; r0 < InputPort.ComponentsCount; r0++)
                    {
                        zIndividual0 = zTotmod * MM * zFraction[r0] * zComponentsList[zPropVal[r0]];
                        zInCP = zInCP + zIndividual0;
                    }
                    zWriter = new StreamWriter("WAR.txt", true, Encoding.UTF8);
                    zWriter.WriteLine("                   ");
                    zWriter.WriteLine(" type:" + ProcessTypeValue.ToString());
                    zWriter.WriteLine(" name:CO" + CapeIDValue.ToString());
                    zWriter.WriteLine("  ind:" + CapePositionValue.ToString());
                    zWriter.WriteLine("value:" + zInCP.ToString());
                    zWriter.Close();
                    //' write the mass flow results
                    zWriter = new StreamWriter("stream.txt", true, Encoding.UTF8);
                    zWriter.WriteLine("                   ");
                    zWriter.WriteLine("          type:" + ProcessTypeValue.ToString());
                    zWriter.WriteLine("          name:CO" + CapeIDValue.ToString());
                    zWriter.WriteLine("           ind:" + CapePositionValue.ToString());
                    zWriter.WriteLine("mass flow-rate:" + Convert.ToString(zTotmod * MM));
                    zWriter.Close();
                    //' write all results
                    zWriter = new StreamWriter("results.txt", true, Encoding.UTF8);
                    zWriter.WriteLine("                   ");
                    zWriter.WriteLine("          type:" + ProcessTypeValue.ToString());
                    zWriter.WriteLine("          name:CO" + CapeIDValue.ToString());
                    zWriter.WriteLine("           ind:" + CapePositionValue.ToString());
                    zWriter.WriteLine("         value:" + zInCP.ToString());
                    zWriter.WriteLine("mass flow-rate:" + Convert.ToString(zTotmod * MM));
                    zWriter.Close();
                }
                else if (CapePositionValue == 1.0)
                {
                    double zIndividual1;
                    zOutCP = 0;
                    zWriter = new StreamWriter("stream_name_and_composition.txt", true, Encoding.UTF8);
                    zWriter.WriteLine("                   ");
                    zWriter.WriteLine(" type:" + ProcessTypeValue.ToString());
                    zWriter.WriteLine(" name:CO" + CapeIDValue.ToString());
                    zWriter.WriteLine("  ind:" + CapePositionValue.ToString());

                    for (int g0 = 0; g0 < InputPort.ComponentsCount; g0++)
                    {
                        zIndividual1 = zTotmod * MM * zFractionMasses[g0] * zComponentsList[zPropVal[g0]];
                        zOutCP = zOutCP + zIndividual1;
                        double zCompFlowRateWasteCp = zTotmod * MM * zFractionMasses[g0];
                        zWriter.WriteLine(zIds[g0]);
                        zWriter.WriteLine("comp.mass flow-rates=" + zCompFlowRateWasteCp.ToString());
                    }
                    zWriter.Close();

                    zWriter = new StreamWriter("WAR.txt", true, Encoding.UTF8);
                    zWriter.WriteLine("                   ");
                    zWriter.WriteLine(" type:" + ProcessTypeValue.ToString());
                    zWriter.WriteLine(" name:CO" + CapeIDValue.ToString());
                    zWriter.WriteLine("  ind:" + CapePositionValue.ToString());
                    zWriter.WriteLine("value:" + zOutCP.ToString());
                    zWriter.Close();
                    //' write the mass flow results
                    zWriter = new StreamWriter("stream.txt", true, Encoding.UTF8);
                    zWriter.WriteLine("                   ");
                    zWriter.WriteLine("          type:" + ProcessTypeValue.ToString());
                    zWriter.WriteLine("          name:CO" + CapeIDValue.ToString());
                    zWriter.WriteLine("           ind:" + CapePositionValue.ToString());
                    zWriter.WriteLine("mass flow-rate:" + Convert.ToString(zTotmod * MM));
                    zWriter.Close();
                    zWriter = new StreamWriter("results.txt", true, Encoding.UTF8);
                    zWriter.WriteLine("                   ");
                    zWriter.WriteLine("          type:" + ProcessTypeValue.ToString());
                    zWriter.WriteLine("          name:CO" + CapeIDValue.ToString());
                    zWriter.WriteLine("           ind:" + CapePositionValue.ToString());
                    zWriter.WriteLine("         value:" + zOutCP.ToString());
                    zWriter.WriteLine("mass flow-rate:" + Convert.ToString(zTotmod * MM));
                    zWriter.Close();
                }
                else if (CapePositionValue == 2.0)
                { //if the stream is the end of the process

                    zWriter = new StreamWriter("stream_name_and_composition.txt", true, Encoding.UTF8);
                    zWriter.WriteLine("                   ");
                    zWriter.WriteLine(" type:" + ProcessTypeValue.ToString());
                    zWriter.WriteLine(" name:CO" + CapeIDValue.ToString());
                    zWriter.WriteLine("  ind:" + CapePositionValue.ToString());

                    for (int u0 = 0; u0 < InputPort.ComponentsCount; u0++)
                    {
                        double zCompFlowRateWasteCp = zTotmod * MM * zFractionMasses[u0];
                        zWriter.WriteLine(zIds[u0]);
                        zWriter.WriteLine("comp.mass flow-rates=" + zCompFlowRateWasteCp.ToString());
                    }
                    zWriter.Close();

                    zWriter = new StreamWriter("WAR.txt", true, Encoding.UTF8);
                    zWriter.WriteLine("                   ");
                    zWriter.WriteLine(" type:" + ProcessTypeValue.ToString());
                    zWriter.WriteLine(" name:CO" + CapeIDValue.ToString());
                    zWriter.WriteLine("  ind:" + CapePositionValue.ToString());
                    zWriter.WriteLine("value:output stream of the chemical process");
                    zWriter.Close();
                    //' write the mass flow results
                    zWriter = new StreamWriter("stream.txt", true, Encoding.UTF8);
                    zWriter.WriteLine("                   ");
                    zWriter.WriteLine("          type:" + ProcessTypeValue.ToString());
                    zWriter.WriteLine("          name:CO" + CapeIDValue.ToString());
                    zWriter.WriteLine("           ind:" + CapePositionValue.ToString());
                    zWriter.WriteLine("mass flow-rate:" + Convert.ToString(zTotmod * MM));
                    zWriter.Close();
                    zWriter = new StreamWriter("results.txt", true, Encoding.UTF8);
                    zWriter.WriteLine("                   ");
                    zWriter.WriteLine("          type:" + ProcessTypeValue.ToString());
                    zWriter.WriteLine("          name:CO" + CapeIDValue.ToString());
                    zWriter.WriteLine("           ind:" + CapePositionValue.ToString());
                    zWriter.WriteLine("         value:0");
                    zWriter.WriteLine("mass flow-rate:" + Convert.ToString(zTotmod * MM));
                    zWriter.Close();
                }
                else if (CapePositionValue == 3.0)
                { //if the stream is internal
                    zWriter = new StreamWriter("WAR.txt", true, Encoding.UTF8);
                    zWriter.WriteLine("                   ");
                    zWriter.WriteLine(" type:" + ProcessTypeValue.ToString());
                    zWriter.WriteLine(" name:CO" + CapeIDValue.ToString());
                    zWriter.WriteLine("  ind:" + CapePositionValue.ToString());
                    zWriter.WriteLine("value:internal stream of the chemical process");
                    zWriter.Close();
                    //' write the mass flow results
                    zWriter = new StreamWriter("stream.txt", true, Encoding.UTF8);
                    zWriter.WriteLine("                   ");
                    zWriter.WriteLine("          type:" + ProcessTypeValue.ToString());
                    zWriter.WriteLine("          name:CO" + CapeIDValue.ToString());
                    zWriter.WriteLine("           ind:" + CapePositionValue.ToString());
                    zWriter.WriteLine("mass flow-rate:" + Convert.ToString(zTotmod * MM));
                    zWriter.Close();
                    zWriter = new StreamWriter("results.txt", true, Encoding.UTF8);
                    zWriter.WriteLine("                   ");
                    zWriter.WriteLine("          type:" + ProcessTypeValue.ToString());
                    zWriter.WriteLine("          name:CO" + CapeIDValue.ToString());
                    zWriter.WriteLine("           ind:" + CapePositionValue.ToString());
                    zWriter.WriteLine("         value:0");
                    zWriter.WriteLine("mass flow-rate:" + Convert.ToString(zTotmod * MM));
                    zWriter.Close();
                }
            }

            if (ProcessTypeValue == 1.0)
            {//' for process_type=1 calc. the impactof process of generation of energy
                if (CapePositionValue == 0.0)
                {// ' beginning of the process
                    zInEP = 0;
                    double zIndividual2;
                    for (int r1 = 0; r1 < InputPort.ComponentsCount; r1++)
                    {
                        zIndividual2 = zTotmod * MM * zFraction[r1] * zComponentsList[zPropVal[r1]];
                        zInEP = zInEP + zIndividual2;
                    }
                    zWriter = new StreamWriter("WAR.txt", true, Encoding.UTF8);
                    zWriter.WriteLine("                   ");
                    zWriter.WriteLine(" type:" + ProcessTypeValue.ToString());
                    zWriter.WriteLine(" name:CO" + CapeIDValue.ToString());
                    zWriter.WriteLine("  ind:" + CapePositionValue.ToString());
                    zWriter.WriteLine("value:" + zInEP.ToString());
                    zWriter.Close();
                    //' write the mass flow results
                    zWriter = new StreamWriter("stream.txt", true, Encoding.UTF8);
                    zWriter.WriteLine("                   ");
                    zWriter.WriteLine("          type:" + ProcessTypeValue.ToString());
                    zWriter.WriteLine("          name:CO" + CapeIDValue.ToString());
                    zWriter.WriteLine("           ind:" + CapePositionValue.ToString());
                    zWriter.WriteLine("mass flow-rate:" + Convert.ToString(zTotmod * MM));
                    zWriter.Close();
                    //' write all results
                    zWriter = new StreamWriter("results.txt", true, Encoding.UTF8);
                    zWriter.WriteLine("                   ");
                    zWriter.WriteLine("          type:" + ProcessTypeValue.ToString());
                    zWriter.WriteLine("          name:CO" + CapeIDValue.ToString());
                    zWriter.WriteLine("           ind:" + CapePositionValue.ToString());
                    zWriter.WriteLine("         value:" + zInEP.ToString());
                    zWriter.WriteLine("mass flow-rate:" + Convert.ToString(zTotmod * MM));
                    zWriter.Close();
                }
                else if (CapePositionValue == 1.0)
                {//end of process
                    zWriter = new StreamWriter("stream_name_and_composition.txt", true, Encoding.UTF8);
                    zWriter.WriteLine("                   ");
                    zWriter.WriteLine(" type:" + ProcessTypeValue.ToString());
                    zWriter.WriteLine(" name:CO" + CapeIDValue.ToString());
                    zWriter.WriteLine("  ind:" + CapePositionValue.ToString());
                    for (int k0 = 0; k0 < InputPort.ComponentsCount; k0++)
                    {
                        double zCompFlowRateWasteEp = zTotmod * MM * zFractionMasses[k0];
                        zWriter.WriteLine(zIds[k0]);
                        zWriter.WriteLine("comp.mass flow-rates=" + zCompFlowRateWasteEp.ToString());
                    }
                    zWriter.Close();

                    //calculaction of the energy
                    StreamReader zReader = new StreamReader("ENERGY.txt");
                    string s = null;
                    while (!zReader.EndOfStream)
                    {
                        s = zReader.ReadLine();
                    }
                    zReader.Close();
                    s = s.Substring(s.Length - 15);

                    double zHProcess = Convert.ToDouble(s.Trim());
                    if (zHProcess < 0)
                    {
                        zOutEP = 0;
                        for (int g1 = 0; g1 < InputPort.ComponentsCount; g1++)
                        {
                            double zIndividual3 = zTotmod * MM * zFraction[g1] * zComponentsList[zPropVal[g1]];
                            zOutEP = zOutEP + zIndividual3;
                        }
                        zWriter = new StreamWriter("WAR.txt", true, Encoding.UTF8);
                        zWriter.WriteLine("                   ");
                        zWriter.WriteLine(" type:" + ProcessTypeValue.ToString());
                        zWriter.WriteLine(" name:CO" + CapeIDValue.ToString());
                        zWriter.WriteLine("  ind:" + CapePositionValue.ToString());
                        zWriter.WriteLine("value:" + zOutEP.ToString());
                        zWriter.Close();
                        //' write the mass flow results
                        zWriter = new StreamWriter("stream.txt", true, Encoding.UTF8);
                        zWriter.WriteLine("                   ");
                        zWriter.WriteLine("          type:" + ProcessTypeValue.ToString());
                        zWriter.WriteLine("          name:CO" + CapeIDValue.ToString());
                        zWriter.WriteLine("           ind:" + CapePositionValue.ToString());
                        zWriter.WriteLine("mass flow-rate:" + Convert.ToString(zTotmod * MM));
                        zWriter.Close();
                        zWriter = new StreamWriter("results.txt", true, Encoding.UTF8);
                        zWriter.WriteLine("                   ");
                        zWriter.WriteLine("          type:" + ProcessTypeValue.ToString());
                        zWriter.WriteLine("          name:CO" + CapeIDValue.ToString());
                        zWriter.WriteLine("           ind:" + CapePositionValue.ToString());
                        zWriter.WriteLine("         value:" + zOutEP.ToString());
                        zWriter.WriteLine("mass flow-rate:" + Convert.ToString(zTotmod * MM));
                        zWriter.Close();
                    }
                    else
                    {
                    }
                }
            }

            //
            //Output thermo material object
            //
            OutputPort.Temperature = zTemperature;
            OutputPort.Pressure = zPressure;
            OutputPort.Enthalpy = zEnthalpy;
            OutputPort.TotalFlow = zTotalFlow;
            OutputPort.Volume = zVolume;
            OutputPort.Fraction = zFraction;

            try
            {
                OutputPort.CalculateEquilibrium(CapeOpenFlashType.PH);
            }
            catch
            {
            }

        }
    }
}
