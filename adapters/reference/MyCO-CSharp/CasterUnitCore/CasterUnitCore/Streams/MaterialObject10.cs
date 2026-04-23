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
using System.Runtime.InteropServices;
using System.Windows;
using CAPEOPEN;
using CasterCore;

namespace CasterUnitCore
{
    /// <summary>
    /// CO1.0 material wrapper
    /// </summary>
    [Serializable]
    public class MaterialObject10 : MaterialObject
    {
        #region Interface of Material

        private ICapeThermoMaterialObject _capeThermoMaterialObject;
        //private ICapeThermoPropertyPackage _capeThermoPropertyPackage;

        #endregion

        #region Constructor

        /// <summary>
        /// create a MaterialObject10
        /// </summary>
        public MaterialObject10()
            : this(null)
        {
            CasterLogger.Debug("Create empty MaterialObject10");
        }

        /// <summary>
        /// create a MaterialObject10 connected to object, should only be invoked by CapeUnitPortBase
        /// </summary>
        public MaterialObject10(object objectToConnect)
        {
            CasterLogger.Debug("Create MaterialObject10 with a material");
            SetMaterial(objectToConnect);
            CasterLogger.Debug("Complete create MaterialObject10");
        }

        #endregion

        #region Material Object Manipulate

        public override object CapeThermoMaterialObject
        {
            get { return _capeThermoMaterialObject; }
        }

        public override void ClearAllProperties()
        {
            CasterLogger.Debug("ClearAllProperties");
            Debug.Assert(IsValid());
            _capeThermoMaterialObject.RemoveResults("");
        }

        public override MaterialObject Duplicate()
        {
            CasterLogger.Debug("Duplicating");
            Debug.Assert(IsValid());
            if (_capeThermoMaterialObject == null) return null;
            MaterialObject newMaterial = new MaterialObject10();
            newMaterial.SetMaterial(_capeThermoMaterialObject.Duplicate());
            CasterLogger.Debug("Duplicated");
            return newMaterial;
        }

        public override bool IsValid()
        {
            return _capeThermoMaterialObject != null;
        }

        public override bool SetMaterial(object materialObject)
        {
            CasterLogger.Debug("SetMaterial");
            if (materialObject == null) return false;

            if (materialObject is ICapeThermoMaterialObject)
            {
                //_alreadyFlashed = true;
                MaterialObjectVersion = 10;
                _capeThermoMaterialObject = materialObject as ICapeThermoMaterialObject;
                //_capeThermoPropertyPackage = materialObject as ICapeThermoPropertyPackage;
                //if (_capeThermoPropertyPackage == null)
                //    Debug.WriteLine("Aspen dont support CapeThermoPropertyPackage interface!");
            }
            else if (materialObject is MaterialObject10)
            {
                SetMaterial(((MaterialObject10)materialObject).CapeThermoMaterialObject);
            }
            else
                throw new ArgumentException("parameter is not a CO1.0 material object");

            GetListOfAllowedPhase(out string[] _, out string _2);
            UpdateCompoundList();
            CasterLogger.Debug("SetMaterial completed");
            return true;
        }

        protected override void Dispose(bool disposing)
        {
            CasterLogger.Debug("Dispose material10.");
            if (_capeThermoMaterialObject != null && _capeThermoMaterialObject.GetType().IsCOMObject)
                Marshal.ReleaseComObject(_capeThermoMaterialObject);
            //if (_capeThermoPropertyPackage != null && _capeThermoPropertyPackage.GetType().IsCOMObject)
            //    Marshal.FinalReleaseComObject(_capeThermoPropertyPackage);
            _capeThermoMaterialObject = null;
            CasterLogger.Debug("Dispose material10 completed.");
        }

        #endregion

        #region DoFlash

        protected override bool CheckEquilibriumSpec(string[] flashSpec1, string[] flashSpec2, string solutionType)
        {
            return true;
        }

        public override bool DoFlash(string[] flashSpec1, string[] flashSpec2, string solutionType, bool showWarning = false)
        {
            CasterLogger.Debug("DoFlash with " + flashSpec1.First());
            try
            {
                _capeThermoMaterialObject.CalcEquilibrium(flashSpec1.First(), null);
                //_alreadyFlashed = true;
            }
            catch (Exception e)
            {
                if (showWarning)
                    MessageBox.Show("Flash failed. " + e.Message);
                CasterLogger.Error("Flash failed. " + e.Message);
                Debug.WriteLine("Flash failed. {0}", e.Message);
                return false;
            }
            CasterLogger.Debug("Flash completed.");
            return true;
        }

