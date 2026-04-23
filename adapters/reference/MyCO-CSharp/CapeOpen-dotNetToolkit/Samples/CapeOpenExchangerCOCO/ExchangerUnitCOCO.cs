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

using System;
using System.Collections.Generic;
using System.Text;
using Mose.CapeOpenToolkit.UnitProxy;

namespace CapeOpenExchangerCOCO
{
    public partial class ExchangerUnitCOCO : Mose.CapeOpenToolkit.UnitProxy.CapeOpenUnitSimple 
    {

        protected override void OnCalculate()
        {
            LogClass.WriteLog("Entro");
            double zTemperature1In = InputPort1.Temperature;
            double zPressure1In = InputPort1.Pressure;
            double zEnthalpy1In = InputPort1.Enthalpy;
            LogClass.WriteLog("Dopo Enth");
            double zTotalFlow1In = InputPort1.TotalFlow;
            double[] zFraction1In = InputPort1.Fraction;

            LogClass.WriteLog("Prima di porta 2");

            double zTemperature2In = InputPort2.Temperature;
            double zPressure2In = InputPort2.Pressure;
            double zEnthalpy2In = InputPort2.Enthalpy;
            double zTotalFlow2In = InputPort2.TotalFlow;
            double[] zFraction2In = InputPort2.Fraction;

            //double zTemperature1Out;
            //double zPressure1Out;
            //double zEnthalpy1Out; 
            //double zTotalFlow1Out; 
            //double[] zFraction1Out;

            //double zTemperature2Out;
            //double zPressure2Out;
            //double zEnthalpy2Out;
            //double zTotalFlow2Out;
            //double[] zFraction2Out;

            double zDuty;

            LogClass.WriteLog("Temperatura T1= " + zTemperature1In.ToString() + " Temperatura T2= " + zTemperature2In.ToString());
            
            if (zTemperature1In >= zTemperature2In)
            {
                OutputPort1.Pressure= zPressure1In;
                OutputPort1.TotalFlow= zTotalFlow1In;
                OutputPort1.Fraction = zFraction1In;
                OutputPort1.Temperature = 250;
                //OutputPort1.Temperature = InputPort1.Temperature;

                LogClass.WriteLog("Prima di calc equilibrium porta 1 x");
                OutputPort1.CalculateEquilibrium(CapeOpenFlashType.TP);
                LogClass.WriteLog("DOPO di calc equilibrium porta 1");

                LogClass.WriteLog("Enthalpy IN1 = " + zEnthalpy1In.ToString() + "Enthalpy OUT1 = " + OutputPort1.Enthalpy.ToString());

                zDuty = zTotalFlow1In * (OutputPort1.Enthalpy - zEnthalpy1In);

                OutputPort2.Pressure = zPressure2In;
                OutputPort2.TotalFlow = zTotalFlow2In;
                OutputPort2.Fraction = zFraction2In;
                LogClass.WriteLog("Prima di set Enth");
                OutputPort2.Enthalpy = zEnthalpy2In + zDuty;
                LogClass.WriteLog("DOPO di set Enth");
                //OutputPort2.Enthalpy = 1199.8816;
                //OutputPort2.Temperature = 300;

                //OutputPort2.Temperature = InputPort2.Temperature;
                //OutputPort2.CalculateProperty(CapeOpenThermoMaterialPropertyType.Volume, CapeOpenPhaseType.Overall);
                //LogClass.WriteLog("Prima di calc equilibrium porta 2; Enthalpy IN2 = " + zEnthalpy2In.ToString() + " Duty = " + zDuty.ToString() + " EnthaplyOut2 = " + OutputPort2.Enthalpy);
                
                //OutputPort2.CalculateProperty(CapeOpenThermoMaterialPropertyType.Temperature, CapeOpenPhaseType.Overall, CapeOpenCalculationType.Nothing);

                //LogClass.WriteLog("Temperatura = " + OutputPort2.Temperature.ToString());


                //string[] zProps = new string[1];
                //zProps[0] = "Temperature";

                //OutputPort2.CalculateEquilibrium(CapeOpenFlashType.PH,zProps);

                OutputPort2.CalculateEquilibrium(CapeOpenFlashType.PH);

                //LogClass.WriteLog("Temperatura = " + OutputPort2.Temperature.ToString() + " EnthaplyOut2 = " + OutputPort2.Enthalpy);

                //OutputPort2.Enthalpy = 1199.8816;

                LogClass.WriteLog("Dopo di calc equilibrium porta 2");

            }


        }
    }
}