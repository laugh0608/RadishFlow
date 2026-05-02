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
#pragma warning disable 1591

namespace CasterCore
{
    /// <summary>
    /// Enumeration of unit categories. 
    /// Empty is not a real category, but a replacement of null, no unit should belongs to this category.
    /// </summary>
    [Serializable]
    public enum UnitCategoryEnum
    {
        Empty, Mass, Time, Length, ElectricalCurrent, Temperature, TemperatureDrop,
        AmountOfSubstance, Luminous, Currency, FlowRate,
        MassFlowRate, MolarFlowRate, DensityAndMassConcentration,
        CurrentDensity, Area, Volume, MolarConcentration,
        Velocity, Force, Pressure, PressureHead, WeightFraction,
        Accelerate, SpecificVolume, MolarDensity, MolarVolume,
        m3PerK, InverseLength, MolarWeight, Voltage, InverseTime,
        Energy, Power, HeatTransferCoefficient, Charge, MolarFlowRateDensity,
        MolarPlateDensity, Dimensionless, Angle,
    }
}