        public override bool DoTPFlash(bool showWarning = false)
        {
            CasterLogger.Debug("DoTPFlash");
            if (_capeThermoMaterialObject == null) return false;
            try
            {
                _capeThermoMaterialObject.CalcEquilibrium("TP", null);
                //_alreadyFlashed = true;
            }
            catch (Exception e)
            {
                if (showWarning)
                    MessageBox.Show("Flash failed. " + e.Message);
                CasterLogger.Error("Flash failed. " + e.Message);
                Debug.WriteLine("Flash failed. {0}", e.Message);
                return false;
            }
            CasterLogger.Debug("Flash completed.");
            return true;
        }

        public override bool DoPHFlash(bool showWarning = false)
        {
            CasterLogger.Debug("DoPHFlash");
            if (_capeThermoMaterialObject == null) return false;
            try
            {
                _capeThermoMaterialObject.CalcEquilibrium("PH", null);
                //_alreadyFlashed = true;
            }
            catch (Exception e)
            {
                if (showWarning)
                    MessageBox.Show("Flash failed. " + e.Message);
                CasterLogger.Error("Flash failed. " + e.Message);
                Debug.WriteLine("Flash failed. {0}", e.Message);
                return false;
            }
            CasterLogger.Debug("Flash completed.");
            return true;
        }

        public override bool DoTHFlash(bool showWarning = false)
        {
            CasterLogger.Debug("DoTHFlash");
            if (_capeThermoMaterialObject == null) return false;
            try
            {
                _capeThermoMaterialObject.CalcEquilibrium("TH", null);
                //_alreadyFlashed = true;
            }
            catch (Exception e)
            {
                if (showWarning)
                    MessageBox.Show("Flash failed. " + e.Message);
                CasterLogger.Error("Flash failed. " + e.Message);
                Debug.WriteLine("Flash failed. {0}", e.Message);
                return false;
            }
            CasterLogger.Debug("Flash completed.");
            return true;
        }

        public override bool DoTVFFlash(bool showWarning = false)
        {
            CasterLogger.Debug("DoTVFFlash");
            if (_capeThermoMaterialObject == null) return false;
            try
            {
                _capeThermoMaterialObject.CalcEquilibrium("TVF", null);
                //_alreadyFlashed = true;
            }
            catch (Exception e)
            {
                if (showWarning)
                    MessageBox.Show("Flash failed. " + e.Message);
                CasterLogger.Error("Flash failed. " + e.Message);
                Debug.WriteLine("Flash failed. {0}", e.Message);
                return false;
            }
            CasterLogger.Debug("Flash completed.");
            return true;
        }

        public override bool DoPVFlash(bool showWarning = false)
        {
            CasterLogger.Debug("DoTVFFlash");
            if (_capeThermoMaterialObject == null) return false;
            try
            {
                _capeThermoMaterialObject.CalcEquilibrium("PVF", null);
                //_alreadyFlashed = true;
            }
            catch (Exception e)
            {
                if (showWarning)
                    MessageBox.Show("Flash fails. " + e.Message);
                CasterLogger.Error("Flash failed. " + e.Message);
                Debug.WriteLine("Flash fails. {0}", e.Message);
                return false;
            }
            CasterLogger.Debug("Flash completed.");
            return true;
        }

        #endregion

        #region Phase

        public override Phases[] GetListOfAllowedPhase(out string[] phaseAggregationList, out string keyCompoundId)
        {
            CasterLogger.Debug("GetListOfAllowedPhase");
            object allowedPhaseObject = null;
            phaseAggregationList = null;  //CO1.0 Not support
            keyCompoundId = null;         //CO1.0 Not support
            if (_capeThermoMaterialObject == null) return null;
            //try
            //{
            //    if (_capeThermoPropertyPackage == null)
            //        allowedPhaseObject = _capeThermoMaterialObject.PhaseIds;
            //    else
            //        allowedPhaseObject = _capeThermoPropertyPackage.GetPhaseList();

            //}
            //catch (Exception e)
            //{
            CasterLogger.Debug("Allowed phase not found, use Vapor and Liquid.");
            Debug.WriteLine("Allowed phase not found, use Vapor and Liquid.");
            allowedPhaseObject = new[] { "Vapor", "Liquid" };
            //}

            string[] phaseStringList = allowedPhaseObject as string[];
            Phases[] phaseList = (from phaseString in phaseStringList select new Phases(phaseString)).ToArray();

            //Set Proper Phase name
            var phases = string.Join(", ", phaseList.Select(p => p.Value).ToArray());
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
            object phaseLabel;
            presentPhaseStatus = null;   //CO1.0 Not support
            if (_capeThermoMaterialObject == null) return null;
            try
            {
                phaseLabel = _capeThermoMaterialObject.PhaseIds;
            }
            catch (Exception e)
            {
                CasterLogger.ErrorFormatted("No present phase. {0}", e.Message);
                Debug.WriteLine("No present phase. {0}", e.Message);
                phaseLabel = new string[0];
            }

            string[] phaseStringList = phaseLabel as string[];
            Phases[] phaseList = (from phaseString in phaseStringList
                                  select new Phases(phaseString)).ToArray();
            var phases = string.Join(", ", phaseList.Select(p => p.Value).ToArray());
            CasterLogger.Debug("Material present phases: " + phases);
            return phaseList;
        }

