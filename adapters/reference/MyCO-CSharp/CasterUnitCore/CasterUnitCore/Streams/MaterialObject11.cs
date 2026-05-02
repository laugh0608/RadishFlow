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
using System.Runtime.InteropServices;
using System.Security.Policy;
using System.Windows;
using CAPEOPEN;
using CasterCore;

namespace CasterUnitCore
{
    /// <summary>
    /// CO1.0 material wrapper
    /// </summary>
    [Serializable]
    public class MaterialObject11 : MaterialObject
    {
        #region Interface of material

        ICapeThermoMaterial _capeThermoMaterial;
        ICapeThermoMaterialContext _capeThermoMaterialContext;
        ICapeThermoPhases _capeThermoPhases;
        ICapeThermoCompounds _capeThermoCompounds;
        ICapeThermoPropertyRoutine _capeThermoPropertyRoutine;
        ICapeThermoEquilibriumRoutine _capeThermoEquilibriumRoutine;
        ICapeThermoUniversalConstant _capeThermoUniversalConstant;

        #endregion

        #region Constructor
        /// <summary>
        /// create a MaterialObject11
        /// </summary>
        public MaterialObject11()
            : this(null)
        {
            CasterLogger.Debug("Create empty MaterialObject11");
        }
        /// <summary>
        /// create a MaterialObject11 connected to object, should only be invoked by CapeUnitPortBase
        /// </summary>
        public MaterialObject11(object objectToConnect)
        {
            CasterLogger.Debug("Create MaterialObject11 with a material");
            SetMaterial(objectToConnect);
            CasterLogger.Debug("Complete create MaterialObject11");
        }

        #endregion

        #region Material Object Manipulate

        public override object CapeThermoMaterialObject
        {
            get { return _capeThermoMaterial; }
        }

        public override void ClearAllProperties()
        {
            CasterLogger.Debug("Material 11 ClearAllProperties");
            Debug.Assert(IsValid());
            _capeThermoMaterial.ClearAllProps();
            //_alreadyFlashed = false;
        }

        public override MaterialObject Duplicate()
        {
            CasterLogger.Debug("Duplicating");
            Debug.Assert(IsValid());
            ICapeThermoMaterial newMaterial = _capeThermoMaterial.CreateMaterial();
            object sourceMaterial = _capeThermoMaterial;
            newMaterial.CopyFromMaterial(ref sourceMaterial);
            MaterialObject11 mo = new MaterialObject11();
            mo.SetMaterial(newMaterial);
            CasterLogger.Debug("Duplicated");
            return mo;
        }

        public override bool IsValid()
        {
            return _capeThermoMaterial != null;
        }

        public override bool SetMaterial(object material)
        {
            CasterLogger.Debug("SetMaterial");
            if (material == null) return false;
            if (material is ICapeThermoMaterial)
            {
                //_alreadyFlashed = true;
                MaterialObjectVersion = 11;
                _capeThermoMaterial = material as ICapeThermoMaterial;
                _capeThermoMaterialContext = material as ICapeThermoMaterialContext;
                _capeThermoPhases = material as ICapeThermoPhases;
                _capeThermoCompounds = material as ICapeThermoCompounds;
                _capeThermoPropertyRoutine = material as ICapeThermoPropertyRoutine;
                _capeThermoEquilibriumRoutine = material as ICapeThermoEquilibriumRoutine;
                _capeThermoUniversalConstant = material as ICapeThermoUniversalConstant;
            }
            else if (material is MaterialObject11)
            {
                SetMaterial(((MaterialObject11)material).CapeThermoMaterialObject);
            }
            else
                throw new ArgumentException("parameter is not a CO1.1 material object");

            GetListOfAllowedPhase(out string[] _, out string _2);
            UpdateCompoundList();
            CasterLogger.Debug("SetMaterial completed");
            return true;
        }

