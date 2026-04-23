#pragma once

//! Supported component string constants
/*!
	Enumeration with identifiers for supported component string constants
*/

typedef enum 
{ Name=0, /*!< Compound name */
  CASNumber=1, /*!< CAS registry number */
  ChemicalFormula=2 /*!< Chemical formula, Hills notation */
} StringConstant;

//! Supported component real constants
/*!
	Enumeration with identifiers for supported component real constants
*/

typedef enum 
{ NormalBoilingPoint=0, /*!< Normal boiling point temperature [K] */
  MolecularWeight=1, /*!< Relative molecular weight */
  CriticalTemperature=2, /*!< Critical temperature [K] */
  CriticalPressure=3, /*!< Critical pressure [Pa] */
  CriticalVolume=4, /*!< Critical volume [m3/mol] */
  CriticalDensity=5 /*!< Critical density [mol/m3] */
} RealConstant;

#define RealConstantCount 6

//! Supported temperature dependent properties:
/*!
	Enumeration with identifiers for temperature dependent properties
*/

typedef enum 
{ HeatOfVaporization=0, /*!< Heat of vaporization at the saturation line [J/mol] */
  HeatOfVaporizationDT=1, /*!< Temperature derivative of heat of vaporization at the saturation line [J/mol/K] */
  IdealGasHeatCapacity=2, /*!< Ideal gas heat capacity Cp [J/mol/K] */
  IdealGasHeatCapacityDT=3, /*!< Temperature derivative of ideal gas heat capacity Cp [J/mol/K/K] */
  VaporPressure=4, /*!< Vapor pressure [Pa] */
  VaporPressureDT=5, /*!< Temperature derivative of vapor pressure [Pa] */
  LiquidDensity=6, /*!< Liquid density [mol/m3] */
  LiquidDensityDT=7 /*!< Temperature derivative of liquid density [mol/m3] */
} TDependentProperty; /*!<  */

#define TDependentPropertyCount 8

//! Supported single phase mixture properties:
/*!
	Enumeration with identifiers for supported single phase mixture properties
*/

typedef enum 
{ Density=0, /*!< Density [mol/m3] */
  DensityDT=1, /*!< Temperature derivate of density [mol/m3/K] */
  DensityDP=2, /*!< Pressure derivative of density [mol/m3/Pa] */
  DensityDX=3, /*!< Mole fraction derivative of density [mol/m3] */
  DensityDn=4, /*!< Mole number derivative (for a total of 1 mole) of density [mol/m3/mol] */
  Volume=5, /*!< Volume [m3/mol] */
  VolumeDT=6, /*!< Temperature derivate of volume [m3/mol/K] */
  VolumeDP=7, /*!< Pressure derivative of volume [m3/mol/Pa] */
  VolumeDX=8, /*!< Mole fraction derivative of volume [m3/mol] */
  VolumeDn=9, /*!< Mole number derivative (for a total of 1 mole) of volume [m3/mol] */
  Enthalpy=10, /*!< Enthalpy [J/mol] */
  EnthalpyDT=11, /*!< Temperature derivate of enthalpy [J/mol/K] */
  EnthalpyDP=12, /*!< Pressure derivative of enthalpy [J/mol/Pa] */
  EnthalpyDX=13, /*!< Mole fraction derivative of enthalpy [J/mol] */
  EnthalpyDn=14, /*!< Mole number derivative (for a total of 1 mole) of  enthalpy [J/mol]*/
  Entropy=15, /*!< Entropy [J/mol/K] */
  EntropyDT=16, /*!< Temperature derivate of entropy [J/mol/K/K] */
  EntropyDP=17, /*!< Pressure derivative of entropy [J/mol/K/Pa] */
  EntropyDX=18, /*!< Mole fraction derivative of entropy [J/mol/K] */
  EntropyDn=19, /*!< Mole number derivative (for a total of 1 mole) of entropy [J/mol/K] */
  Fugacity=20, /*!< Fugacity [Pa] */
  FugacityDT=21, /*!< Temperature derivate of fugacity [Pa/K] */
  FugacityDP=22, /*!< Pressure derivative of fugacity [Pa/Pa] */
  FugacityDX=23, /*!< Mole fraction derivative of fugacity [Pa] */
  FugacityDn=24, /*!< Mole number derivative (for a total of 1 mole) of fugacity [Pa/mol] */
  FugacityCoefficient=25, /*!< Fugacity coefficient */
  FugacityCoefficientDT=26, /*!< Temperature derivate of fugacity coefficient [1/K]*/
  FugacityCoefficientDP=27, /*!< Pressure derivative of fugacity coefficient [1/Pa] */
  FugacityCoefficientDX=28, /*!< Mole fraction derivative of fugacity coefficient */
  FugacityCoefficientDn=29, /*!< Mole number derivative (for a total of 1 mole) of fugacity coefficient [1/mol] */
  LogFugacityCoefficient=30, /*!< Ln fugacity coefficient */
  LogFugacityCoefficientDT=31, /*!< Temperature derivate of ln fugacity coefficient [1/K] */
  LogFugacityCoefficientDP=32, /*!< Pressure derivative of ln fugacity coefficient [1/Pa] */
  LogFugacityCoefficientDX=33, /*!< Mole fraction derivative of ln fugacity coefficient */
  LogFugacityCoefficientDn=34, /*!< Mole number derivative (for a total of 1 mole) of ln fugacity coefficient [1/mol] */
  Activity=35, /*!< Activity */
  ActivityDT=36, /*!< Temperature derivate of activity [1/K] */
  ActivityDP=37, /*!< Pressure derivative of activity [1/Pa] */
  ActivityDX=38, /*!< Mole fraction derivative of activity */
  ActivityDn=39 /*!< Mole number derivative (for a total of 1 mole) of activity [1/mol] */
} SinglePhaseProperty;