        public override void SetListOfPresentPhases(IEnumerable<Phases> presentPhases, IEnumerable<eCapePhaseStatus> presentPhasesStatus)
        {
            CasterLogger.Error("CO1.0 cannot set Present Phases");
            throw new Exception("CO1.0 cannot set Present Phases");
        }

        #endregion

        #region CompoundId

        public override string[] Formulas
        {
            get
            {
                CasterLogger.Error("formula name is unavailable for CO1.0");
                throw new Exception("formula name is unavailable for CO1.0");
            }
        }

        public override string[] UpdateCompoundList()
        {
            try
            {
                aliasName = _capeThermoMaterialObject.ComponentIds as string[];
                CasterLogger.Debug("UpdateCompoundList result" + aliasName);
                return aliasName;
            }
            catch (Exception e)
            {
                CasterLogger.ErrorFormatted("Unable to get compound list. Make sure to call UpdateComoundList after compound list changed. {0}", e.Message);
                Debug.WriteLine("Unable to get compound list. Make sure to call UpdateComoundList after compound list changed. {0}", e.Message);
                return null;
            }
        }

        #endregion

        #region Overall Property

        /// <summary>
        /// Calculate or get property
        /// </summary>
        /// <param name="propName">property name, defined in CO reference</param>
        /// <param name="phases">empty means </param>
        /// <param name="basis"></param>
        /// <param name="calculate"></param>
        /// <returns></returns>
        protected double[] GetPropList(string propName, PropertyBasis basis, Phases[] phases = null, bool calculate = true)
        {
            object value = null;
            try
            {
                // overall don't need calculate 
                if (calculate && phases != null && phases.Length != 0)
                {
                    CasterLogger.Debug($"Calculate property {propName} for {phases}");
                    _capeThermoMaterialObject.CalcProp(new[] { propName }, phases, "mixture");
                }
            }
            catch (Exception e)
            {
                Debug.WriteLine("Calculate single phase prop {0} fails. {1}", propName, e.Message);
            }
            if (phases == null || phases.Length == 0) // overall
            {
                CasterLogger.Debug($"Get property {propName} for overall phase");
                value = _capeThermoMaterialObject.GetProp(propName, "Overall", null, "mixture", basis.ToString());
            }
            else if (phases.Length == 1)  // one phase
            {
                CasterLogger.Debug($"Get property {propName} for {phases[0]}");
                value = _capeThermoMaterialObject.GetProp(propName, phases[0].Value, null, "mixture", basis.ToString());
            }
            else if (phases.Length == 2)  // two phases, example: kvalues
            {
                CasterLogger.Debug($"Get property {propName} for {phases[0]} and {phases[1]}");
                string phaseName = null;
                if (phases.Contains(Phases.Vapor) && phases.Contains(Phases.Liquid))
                    phaseName = Phases.Vapor.Value + Phases.Liquid.Value;   // vapor in front
                else
                    phaseName = phases[0].Value + phases[1].Value;
                value = _capeThermoMaterialObject.GetProp(propName, phaseName, null, "mixture", basis.ToString());
            }
            else
            {
                CasterLogger.Error($"Only support overall, 1 phase or 2 phases");
                throw new ArgumentOutOfRangeException($"Only support overall, 1 phase or 2 phases");
            }
            CasterLogger.Debug($"Get property {propName}: {value}");
            return value as double[];
        }