        protected override void Dispose(bool disposing)
        {
            CasterLogger.Debug("Dispose material10.");
            if (_capeThermoMaterial != null && _capeThermoMaterial.GetType().IsCOMObject)
                Marshal.ReleaseComObject(_capeThermoMaterial);
            if (_capeThermoCompounds != null && _capeThermoCompounds.GetType().IsCOMObject)
                Marshal.ReleaseComObject(_capeThermoCompounds);
            if (_capeThermoEquilibriumRoutine != null && _capeThermoEquilibriumRoutine.GetType().IsCOMObject)
                Marshal.ReleaseComObject(_capeThermoEquilibriumRoutine);
            if (_capeThermoMaterialContext != null && _capeThermoMaterialContext.GetType().IsCOMObject)
                Marshal.ReleaseComObject(_capeThermoMaterialContext);
            if (_capeThermoPhases != null && _capeThermoPhases.GetType().IsCOMObject)
                Marshal.ReleaseComObject(_capeThermoPhases);
            if (_capeThermoPropertyRoutine != null && _capeThermoPropertyRoutine.GetType().IsCOMObject)
                Marshal.ReleaseComObject(_capeThermoPropertyRoutine);
            if (_capeThermoUniversalConstant != null && _capeThermoUniversalConstant.GetType().IsCOMObject)
                Marshal.ReleaseComObject(_capeThermoUniversalConstant);
            _capeThermoMaterial = null;
            _capeThermoMaterialContext = null;
            _capeThermoPhases = null;
            _capeThermoCompounds = null;
            _capeThermoPropertyRoutine = null;
            _capeThermoEquilibriumRoutine = null;
            _capeThermoUniversalConstant = null;
            CasterLogger.Debug("Dispose material10 completed.");
        }

        #endregion

        #region DoFlash

        protected override bool CheckEquilibriumSpec(string[] flashSpec1, string[] flashSpec2, string solutionType)
        {
            CasterLogger.Debug($"CheckEquilibriumSpec for {flashSpec1} and {flashSpec2}");
            return _capeThermoEquilibriumRoutine.CheckEquilibriumSpec(flashSpec1, flashSpec2, solutionType);
        }

        public override bool DoFlash(string[] flashSpec1, string[] flashSpec2, string solutionType, bool showWarning = false)
        {
            CasterLogger.Debug($"DoFlash for {flashSpec1} and {flashSpec2} in Solution type: {solutionType}");
            try
            {
                PresentPhases = AllowedPhases;
                _capeThermoEquilibriumRoutine.CalcEquilibrium(flashSpec1, flashSpec2, solutionType);
                //_alreadyFlashed = true;
            }
            catch (Exception e)
            {
                if (showWarning)
                    MessageBox.Show("Flash failed. " + e.Message);
                CasterLogger.ErrorFormatted("Flash failed. {0}", e.Message);
                Debug.WriteLine("Flash failed. {0}", e.Message);
                return false;
            }
            CasterLogger.Debug($"Finish DoFlash");
            return true;
        }

        public override bool DoTPFlash(bool showWarning = false)
        {
            return DoFlash(
                new[] { "temperature", "", "Overall" },
                new[] { "pressure", "", "Overall" },
                "unspecified",
                showWarning
                );
        }

        public override bool DoPHFlash(bool showWarning = false)
        {
            return DoFlash(
                new[] { "enthalpy", "", "Overall" },
                new[] { "pressure", "", "Overall" },
                "unspecified",
                showWarning
                );
        }

        public override bool DoTHFlash(bool showWarning = false)
        {
            return DoFlash(
               new[] { "temperature", null, "Overall" },
               new[] { "enthalpy", null, "Overall" },
               "unspecified",
               showWarning
               );
        }

        public override bool DoTVFFlash(bool showWarning = false)
        {
            return DoFlash(
                new[] { "temperature", null, "Overall" },
                new[] { "phaseFraction", "Mole", "gas" },
                "unspecified",
                showWarning
                );
        }

        public override bool DoPVFlash(bool showWarning = false)
        {
            return DoFlash(
                new[] { "pressure", null, "Overall" },
                new[] { "phaseFraction", "Mole", "gas" },
                "unspecified",
                showWarning
                );
        }

