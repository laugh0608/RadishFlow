//some definitions for the properties
#include "stdafx.h"
#include "Properties.h"

//! Single phase property dimensions
/*!
	Dimensions of single phase properties (scalar, vector or matrix)
*/

const int SinglePhasePropertyDimension[SinglePhasePropertyCount]=
 {DIMENSION_SCALAR, //Density=0,
  DIMENSION_SCALAR, //DensityDT=1,
  DIMENSION_SCALAR, //DensityDP=2,
  DIMENSION_VECTOR, //DensityDX=3,
  DIMENSION_VECTOR, //DensityDn=4,
  DIMENSION_SCALAR, //Volume=5,
  DIMENSION_SCALAR, //VolumeDT=6,
  DIMENSION_SCALAR, //VolumeDP=7,
  DIMENSION_VECTOR, //VolumeDX=9,
  DIMENSION_VECTOR, //VolumeDn=9,
  DIMENSION_SCALAR, //Enthalpy=10,
  DIMENSION_SCALAR, //EnthalpyDT=11,
  DIMENSION_SCALAR, //EnthalpyDP=12,
  DIMENSION_VECTOR, //EnthalpyDX=13,
  DIMENSION_VECTOR, //EnthalpyDn=14,
  DIMENSION_SCALAR, //Entropy=15,
  DIMENSION_SCALAR, //EntropyDT=16,
  DIMENSION_SCALAR, //EntropyDP=17,
  DIMENSION_VECTOR, //EntropyDX=18,
  DIMENSION_VECTOR, //EntropyDn=19,
  DIMENSION_VECTOR, //Fugacity=20,
  DIMENSION_VECTOR, //FugacityDT=21,
  DIMENSION_VECTOR, //FugacityDP=22,
  DIMENSION_MATRIX, //FugacityDX=23,
  DIMENSION_MATRIX, //FugacityDn=24,
  DIMENSION_VECTOR, //FugacityCoefficient=25,
  DIMENSION_VECTOR, //FugacityCoefficientDT=26,
  DIMENSION_VECTOR, //FugacityCoefficientDP=27,
  DIMENSION_MATRIX, //FugacityCoefficientDX=28,
  DIMENSION_MATRIX, //FugacityCoefficientDn=29,
  DIMENSION_VECTOR, //LogFugacityCoefficient=30,
  DIMENSION_VECTOR, //LogFugacityCoefficientDT=31,
  DIMENSION_VECTOR, //LogFugacityCoefficientDP=32,
  DIMENSION_MATRIX, //LogFugacityCoefficientDX=33,
  DIMENSION_MATRIX, //LogFugacityCoefficientDn=34,
  DIMENSION_VECTOR, //Activity=35,
  DIMENSION_VECTOR, //ActivityDT=36,
  DIMENSION_VECTOR, //ActivityDP=37,
  DIMENSION_MATRIX, //ActivityDX=38,
  DIMENSION_MATRIX  //ActivityDn=39
 };
 
 //! Two-phase property dimensions
/*!
	Dimensions of two-phase properties (scalar, vector or matrix)
*/

const int TwoPhasePropertyDimension[TwoPhasePropertyCount]=
 {DIMENSION_VECTOR, //Kvalue=0, 
  DIMENSION_VECTOR, //KvalueDT=1,
  DIMENSION_VECTOR, //KvalueDP=2,
  DIMENSION_VECTOR, //LogKvalue=3,
  DIMENSION_VECTOR, //LogKvalueDT=4, 
  DIMENSION_VECTOR, //LogKvalueDP=5, 
  DIMENSION_MATRIX, //KvalueDX=6,
  DIMENSION_MATRIX, //KvalueDn=7,
  DIMENSION_MATRIX, //LogKvalueDX=8,
  DIMENSION_MATRIX  //LogKvalueDn=9 
 };