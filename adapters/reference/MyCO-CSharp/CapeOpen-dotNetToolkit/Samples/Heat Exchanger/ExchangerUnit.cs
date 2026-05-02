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

namespace HeatExchanger
{
    public partial class ExchangerUnit : Mose.CapeOpenToolkit.UnitProxy.CapeOpenUnitSimple, IEditableUnit
    {
        private OperationType mOperation = OperationType.ColdOutletTemp;

        public enum OperationType
        {
            HotOuttetTemp,
            ColdOutletTemp,
            Duty
        }

        protected override void OnCalculate()
        {
            double zTemperature1In = InputPort1.Temperature;
            double zPressure1In = InputPort1.Pressure;
            double zEnthalpy1In = InputPort1.Enthalpy;
            double zTotalFlow1In = InputPort1.TotalFlow;
            double[] zFraction1In = InputPort1.Fraction;

            double zTemperature2In = InputPort2.Temperature;
            double zPressure2In = InputPort2.Pressure;
            double zEnthalpy2In = InputPort2.Enthalpy;
            double zTotalFlow2In = InputPort2.TotalFlow;
            double[] zFraction2In = InputPort2.Fraction;

            double zDuty;

            switch (Operation)
            {
                case OperationType.HotOuttetTemp:

                    if (zTemperature1In > zTemperature2In)
                    {
                        OutputPort1.Pressure = zPressure1In;
                        OutputPort1.TotalFlow = zTotalFlow1In;
                        OutputPort1.Fraction = zFraction1In;
                        OutputPort1.Temperature = ParameterValue;

                        OutputPort1.CalculateEquilibrium(CapeOpenFlashType.TP);

                        zDuty = (zTotalFlow1In) * (OutputPort1.Enthalpy - zEnthalpy1In);

                        OutputPort2.Pressure = zPressure2In;
                        OutputPort2.TotalFlow = zTotalFlow2In;
                        OutputPort2.Fraction = zFraction2In;
                        OutputPort2.Enthalpy = zEnthalpy2In - (zDuty/zTotalFlow2In);

                        OutputPort2.Temperature = 400;

                        OutputPort2.CalculateEquilibrium(CapeOpenFlashType.PH);
                    }
                    else //(zTemperature1In < zTemperature2In)
                    {
                        OutputPort2.Pressure = zPressure2In;
                        OutputPort2.TotalFlow = zTotalFlow2In;
                        OutputPort2.Fraction = zFraction2In;
                        OutputPort2.Temperature = ParameterValue;

                        OutputPort2.CalculateEquilibrium(CapeOpenFlashType.TP);

                        zDuty = (zTotalFlow2In) * (OutputPort2.Enthalpy - zEnthalpy2In);

                        OutputPort1.Pressure = zPressure1In;
                        OutputPort1.TotalFlow = zTotalFlow1In;
                        OutputPort1.Fraction = zFraction1In;

                        OutputPort1.Enthalpy = zEnthalpy1In - (zDuty/ zTotalFlow1In);

                        OutputPort1.Temperature = 400;

                        OutputPort1.CalculateEquilibrium(CapeOpenFlashType.PH);
                    }
                    break;

                case OperationType.ColdOutletTemp:
                    if (zTemperature1In < zTemperature2In)
                    {
                        OutputPort1.Pressure = zPressure1In;
                        OutputPort1.TotalFlow = zTotalFlow1In;
                        OutputPort1.Fraction = zFraction1In;
                        OutputPort1.Temperature = ParameterValue;

                        OutputPort1.CalculateEquilibrium(CapeOpenFlashType.TP);

                        zDuty = (zTotalFlow1In) * (OutputPort1.Enthalpy - zEnthalpy1In);

                        OutputPort2.Pressure = zPressure2In;
                        OutputPort2.TotalFlow = zTotalFlow2In;
                        OutputPort2.Fraction = zFraction2In;
                        OutputPort2.Enthalpy = zEnthalpy2In - (zDuty/ zTotalFlow2In);

                        OutputPort2.Temperature = 400;

                        OutputPort2.CalculateEquilibrium(CapeOpenFlashType.PH);
                    }
                    else //(zTemperature1In > zTemperature2In)
                    {
                        OutputPort2.Pressure = zPressure2In;
                        OutputPort2.TotalFlow = zTotalFlow2In;
                        OutputPort2.Fraction = zFraction2In;
                        OutputPort2.Temperature = ParameterValue;

                        OutputPort2.CalculateEquilibrium(CapeOpenFlashType.TP);

                        zDuty = (zTotalFlow2In) * (OutputPort2.Enthalpy - zEnthalpy2In);

                        OutputPort1.Pressure = zPressure1In;
                        OutputPort1.TotalFlow = zTotalFlow1In;
                        OutputPort1.Fraction = zFraction1In;

                        OutputPort1.Enthalpy = zEnthalpy1In - (zDuty/ zTotalFlow1In);

                        OutputPort1.Temperature = 400;

                        OutputPort1.CalculateEquilibrium(CapeOpenFlashType.PH);
                    }
                    break;
                case OperationType.Duty:

                    zDuty = ParameterValue;

                    if (zTemperature1In > zTemperature2In)
                    {
                        OutputPort1.Pressure = zPressure1In;
                        OutputPort1.TotalFlow = zTotalFlow1In;
                        OutputPort1.Fraction = zFraction1In;
                        OutputPort1.Enthalpy = zEnthalpy1In - zDuty;
                        OutputPort1.Temperature = 100;

                        OutputPort1.CalculateEquilibrium(CapeOpenFlashType.PH);
                       
                        OutputPort2.Pressure = zPressure2In;
                        OutputPort2.TotalFlow = zTotalFlow2In;
                        OutputPort2.Fraction = zFraction2In;
                        OutputPort2.Enthalpy = zEnthalpy2In + zDuty;

                        OutputPort2.Temperature = 400;

                        OutputPort2.CalculateEquilibrium(CapeOpenFlashType.PH);
                    }
                    else //(zTemperature1In < zTemperature2In)
                    {
                        OutputPort1.Pressure = zPressure1In;
                        OutputPort1.TotalFlow = zTotalFlow1In;
                        OutputPort1.Fraction = zFraction1In;
                        OutputPort1.Enthalpy = zEnthalpy1In + zDuty;
                        OutputPort1.Temperature = 100;

                        OutputPort1.CalculateEquilibrium(CapeOpenFlashType.PH);

                        OutputPort2.Pressure = zPressure2In;
                        OutputPort2.TotalFlow = zTotalFlow2In;
                        OutputPort2.Fraction = zFraction2In;
                        OutputPort2.Enthalpy = zEnthalpy2In - zDuty;

                        OutputPort2.Temperature = 400;

                        OutputPort2.CalculateEquilibrium(CapeOpenFlashType.PH);
                    }
                    break;
            }
        }

        #region ICapeUnitEdit Membri di

        void CAPEOPEN093.ICapeUnitEdit.Edit()
        {
            ParametersForm.Edit(this);
        }

        #endregion
    }
}
