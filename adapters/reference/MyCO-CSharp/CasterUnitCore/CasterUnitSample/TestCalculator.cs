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

using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Forms;
using CasterUnitCore;
using CAPEOPEN;
using CasterCore;

namespace CasterUnitSample
{
    public class TestCalculator : Calculator
    {
        #region Parameters & Materials
        //materials
        public MaterialObject inputMaterial;
        public MaterialObject outputMaterial;
        public CapeEnergyPort collections;
        //output Parameters, results
        public double ParamT;
        public double ParamP;
        #endregion

        #region Overrides of SpecCalculator

        public override void BeforeCalculate()
        {
            CasterLogger.Debug("BeforeCalculate");
            //Get material connected to this unit
            inputMaterial = ((CapeMaterialPort)UnitOp.Ports["feed"]).Material.Duplicate();
            outputMaterial = inputMaterial.Duplicate();
            //collections = (CapeEnergyPort) UnitOp.Ports["energy"];
            //double b = collections.Work;
            //Initialize the output Parameters
            //Warning!! For Parameters, do not use ParamTout=ParamT, use the value property
            ParamT = ((CapeRealParameter)UnitOp.Parameters["T"]).SIValue;
            ParamP = ((CapeRealParameter)UnitOp.Parameters["P"]).SIValue;
            CasterLogger.Debug($"Calculation Parameters: T {ParamT} P {ParamP}");
        }

        public override void Calculate()
        {
            CasterLogger.Debug("Calculate");
            //throw new ECapeUnknownException(UnitOp,"Test");

            #region Flash
            outputMaterial.T = ParamT;
            outputMaterial.P = ParamP;
            Debug.Assert(outputMaterial.DoTPFlash());
            outputMaterial.GetSinglePhasePropDouble("enthalpy", Phases.Vapor, PropertyBasis.Mole);
            outputMaterial.Enthalpy = inputMaterial.Enthalpy;
            Debug.Assert(outputMaterial.DoPHFlash());
            #endregion

            //#region Overall

            //double Tcurr = outputMaterial.T;
            //double Pcurr = outputMaterial.P;
            //var composition = outputMaterial.Composition;
            //double enthalpy = outputMaterial.Enthalpy;
            //double volume = outputMaterial.VolumeFlow;
            ////double gibbsEnergy = 0;
            ////if (outputMaterial.AvailableSinglePhaseProp.Contains("gibbsEnergy"))
            ////    gibbsEnergy = outputMaterial.GibbsEnergy;
            ////double entropy = 0;
            ////if (outputMaterial.AvailableSinglePhaseProp.Contains("entropy"))
            ////    entropy = outputMaterial.Entropy;

            //#endregion

            //#region SinglePhase

            //var fugacity = outputMaterial.GetSinglePhasePropList("logFugacityCoefficient", Phases.Vapor, PropertyBasis.Undefined);
            //var activity = outputMaterial.GetSinglePhasePropList("activityCoefficient", Phases.Liquid, PropertyBasis.Undefined);

            //#endregion

            //#region TwoPhase

            //outputMaterial.VaporFraction = 1;
            //Debug.Assert(outputMaterial.DoPVFlash());  //COFE fails
            //var kvalue = outputMaterial.KValue;

            //#endregion

            //#region Constant

            //var R = outputMaterial.GetUniversalConstProp("molarGasConstant");
            //var molecularWeight = outputMaterial.GetCompoundConstPropDouble("molecularWeight",
            //    outputMaterial.Compounds[0]);
            //var heatCapacity = outputMaterial.GetCompoundTDependentProp("heatOfVaporization",
            //    outputMaterial.Compounds[0], ParamT);

            //#endregion

            //#region Enthalpy

            //double[] idealGasEnthalpy = new double[3];
            //for (int i = 0; i < outputMaterial.CompoundNum; i++)
            //    idealGasEnthalpy[i] = outputMaterial.GetCompoundTDependentProp("idealGasEnthalpy",
            //        outputMaterial.Compounds[i], outputMaterial.T);
            //double excessH = outputMaterial.GetSinglePhasePropDouble("mixtureEnthalpyDifference", Phases.Vapor, PropertyBasis.Mole);

            //#endregion

            CasterLogger.Debug("Calculate complete");
            return;
        }

        public override void OutputResult()
        {
            CasterLogger.Debug("OutputResult");
            ((CapeRealParameter)UnitOp.Results["Tout"]).value = outputMaterial.T;
            ((CapeRealParameter)UnitOp.Results["Pout"]).value = outputMaterial.P;

            ((CapeMaterialPort)UnitOp.Ports["product"]).Material
                .SetOverallTPFlowCompositionAndFlash(
                outputMaterial.T,
                outputMaterial.P,
                outputMaterial.TotalFlow,
                outputMaterial.Composition);

            //Clear Reference
            inputMaterial.Destroy();
            outputMaterial.Destroy();
            CasterLogger.Debug("OutputResult completed");
        }

        #endregion
    }
}