#define SinglePhasePropertyCount 40

//! Supported two-phase mixture properties:
/*!
	Enumeration with identifiers for supported two-phase mixture properties
*/

typedef enum 
{ Kvalue=0, /*!< K values */
  KvalueDT=1, /*!< Temperature derivate of K values [1/K]*/
  KvalueDP=2, /*!< Pressure derivative of K values [1/Pa]*/
  LogKvalue=3, /*!< ln K values */
  LogKvalueDT=4, /*!< Temperature derivate of ln K values [1/K] */
  LogKvalueDP=5, /*!< Pressure derivative of ln K values [1/Pa]*/
  //from here on out, only derivatives, see CAPE-OPEN 1.0 property packages
  KvalueDX=6, /*!< Mole fraction derivative of K values */
  KvalueDn=7, /*!< Mole number derivative (for a total of 1 mole) of K values */
  LogKvalueDX=8, /*!< Mole fraction derivative of ln K values [1/mol] */
  LogKvalueDn=9 /*!< Mole number derivative (for a total of 1 mole) of ln K values [1/mol] */
} TwoPhaseProperty;

#define TwoPhasePropertyCount 10

//! Supported phases:
/*!
	Enumeration with identifiers for supported phases
*/

typedef enum 
{ Vapor=0, /*!< Vapor phase */
  Liquid=1, /*!< Liquid phase */
} Phase;

#define PhaseCount 2

//! Supported flashes:
/*!
	Enumeration with identifiers for phase equilibrium (flash) calculations
*/

typedef enum 
{ TP=0,  /*!< Temperature [K], Pressure [Pa]*/
  TVF=1, /*!< Temperature [K], Vapor fraction [mol/mol]*/
  PVF=2, /*!< Pressure [Pa], Vapor fraction [mol/mol]*/
  TVFm=3, /*!< Temperature [K], Vapor fraction (mass basis) [kg/kg]*/
  PVFm=4, /*!< Pressure [Pa], Vapor fraction (mass basis) [kg/kg]*/
  PH=5, /*!< Pressure [Pa], Enthalpy [J/mol]*/
  PS=6, /*!< Pressure [Pa], Entropy [J/mol/K]*/
} FlashType;

#define FlashTypeCount 7

//! Supported flash phase combinations:
/*!
	Enumeration with identifiers for allowed phases resulting from a flash calculation
*/

typedef enum 
{ VaporLiquid=3, /*!< Both vapor and liquid are allowed (default)*/
  VaporOnly=1, /*!< Only vapor is allowed (default)*/
  LiquidOnly=2, /*!< Only liquid is allowed (default)*/
} FlashPhaseType;

#define FlashPhaseTypeCount 3

//defined only at the scope of IDealThermoModule.dll
#ifdef IDEALTHERMOMODULE_EXPORTS
#define DIMENSION_SCALAR 0
#define DIMENSION_VECTOR 1
#define DIMENSION_MATRIX 2
extern const int SinglePhasePropertyDimension[SinglePhasePropertyCount]; 
extern const int TwoPhasePropertyDimension[TwoPhasePropertyCount];
#endif