        #endregion

        #region Phase

        public override Phases[] GetListOfAllowedPhase(out string[] phaseAggregationList, out string keyCompoundId)
        {
            CasterLogger.Debug("GetListOfAllowedPhase");
            object allowedPhaseObject = null;
            object phaseAggregationObject = null;
            object keyCompoundIdObject = null;
            phaseAggregationList = null;
            keyCompoundId = null;
            try
            {
                _capeThermoPhases.GetPhaseList(ref allowedPhaseObject, ref phaseAggregationObject, ref keyCompoundIdObject);
            }
            catch (Exception e)
            {
                CasterLogger.ErrorFormatted("Cannot get AllowedPhase. {0}", e.Message);
                Debug.WriteLine("Cannot get AllowedPhase. {0}", e.Message);
            }
            if (allowedPhaseObject == null)
                return new[] { Phases.Vapor, Phases.Liquid };

            phaseAggregationList = phaseAggregationObject as string[];
            keyCompoundId = keyCompoundIdObject as string;

            string[] phaseStringList = allowedPhaseObject as string[];
            Phases[] phaseList = (from phaseString in phaseStringList
                                  select new Phases(phaseString)).ToArray();

            //Set Proper Phase name
            var phases = string.Join(", ", phaseList.Select(p=>p.Value).ToArray());
            CasterLogger.Debug("Material allowd phases: " + phases);
            if (phaseList.FirstOrDefault(phase => phase.Value.Contains("vap")) != null)
                Phases.Vapor = phaseList.First(phase => phase.Value.Contains("vap"));
            if (phaseList.FirstOrDefault(phase => phase.Value.Contains("liq")) != null)
                Phases.Liquid = phaseList.First(phase => phase.Value.Contains("liq"));
            if (phaseList.FirstOrDefault(phase => phase.Value.Contains("solid")) != null)
                Phases.Liquid = phaseList.First(phase => phase.Value.Contains("solid"));

            return phaseList;
        }

        public override Phases[] GetListOfPresentPhases(out eCapePhaseStatus[] presentPhaseStatus)
        {
            CasterLogger.Debug("GetListOfPresentPhases");
            object phaseLabel = null;
            object phaseStatus = null;
            presentPhaseStatus = null;
            if (_capeThermoMaterial == null) return null;
            _capeThermoMaterial.GetPresentPhases(ref phaseLabel, ref phaseStatus);
            string[] status = phaseStatus as string[];
            if (status != null)
            {
                presentPhaseStatus = new eCapePhaseStatus[status.Length];
                for (int i = 0; i < status.Length; i++)
                {
                    eCapePhaseStatus s;
                    if (Enum.TryParse(status[i], out s))
                        presentPhaseStatus[i] = s;
                }
            }

            string[] phaseStringList = phaseLabel as string[];
            Phases[] phaseList = (from phaseString in phaseStringList select new Phases(phaseString)).ToArray();
            var phases = string.Join(", ", phaseList.Select(p => p.Value).ToArray());
            CasterLogger.Debug("Material present phases: " + phases);
            return phaseList;
        }
        public override void SetListOfPresentPhases(IEnumerable<Phases> presentPhases, IEnumerable<eCapePhaseStatus> presentPhasesStatus)
        {
            CasterLogger.Debug("SetListOfPresentPhases: " + string.Join(", ", presentPhases.Select(p=>p.Value).ToArray()));
            if (_capeThermoMaterial == null) return;
            int[] phaseStatus = (from status in presentPhasesStatus select (int)status).ToArray();
            string[] phaseStringList = (from phase in presentPhases select phase.Value).ToArray();
            _capeThermoMaterial.SetPresentPhases(phaseStringList, phaseStatus);
            //_alreadyFlashed = false;
        }

        #endregion

        #region CompoundId

