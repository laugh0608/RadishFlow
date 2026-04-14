Attribute VB_Name = "PropertyConstants"
Option Explicit

'compound string constants
  Public Const CompoundName As Long = 0 'Compound name
  Public Const CASNumber As Long = 1 'CAS registry number
  Public Const ChemicalFormula As Long = 2 'Chemical formula, Hills notation
  
  Public Const StringConstantsCount As Long = 3

'compound real constants
  Public Const NormalBoilingPoint As Long = 0 'Normal boiling point temperature [K]
  Public Const MolecularWeight As Long = 1 'Relative molecular weight
  Public Const CriticalTemperature As Long = 2 'Critical temperature [K]
  Public Const CriticalPressure As Long = 3 'Critical pressure [Pa]
  Public Const CriticalVolume As Long = 4 'Critical volume [m3/mol]
  Public Const CriticalDensity As Long = 5 'Critical density [mol/m3]

  Public Const RealConstantsCount As Long = 6

'temperature dependent properties
  Public Const HeatOfVaporization As Long = 0 'Heat of vaporization at the saturation line [J/mol]
  Public Const HeatOfVaporizationDT As Long = 1 'Temperature derivative of heat of vaporization at the saturation line [J/mol/K]
  Public Const IdealGasHeatCapacity As Long = 2 'Ideal gas heat capacity Cp [J/mol/K]
  Public Const IdealGasHeatCapacityDT As Long = 3 'Temperature derivative of ideal gas heat capacity Cp [J/mol/K/K]
  Public Const VaporPressure As Long = 4 'Vapor pressure [Pa]
  Public Const VaporPressureDT As Long = 5 'Temperature derivative of vapor pressure [Pa]

  Public Const TDepPropertyCount As Long = 6
  
'not exposed as T-dependent property but used for compound constant:
  Public Const LiquidDensity As Long = 6

'single phase properties
  Public Const Density As Long = 0 'Density [mol/m3]
  Public Const DensityDT As Long = 1 'Temperature derivate of density [mol/m3/K]
  Public Const DensityDP As Long = 2 'Pressure derivative of density [mol/m3/Pa]
  Public Const DensityDX As Long = 3 'Mole fraction derivative of density [mol/m3]
  Public Const DensityDn As Long = 4 'Mole number derivative (for a total of 1 mole) of density [mol/m3/mol]
  Public Const Volume As Long = 5 'Volume [m3/mol]
  Public Const VolumeDT As Long = 6 'Temperature derivate of volume [m3/mol/K]
  Public Const VolumeDP As Long = 7 'Pressure derivative of volume [m3/mol/Pa]
  Public Const VolumeDX As Long = 8 'Mole fraction derivative of volume [m3/mol]
  Public Const VolumeDn As Long = 9 'Mole number derivative (for a total of 1 mole) of volume [m3/mol]
  Public Const Enthalpy As Long = 10 'Enthalpy [J/mol]
  Public Const EnthalpyDT As Long = 11 'Temperature derivate of enthalpy [J/mol/K]
  Public Const EnthalpyDP As Long = 12 'Pressure derivative of enthalpy [J/mol/Pa]
  Public Const EnthalpyDX As Long = 13 'Mole fraction derivative of enthalpy [J/mol]
  Public Const EnthalpyDn As Long = 14 'Mole number derivative (for a total of 1 mole) of  enthalpy [J/mol]
  Public Const Entropy As Long = 15 'Entropy [J/mol/K]
  Public Const EntropyDT As Long = 16 'Temperature derivate of entropy [J/mol/K/K]
  Public Const EntropyDP As Long = 17 'Pressure derivative of entropy [J/mol/K/Pa]
  Public Const EntropyDX As Long = 18 'Mole fraction derivative of entropy [J/mol/K]
  Public Const EntropyDn As Long = 19 'Mole number derivative (for a total of 1 mole) of entropy [J/mol/K]
  Public Const Fugacity As Long = 20 'Fugacity [Pa]
  Public Const FugacityDT As Long = 21 'Temperature derivate of fugacity [Pa/K]
  Public Const FugacityDP As Long = 22 'Pressure derivative of fugacity [Pa/Pa]
  Public Const FugacityDX As Long = 23 'Mole fraction derivative of fugacity [Pa]
  Public Const FugacityDn As Long = 24 'Mole number derivative (for a total of 1 mole) of fugacity [Pa/mol]
  Public Const FugacityCoefficient As Long = 25 'Fugacity coefficient
  Public Const FugacityCoefficientDT As Long = 26 'Temperature derivate of fugacity coefficient [1/K]
  Public Const FugacityCoefficientDP As Long = 27 'Pressure derivative of fugacity coefficient [1/Pa]
  Public Const FugacityCoefficientDX As Long = 28 'Mole fraction derivative of fugacity coefficient
  Public Const FugacityCoefficientDn As Long = 29 'Mole number derivative (for a total of 1 mole) of fugacity coefficient [1/mol]
  Public Const LogFugacityCoefficient As Long = 30 'Ln fugacity coefficient
  Public Const LogFugacityCoefficientDT As Long = 31 'Temperature derivate of ln fugacity coefficient [1/K]
  Public Const LogFugacityCoefficientDP As Long = 32 'Pressure derivative of ln fugacity coefficient [1/Pa]
  Public Const LogFugacityCoefficientDX As Long = 33 'Mole fraction derivative of ln fugacity coefficient
  Public Const LogFugacityCoefficientDn As Long = 34 'Mole number derivative (for a total of 1 mole) of ln fugacity coefficient [1/mol]
  Public Const Activity As Long = 35 'Activity
  Public Const ActivityDT As Long = 36 'Temperature derivate of activity [1/K]
  Public Const ActivityDP As Long = 37 'Pressure derivative of activity [1/Pa]
  Public Const ActivityDX As Long = 38 'Mole fraction derivative of activity
  Public Const ActivityDn As Long = 39 'Mole number derivative (for a total of 1 mole) of activity [1/mol]

  Public Const SinglePhasePropertyCount As Long = 40