        /// <summary>
        /// set property
        /// </summary>
        /// <param name="propName">property name, defined in CO reference</param>
        /// <param name="phases">empty means overall, if it's vapor and liquid, will be combined to "vaporliquid", otherwise, just add the names</param>
        /// <returns></returns>
        protected void SetPropList(string propName, PropertyBasis basis, IEnumerable<double> value, Phases[] phases = null)
        {
            CasterLogger.Debug($"Set property for {propName}: {value}");
            double[] temp = value as double[] ?? value.ToArray();
            if (phases == null || phases.Length == 0) // overall
            {
                CasterLogger.Debug($"Set property {propName} for overall phase");
                _capeThermoMaterialObject.SetProp(propName, "Overall", null, "mixture", basis.ToString(), temp);
            }
            else if (phases.Length == 1)  // one phase
            {
                CasterLogger.Debug($"Set property {propName} for {phases[0]}");
                _capeThermoMaterialObject.SetProp(propName, phases[0].Value, null, null, basis.ToString(), temp);
            }
            else if (phases.Length == 2)  // two phases
            {
                CasterLogger.Debug($"Set property {propName} for {phases[0]} and {phases[1]}");
                string phaseName = null;
                if (phases.Contains(Phases.Vapor) && phases.Contains(Phases.Liquid))
                    phaseName = Phases.Vapor.Value + Phases.Liquid.Value;   // vapor in front
                else
                    phaseName = phases[0].Value + phases[1].Value;
                _capeThermoMaterialObject.SetProp(propName, phaseName, null, "mixture", basis.ToString(), temp);
            }
            else
            {
                CasterLogger.Error($"Only support overall, 1 phase or 2 phases");
                throw new ArgumentOutOfRangeException($"Only support overall, 1 phase or 2 phases");
            }
            //_alreadyFlashed = false;
            CasterLogger.Debug($"Set property completed");
        }

        public override double[] GetOverallPropList(string propName, PropertyBasis basis, bool calculate = false)
        {
            return GetPropList(propName, basis);
        }

        public override void SetOverallPropList(string propName, PropertyBasis basis, IEnumerable<double> value)
        {
            SetPropList(propName, basis, value);
        }

        #endregion

        #region Single Phase Property

        public override string[] AvailableSinglePhaseProp
        {
            get { return _capeThermoMaterialObject.GetPropList(); }
        }

        public override double[] GetSinglePhasePropList(string propName, Phases phase, PropertyBasis basis, bool calculate = true)
        {
            return GetPropList(propName, basis, new[] { phase }, calculate);
        }

        public override void SetSinglePhasePropList(string propName, Phases phase, PropertyBasis basis, IEnumerable<double> value)
        {
            SetPropList(propName, basis, value, new[] { phase });
        }

        #endregion

        #region Two Phase Property

        public override string[] AvailableTwoPhaseProp
        {
            get { return _capeThermoMaterialObject.GetPropList(); }
        }

        public override double[] GetTwoPhasePropList(string propName, Phases phase1, Phases phase2, PropertyBasis basis, bool calculate = true)
        {
            return GetPropList(propName, basis, new[] { phase1, phase2 }, calculate);
        }

        public override void SetTwoPhasePropList(string propName, Phases phase1, Phases phase2, PropertyBasis basis, IEnumerable<double> value)
        {
            SetPropList(propName, basis, value, new[] { phase1, phase2 });
        }

        #endregion

        #region Constant Property & T-Dependent Property & P-Dependent Property

        public override string[] AvailableConstProp
        {
            get { throw new System.Exception("CO1.0 Not support AvailableConstProp."); }
        }

        public override string[] AvailableTDependentProp
        {
            get { throw new System.Exception("CO1.0 Not support AvailableTDependentProp."); }
        }

        public override string[] AvailablePDependentProp
        {
            get { throw new System.Exception("CO1.0 Not support AvailablePDependentProp."); }
        }

        public override string[] AvailableUniversalConstProp
        {
            get { throw new System.Exception("CO1.0 Not support AvailableUniversalConstProp."); }
        }

        public override double GetCompoundConstPropDouble(string propName, string compoundId)
        {
            object[] value = _capeThermoMaterialObject.GetComponentConstant(new[] { propName }, new[] { compoundId });
            return Convert.ToDouble(value.SingleOrDefault());
        }

        public override double GetCompoundTDependentProp(string propName, string compoundId, double T)
        {
            throw new System.Exception("CO1.0 Not support GetCompoundTDependentProp");
        }

        public override double GetCompoundPDependentProp(string propName, string compoundId, double P)
        {
            throw new System.Exception("CO1.0 Not support GetCompoundPDependentProp");
        }

        public override double GetUniversalConstProp(string constantId)
        {
            return (_capeThermoMaterialObject.GetUniversalConstant(new[] { constantId }) as double[]).SingleOrDefault();
        }

        #endregion
    }
}