        public override string[] Formulas
        {
            get
            {
                object compoundId = null, formulaId = null, names = null, boilTemps = null, molwts = null, casnos = null;
                _capeThermoCompounds.GetCompoundList(ref compoundId, ref formulaId, ref names, ref boilTemps, ref molwts, ref casnos);
                return formulaId as string[];
            }
        }

        public override string[] UpdateCompoundList()
        {
            CasterLogger.Debug("UpdateCompoundList");
            object compoundId = null, formulaId = null, names = null, boilTemps = null, molwts = null, casnos = null;
            try
            {
                _capeThermoCompounds.GetCompoundList(ref compoundId, ref formulaId,
                    ref names, ref boilTemps, ref molwts, ref casnos);
            }
            catch (Exception e)
            {
                CasterLogger.ErrorFormatted("Unable to get compound list. Make sure to call UpdateComoundList after compound list changed. {0}", e.Message);
                Debug.WriteLine("Unable to get compound list. Make sure to call UpdateComoundList after compound list changed. {0}", e.Message);
            }
            CasterLogger.Debug("Compound List: " + aliasName);
            return aliasName = compoundId as string[];
        }

        #endregion

        #region Overall Property

        public override double[] GetOverallPropList(string propName, PropertyBasis basis, bool calculate = false)
        {
            CasterLogger.Debug($"Get property {propName} for overall");
            object value = null;
            _capeThermoMaterial.GetOverallProp(propName, basis.ToString(), ref value);
            CasterLogger.Debug($"Property {propName} result: {value}");
            return value as double[];
        }

        public override void SetOverallPropList(string propName, PropertyBasis basis, IEnumerable<double> value)
        {
            CasterLogger.Debug($"Set property {propName} for overall: {value}");
            double[] temp = value as double[] ?? value.ToArray();
            _capeThermoMaterial.SetOverallProp(propName, basis.ToString(), value);
            //_alreadyFlashed = false;
            CasterLogger.Debug($"Set property complete");
        }

        #endregion

        #region Single Phase Property

        public override string[] AvailableSinglePhaseProp
        {
            get { return _capeThermoPropertyRoutine.GetSinglePhasePropList() as string[]; }
        }

        public override double[] GetSinglePhasePropList(string propName, Phases phase, PropertyBasis basis, bool calculate = true)
        {
            CasterLogger.Debug($"Get property {propName} for phase {phase}");
            object value = null;
            if (PresentPhases.All(p => p.Value != phase.Value))
                return new double[CompoundNum];     //default is 0 for every element
            try
            {
                if (calculate)
                    _capeThermoPropertyRoutine.CalcSinglePhaseProp(new[] { propName }, phase.Value);
            }
            catch (Exception e)
            {
                CasterLogger.ErrorFormatted("Calculate single phase prop {0} fails. {1}", propName, e.Message);
                Debug.WriteLine("Calculate single phase prop {0} fails. {1}", propName, e.Message);
            }
            _capeThermoMaterial.GetSinglePhaseProp(propName, phase.Value, basis.ToString(), ref value);
            CasterLogger.Debug($"Property {propName} result: {value}");
            return value as double[];
        }

        public override void SetSinglePhasePropList(string propName, Phases phase, PropertyBasis basis, IEnumerable<double> value)
        {
            CasterLogger.Debug($"Set property {propName} for phase {phase}: {value}");
            if (PresentPhases.All(p => p.Value != phase.Value))
                PresentPhases = AllowedPhases;
            double[] temp = value as double[] ?? value.ToArray();
            _capeThermoMaterial.SetSinglePhaseProp(propName, phase.Value, basis.ToString(), temp);
            //_alreadyFlashed = false;
            CasterLogger.Debug($"Set property complete");
        }

        #endregion

        #region Two Phase Property

        public override string[] AvailableTwoPhaseProp
        {
            get { return _capeThermoPropertyRoutine.GetTwoPhasePropList(); }
        }