'two phase properties
  Public Const Kvalue As Long = 0 'K values
  Public Const KvalueDT As Long = 1 'Temperature derivate of K values [1/K]
  Public Const KvalueDP As Long = 2 'Pressure derivative of K values [1/Pa]
  Public Const LogKvalue As Long = 3 'ln K values
  Public Const LogKvalueDT As Long = 4 'Temperature derivate of ln K values [1/K]
  Public Const LogKvalueDP As Long = 5 'Pressure derivative of ln K values [1/Pa]
  Public Const KvalueDX As Long = 6 'Mole fraction derivative of K values
  Public Const KvalueDn As Long = 7 'Mole number derivative (for a total of 1 mole) of K values
  Public Const LogKvalueDX As Long = 8 'Mole fraction derivative of ln K values [1/mol]
  Public Const LogKvalueDn As Long = 9 'Mole number derivative (for a total of 1 mole) of ln K values [1/mol]

  Public Const TwoPhasePropertyCount As Long = 10

'phases
  Public Const Vapor As Long = 0 'Vapor phase
  Public Const Liquid As Long = 1 'Liquid phase

'flash types
  Public Const TP As Long = 0 'Temperature [K], Pressure [Pa]
  Public Const TVF As Long = 1 'Temperature [K], Vapor fraction [mol/mol]
  Public Const PVF As Long = 2 'Pressure [Pa], Vapor fraction [mol/mol]
  Public Const TVFm As Long = 3 'Temperature [K], Vapor fraction (mass basis) [kg/kg]
  Public Const PVFm As Long = 4 'Pressure [Pa], Vapor fraction (mass basis) [kg/kg]
  Public Const PH As Long = 5 'Pressure [Pa], Enthalpy [J/mol]
  Public Const PS As Long = 6 'Pressure [Pa], Entropy [J/mol/K]

'flash phase requests
  Public Const VaporLiquid As Long = 3 'Both vapor and liquid are allowed (default)
  Public Const VaporOnly As Long = 1 'Only vapor is allowed (default)
  Public Const LiquidOnly As Long = 2 'Only liquid is allowed (default)

