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
using CAPEOPEN093;

namespace Mose.CapeOpenToolkit.UnitProxy
{
    public class CapeOpenThermoMaterialObject : CapeOpenObjectBase
    {
        private ICapeThermoMaterialObject mConnectedObject;

        public static CapeOpenThermoMaterialObject Connect(ICapeThermoMaterialObject objectToConnect)
        {
            CapeOpenThermoMaterialObject zObject = new CapeOpenThermoMaterialObject();
            zObject.mConnectedObject = objectToConnect;
            return zObject;
        }

        private CapeOpenThermoMaterialObject()
        {
        }

        public ICapeThermoMaterialObject ConnectedObject
        {
            get
            {
                return mConnectedObject;
            }
        }

        public double Temperature
        {
            get
            {
                return ((double[])mConnectedObject.GetProp("temperature", null, null, null, null))[0];
            }

            set
            {
                double[] zTemperature = new double[1];
                zTemperature[0] = value;
                mConnectedObject.SetProp("temperature", null, null, null, null, zTemperature);
            }
        }

        public double Pressure
        {
            get
            {
                return ((double[])mConnectedObject.GetProp("pressure", null, null, null, null))[0];
            }

            set
            {
                double[] zPressure = new double[1];
                zPressure[0] = value;
                mConnectedObject.SetProp("pressure", null, null, null, null, zPressure);
            }
        }

        public double TotalFlow
        {
            get
            {
                return ((double[])mConnectedObject.GetProp("totalFlow", "Overall", null, null, "mole"))[0];
            }

            set
            {
                double[] zTotalFlow = new double[1];
                zTotalFlow[0] = value;
                mConnectedObject.SetProp("totalFlow", "Overall", null, null, "mole", zTotalFlow);
            }
        }

        public double Enthalpy
        {
            get
            {
                return ((double[])mConnectedObject.GetProp("enthalpy", "Overall", null, "Mixture", "mole"))[0];
            }

            set
            {
                double[] zEnthalpy = new double[1];
                zEnthalpy[0] = value;
                mConnectedObject.SetProp("enthalpy", "Overall", null, "Mixture", "mole", zEnthalpy);
            }
        }

        public double Volume
        {
            get
            {
                return ((double[])mConnectedObject.GetProp("volume","Overall",null,"Mixture","mole"))[0];
            }
            set
            {
                mConnectedObject.SetProp("volume", "Overall", null, "Mixture", null, value);
            }

        }

        public double[] Fraction
        {
            get
            {
                return ((double[])mConnectedObject.GetProp("Fraction", "Overall", mConnectedObject.ComponentIds, null, "mole"));
            }

            set
            {
                mConnectedObject.SetProp("Fraction", "Overall", null, "", "mole", value);
            }
        }

        public void CalculateEquilibrium(CapeOpenFlashType flashType)
        {
            mConnectedObject.CalcEquilibrium(CapeOpenFlashTypeHelper.Map(flashType), null);
        }

        public void CalculateEquilibrium(CapeOpenFlashType flashType, string[] properties)
        {
            mConnectedObject.CalcEquilibrium(CapeOpenFlashTypeHelper.Map(flashType), properties);
        }

        public int ComponentsCount
        {
            get
            {
                return mConnectedObject.GetNumComponents();
            }
        }

        public string[] Ids
        {
            get
            {
                return (string[]) mConnectedObject.ComponentIds;
            }
        }

        public void CalculateProperty(CapeOpenThermoMaterialPropertyType property, CapeOpenPhaseType phase, CapeOpenCalculationType calculationType) 
        {
            string[] zType = new string[1];
            zType[0] = CapeOpenThermoMaterialPropertyTypeHelper.Map(property);
            string[] zPhase = new string[1];
            zPhase[0] = CapeOpenPhaseTypeHelper.Map(phase);
            string zCalculationType = CapeOpenCalculationTypeHelper.Map(calculationType);
            //mConnectedObject.CalcProp(CapeOpenThermoMaterialPropertyTypeHelper.Map(property), CapeOpenPhaseTypeHelper.Map(phase), "Mixture");
            if (calculationType != CapeOpenCalculationType.Nothing)
            {
                mConnectedObject.CalcProp(zType, zPhase, zCalculationType);
            }
            else
            {
                mConnectedObject.CalcProp(zType, zPhase, null);
            }

        }
    }
}