        public override double[] GetTwoPhasePropList(string propName, Phases phase1, Phases phase2, PropertyBasis basis, bool calculate = true)
        {
            CasterLogger.Debug($"Get property {propName} for phase {phase1} and {phase2}");

            object value = null;

            if (PresentPhases.All(p => p.Value != phase1.Value)
                || PresentPhases.All(p => p.Value != phase2.Value))
                return new double[CompoundNum];

            string[] phaseList = { phase1.Value, phase2.Value };
            try
            {
                _capeThermoPropertyRoutine.CalcTwoPhaseProp(new[] { propName }, phaseList);
            }
            catch (Exception e)
            {
                Debug.WriteLine("Calculate two phase prop {0} fails. {1}", propName, e.Message);
            }
            _capeThermoMaterial.GetTwoPhaseProp(propName, phaseList, basis.ToString(), ref value);
            CasterLogger.Debug($"Property {propName} result: {value}");
            return value as double[];
        }

        public override void SetTwoPhasePropList(string propName, Phases phase1, Phases phase2, PropertyBasis basis, IEnumerable<double> value)
        {
            CasterLogger.Debug($"Set property {propName} for phase {phase1} and {phase2}: {value}");
            if (PresentPhases.All(p => p.Value != phase1.Value)
                || PresentPhases.All(p => p.Value != phase2.Value))
                PresentPhases = AllowedPhases;

            string[] phaseList = { phase1.Value, phase2.Value };
            double[] temp = value as double[] ?? value.ToArray();
            _capeThermoMaterial.SetTwoPhaseProp(propName, phaseList, basis.ToString(), temp);
            //_alreadyFlashed = false;
            CasterLogger.Debug($"Set property complete");
        }

        #endregion

        #region Constant Property & T-Dependent Property & P-Dependent Property

        public override string[] AvailableConstProp
        {
            get { return _capeThermoCompounds.GetConstPropList() as string[]; }
        }

        public override string[] AvailableTDependentProp
        {
            get { return _capeThermoCompounds.GetTDependentPropList() as string[]; }
        }

        public override string[] AvailablePDependentProp
        {
            get { return _capeThermoCompounds.GetPDependentPropList() as string[]; }
        }

        public override string[] AvailableUniversalConstProp
        {
            get { return _capeThermoUniversalConstant.GetUniversalConstantList() as string[]; }
        }

        public override double GetCompoundConstPropDouble(string propName, string compoundId)
        {
            CasterLogger.Debug($"GetCompoundConstPropDouble compound {compoundId} for {propName}");
            object[] value = _capeThermoCompounds.GetCompoundConstant(new[] { propName }, new[] { compoundId });
            CasterLogger.Debug("GetCompoundConstPropDouble result " + value);
            try
            {
                return Convert.ToDouble(value.SingleOrDefault());
            }
            catch (Exception) { }
            try
            {
                return (value.Single() as double[]).SingleOrDefault();
            }
            catch (Exception e)
            {
                CasterLogger.ErrorFormatted("Get compound constant prop fails. {0}", e.Message);
                Debug.WriteLine("Get compound constant prop fails. {0}", e.Message);
            }
            return 0;
        }

        public override double GetCompoundTDependentProp(string propName, string compoundId, double T)
        {
            CasterLogger.Debug($"GetCompoundTDependentProp compound {compoundId} for {propName} at temperature {T}");
            object value = null;
            _capeThermoCompounds.GetTDependentProperty(new[] { propName }, T, new[] { compoundId }, ref value);
            return (value as double[]).SingleOrDefault();
        }

        public override double GetCompoundPDependentProp(string propName, string compoundId, double P)
        {
            CasterLogger.Debug($"GetCompoundPDependentProp compound {compoundId} for {propName} at pressure {P}");
            object value = null;
            _capeThermoCompounds.GetPDependentProperty(new[] { propName }, P, new[] { compoundId }, ref value);
            return (value as double[]).SingleOrDefault();
        }

        public override double GetUniversalConstProp(string constantId)
        {
            CasterLogger.Debug($"GetUniversalConstProp for {constantId}");
            object temp = _capeThermoUniversalConstant.GetUniversalConstant(constantId);
            return Convert.ToDouble(temp);
        }

        #endregion
    }
}