'arrays with property names and dimensionality data
' there are no constant arrays in VB6, we have to call an initialize routine
' we do so from the property package constructor, but we initialize only the
' first time around

Private initialized As Boolean
Public TDepPropNames(0 To TDepPropertyCount - 1) As String
Public SinglePhasePropNames(0 To SinglePhasePropertyCount - 1) As String
Public SinglePhasePropMoleBasis(0 To SinglePhasePropertyCount - 1) As Boolean
Public TwoPhasePropNames(0 To TwoPhasePropertyCount - 1) As String

Public Sub InitializeProperties()
If Not initialized Then
 TDepPropNames(0) = "heatOfVaporization"
 TDepPropNames(1) = "heatOfVaporization.Dtemperature"
 TDepPropNames(2) = "idealGasHeatCapacity"
 TDepPropNames(3) = "idealGasHeatCapacity.Dtemperature"
 TDepPropNames(4) = "vaporPressure"
 TDepPropNames(5) = "vaporPressure.Dtemperature"
 SinglePhasePropNames(0) = "density"
 SinglePhasePropNames(1) = "density.Dtemperature"
 SinglePhasePropNames(2) = "density.Dpressure"
 SinglePhasePropNames(3) = "density.DmolFraction"
 SinglePhasePropNames(4) = "density.Dmoles"
 SinglePhasePropNames(5) = "volume"
 SinglePhasePropNames(6) = "volume.Dtemperature"
 SinglePhasePropNames(7) = "volume.Dpressure"
 SinglePhasePropNames(8) = "volume.DmolFraction"
 SinglePhasePropNames(9) = "volume.Dmoles"
 SinglePhasePropNames(10) = "enthalpy"
 SinglePhasePropNames(11) = "enthalpy.Dtemperature"
 SinglePhasePropNames(12) = "enthalpy.Dpressure"
 SinglePhasePropNames(13) = "enthalpy.DmolFraction"
 SinglePhasePropNames(14) = "enthalpy.Dmoles"
 SinglePhasePropNames(15) = "entropy"
 SinglePhasePropNames(16) = "entropy.Dtemperature"
 SinglePhasePropNames(17) = "entropy.Dpressure"
 SinglePhasePropNames(18) = "entropy.DmolFraction"
 SinglePhasePropNames(19) = "entropy.Dmoles"
 SinglePhasePropNames(20) = "fugacity"
 SinglePhasePropNames(21) = "fugacity.Dtemperature"
 SinglePhasePropNames(22) = "fugacity.Dpressure"
 SinglePhasePropNames(23) = "fugacity.DmolFraction"
 SinglePhasePropNames(24) = "fugacity.Dmoles"
 SinglePhasePropNames(25) = "fugacityCoefficient"
 SinglePhasePropNames(26) = "fugacityCoefficient.Dtemperature"
 SinglePhasePropNames(27) = "fugacityCoefficient.Dpressure"
 SinglePhasePropNames(28) = "fugacityCoefficient.DmolFraction"
 SinglePhasePropNames(29) = "fugacityCoefficient.Dmoles"
 SinglePhasePropNames(30) = "logFugacityCoefficient"
 SinglePhasePropNames(31) = "logFugacityCoefficient.Dtemperature"
 SinglePhasePropNames(32) = "logFugacityCoefficient.Dpressure"
 SinglePhasePropNames(33) = "logFugacityCoefficient.DmolFraction"
 SinglePhasePropNames(34) = "logFugacityCoefficient.Dmoles"
 SinglePhasePropNames(35) = "activity"
 SinglePhasePropNames(36) = "activity.Dtemperature"
 SinglePhasePropNames(37) = "activity.Dpressure"
 SinglePhasePropNames(38) = "activity.DmolFraction"
 SinglePhasePropNames(39) = "activity.Dmoles"
 SinglePhasePropMoleBasis(0) = True '"density",
 SinglePhasePropMoleBasis(1) = True '"density.Dtemperature",
 SinglePhasePropMoleBasis(2) = True '"density.Dpressure",
 SinglePhasePropMoleBasis(3) = True '"density.DmolFraction",
 SinglePhasePropMoleBasis(4) = True '"density.Dmoles",
 SinglePhasePropMoleBasis(5) = True '"volume",
 SinglePhasePropMoleBasis(6) = True '"volume.Dtemperature",
 SinglePhasePropMoleBasis(7) = True '"volume.Dpressure",
 SinglePhasePropMoleBasis(8) = True '"volume.DmolFraction",
 SinglePhasePropMoleBasis(9) = True '"volume.Dmoles",
 SinglePhasePropMoleBasis(10) = True '"enthalpy",
 SinglePhasePropMoleBasis(11) = True '"enthalpy.Dtemperature",
 SinglePhasePropMoleBasis(12) = True '"enthalpy.Dpressure",
 SinglePhasePropMoleBasis(13) = True '"enthalpy.DmolFraction",
 SinglePhasePropMoleBasis(14) = True '"enthalpy.Dmoles",
 SinglePhasePropMoleBasis(15) = True '"entropy",
 SinglePhasePropMoleBasis(16) = True '"entropy.Dtemperature",
 SinglePhasePropMoleBasis(17) = True '"entropy.Dpressure",
 SinglePhasePropMoleBasis(18) = True '"entropy.DmolFraction",
 SinglePhasePropMoleBasis(19) = True '"entropy.Dmoles",
 SinglePhasePropMoleBasis(20) = False '"fugacity",
 SinglePhasePropMoleBasis(21) = False '"fugacity.Dtemperature",
 SinglePhasePropMoleBasis(22) = False '"fugacity.Dpressure",
 SinglePhasePropMoleBasis(23) = False '"fugacity.DmolFraction",
 SinglePhasePropMoleBasis(24) = False '"fugacity.Dmoles",
 SinglePhasePropMoleBasis(25) = False '"fugacityCoefficient",
 SinglePhasePropMoleBasis(26) = False '"fugacityCoefficient.Dtemperature",
 SinglePhasePropMoleBasis(27) = False '"fugacityCoefficient.Dpressure",
 SinglePhasePropMoleBasis(28) = False '"fugacityCoefficient.DmolFraction",
 SinglePhasePropMoleBasis(29) = False '"fugacityCoefficient.Dmoles",
 SinglePhasePropMoleBasis(30) = False '"logFugacityCoefficient",
 SinglePhasePropMoleBasis(31) = False '"logFugacityCoefficient.Dtemperature",
 SinglePhasePropMoleBasis(32) = False '"logFugacityCoefficient.Dpressure",
 SinglePhasePropMoleBasis(33) = False '"logFugacityCoefficient.DmolFraction",
 SinglePhasePropMoleBasis(34) = False '"logFugacityCoefficient.Dmoles",
 SinglePhasePropMoleBasis(35) = False '"activity",
 SinglePhasePropMoleBasis(36) = False '"activity.Dtemperature",
 SinglePhasePropMoleBasis(37) = False '"activity.Dpressure",
 SinglePhasePropMoleBasis(38) = False '"activity.DmolFraction",
 SinglePhasePropMoleBasis(39) = False '"activity.Dmoles"
 TwoPhasePropNames(0) = "kvalue"
 TwoPhasePropNames(1) = "kvalue.Dtemperature"
 TwoPhasePropNames(2) = "kvalue.Dpressure"
 TwoPhasePropNames(3) = "logkvalue"
 TwoPhasePropNames(4) = "logkvalue.Dtemperature"
 TwoPhasePropNames(5) = "logkvalue.Dpressure"
 TwoPhasePropNames(6) = "kvalue.DmolFraction"
 TwoPhasePropNames(7) = "kvalue.Dmoles"
 TwoPhasePropNames(8) = "logkvalue.DmolFraction"
 TwoPhasePropNames(9) = "logkvalue.Dmoles"
 initialized = True
End If
End Sub






