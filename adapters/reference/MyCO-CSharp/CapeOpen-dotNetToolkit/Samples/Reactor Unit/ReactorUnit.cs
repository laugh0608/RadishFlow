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

namespace CapeOpenReactor {
    using System;
    using System.Collections.Generic;
    using System.Text;
    using Mose.CapeOpenToolkit.UnitProxy;
    using System.Runtime.InteropServices;
    
    
    public partial class ReactorUnit : Mose.CapeOpenToolkit.UnitProxy.CapeOpenUnitSimple {
        
        protected override void OnCalculate() 
        {
            //
            //Check Components Number and their IDS
            //
            int zComponentNumber;
            zComponentNumber = InputPort.ComponentsCount;
            string[] zIds;
            zIds = InputPort.Ids;

            //
            //Input Port properties
            //
            double zTemperature = InputPort.Temperature;
            double zPressure = InputPort.Pressure;
            double zTotalFlow = InputPort.TotalFlow;

            //Volume is to Calculate...
            InputPort.CalculateProperty(CapeOpenThermoMaterialPropertyType.Volume, CapeOpenPhaseType.Overall, CapeOpenCalculationType.Pure);
            double zVolume = InputPort.Volume;
            double zEnthalpy = InputPort.Enthalpy;
            double[] zFraction = InputPort.Fraction;

            //
            //Units of Measure changing
            //
            double dTotmod = zTotalFlow * 3.6;
            double dTempmod = zTemperature - 273.15;
            double dPresmod = zPressure / 100000.0;

            //Calc. Partial Pressure
            double[] partial_pressure = new double[zComponentNumber];
            for (int i = 0; i < zComponentNumber; i++)
            {
                partial_pressure[i] = zFraction[i] * zPressure;
            }

            //Reaction Speed
            double R = 1.98578;

            double r1;
            double r2;
            double r3;

            //Reaction Temperature
            double Treaction = zTemperature + 119.0;

            r1 = K1Value * Math.Exp(-E1Value / (R * Treaction)) * (partial_pressure[0] / 1000.0) * (partial_pressure[2] / 1000.0);
            r2 = K2Value * Math.Exp(-E2Value / (R * Treaction)) * (partial_pressure[0] / 1000.0) * (partial_pressure[2] / 1000.0);
            r3 = K3Value * Math.Exp(-E3Value / (R * Treaction)) * (partial_pressure[0] / 1000.0) * (partial_pressure[2] / 1000.0);


            //display the output pressure
            double pressureout = zPressure - 80000.0;
            //define the reactor volume
            double vol = 101.78;

            double propene1;
            double propene2;
            double propene3;
            double k1tot;
            double k2tot;
            double k3tot;

            k1tot = K1Value * Math.Exp(-E1Value / (R * Treaction));
            k2tot = K2Value * Math.Exp(-E2Value / (R * Treaction));
            k3tot = K3Value * Math.Exp(-E3Value / (R * Treaction));

            propene1 = 0.00199 * vol * r1;
            propene2 = 0.00199 * vol * r2;
            propene3 = 0.00199 * vol * r3;

            double propene_final = zFraction[0] * zTotalFlow * 3.6 - (propene1 + propene2 + propene3);
            double water_final = zFraction[1] * zTotalFlow * 3.6 + (propene1 + propene2 + 3.0 * propene3);
            double oxygen_final = zFraction[2] * zTotalFlow * 3.6 - (1.5 * propene1 + 2.5 * propene2 + 4.5 * propene3);
            double nitrogen_final = zFraction[3] * zTotalFlow * 3.6;
            double acetic_final = zFraction[4] * zTotalFlow * 3.6 + propene2;
            double acrylic_final = zFraction[5] * zTotalFlow * 3.6 + propene1;
            double CO2_final = zFraction[6] * zTotalFlow * 3.6 + (propene2 + 3.0 * propene3);

            //Calculate outputstream
            double sum = propene_final + water_final + oxygen_final + nitrogen_final + acetic_final + acrylic_final + CO2_final;

            //mole fractions for output stream 
            double fr_propene_final = propene_final / sum;
            double fr_water_final = water_final / sum;
            double fr_oxygen_final = oxygen_final / sum;
            double fr_nitrogen_final = nitrogen_final / sum;
            double fr_acetic_final = acetic_final / sum;
            double fr_acrylic_final = acrylic_final / sum;
            double fr_CO2_final = CO2_final / sum;

            double[] fractionout = new double[8];
            fractionout[0] = fr_propene_final;
            fractionout[1] = fr_water_final;
            fractionout[2] = fr_oxygen_final;
            fractionout[3] = fr_nitrogen_final;
            fractionout[4] = fr_acetic_final;
            fractionout[5] = fr_acrylic_final;
            fractionout[6] = fr_CO2_final;

            //Define Enthalpy
            double deltaH = -3590030.0;
            double enthalpyOut = zEnthalpy - deltaH;

            //
            //Output Thermo Material Object
            //

            double suma = sum / 3.6;

            OutputPort.Temperature = Treaction;
            OutputPort.Pressure = pressureout;
            OutputPort.TotalFlow = suma;
            OutputPort.Enthalpy = enthalpyOut;
            OutputPort.Volume = zVolume;
            OutputPort.Fraction = fractionout;

            OutputPort.CalculateEquilibrium(CapeOpenFlashType.TP);
        }
    }
}
