#include "StdAfx.h"
#include "Compound.h"
#include "IdealThermoModule.h"

//! Load the Compound from a configuration file
/*!
  The compound configuration files are stored in 
  the data folder contained by the folder that 
  contains the DLL.
  
  Compounds are stored in .compound files, each line contains one data item in the following order:
  
  Name: compound name 

  Formula: chemical formula 

  CAS: CAS registry number

  MW: relative molecular weight

  NBP: normal boiling point [K]

  TC: critical temperature [K]

  PC: critical pressure [Pa]

  VC: critical volume [m3/mol]

  CPA, CPB, CPC,CPD, CPE: coefficients of the ideal gas heat capacity correlation [J/mol/K]
  
  HVAPA, HVAPB, HVAPC, HVAPD, HVAPE: coefficients of the heat of vaporization correlation [J/mol]

  RHOLA, RHOLB, RHOLC, RHOLD, RHOLE: coefficients of the liquid density correlation [mol/m3]

  ANTA, ANTB, ANTC: coefficients of the Antoine correlation [Pa]
  
  \param compName Name of the file of the compound to load (e.g. "hexane")
  \param error Error message in case of failure
  \return True for success, false for error
  \sa ::GetDataPath(), Correlation, Antoine
*/

bool Compound::Load(const char *compName,string &error)
{string path;
 path=GetDataPath();
 path+="\\";
 path+=compName;
 path+=".compound";
 FILE *fin;
 if (!name.empty()) 
  {error="Compounds can only be loaded once";
   return false;
  }
 int errCode=fopen_s(&fin,path.c_str(),"rb");
 if (errCode)
  {error="Failed to open \"";
   error+=path;
   error+="\": ";
   error+=ErrorString(errCode);
   return false;
  }
 //read content
 string line;
 if (!ReadLine(fin,line))
  {eof:
   error="Failed to read compound from \"";
   error+=path;
   error+="\": unexpected end of file";
   fclose(fin);
   return false;
  }
 if (lstrcmpi(compName,line.c_str())!=0)
  {//compound name must match file name (we depend on this while saving property packages)
   error="Compound name does not match file name for \"";
   error+=path;
   error+="\"";
   fclose(fin);
   return false;
  }
 name=line;
 if (!ReadLine(fin,line)) goto eof;
 formula=line;
 if (!ReadLine(fin,line)) goto eof;
 CAS=line;
 if (!ReadLine(fin,line)) goto eof;
 if (sscanf_s(line.c_str(),"%lg",&MW)!=1)
  {error="Failed to read molecular weight from \"";
   error+=path;
   error+='"'; 
   fclose(fin);
   return false;
  }
 if (!ReadLine(fin,line)) goto eof;
 if (sscanf_s(line.c_str(),"%lg",&NBP)!=1)
  {error="Failed to read normal boiling point from \"";
   error+=path;
   error+='"'; 
   fclose(fin);
   return false;
  }
 if (!ReadLine(fin,line)) goto eof;
 if (sscanf_s(line.c_str(),"%lg",&TC)!=1)
  {error="Failed to read critical temperature from \"";
   error+=path;
   error+='"';
   fclose(fin);
   return false;
  }
 if (!ReadLine(fin,line)) goto eof;
 if (sscanf_s(line.c_str(),"%lg",&PC)!=1)
  {error="Failed to read critical pressure from \"";
   error+=path;
   error+='"';
   fclose(fin);
   return false;
  }
 if (!ReadLine(fin,line)) goto eof;
 if (sscanf_s(line.c_str(),"%lg",&VC)!=1)
  {error="Failed to read critical volume from \"";
   error+=path;
   error+='"';
   fclose(fin);
   return false;
  }
 //correlations
 double A,B,C,D,E;
 if (!ReadLine(fin,line)) goto eof;
 if (sscanf_s(line.c_str(),"%lg %lg %lg %lg %lg",&A,&B,&C,&D,&E)!=5)
  {error="Failed to read heat capacity coefficients from \"";
   error+=path;
   error+='"';
   fclose(fin);
   return false;
  }
 CpCorrelation=new Correlation(A,B,C,D,E);
 if (!ReadLine(fin,line)) goto eof;
 if (sscanf_s(line.c_str(),"%lg %lg %lg %lg %lg",&A,&B,&C,&D,&E)!=5)
  {error="Failed to read heat of vaporization coefficients from \"";
   error+=path;
   error+='"';
   fclose(fin);
   return false;
  }
 HvapCorrelation=new Correlation(A,B,C,D,E);
 if (!ReadLine(fin,line)) goto eof;
 if (sscanf_s(line.c_str(),"%lg %lg %lg %lg %lg",&A,&B,&C,&D,&E)!=5)
  {error="Failed to read liquid density coefficients from \"";
   error+=path;
   error+='"';
   fclose(fin);
   return false;
  }
 liqDensCorrelation=new Correlation(A,B,C,D,E);
 if (!ReadLine(fin,line)) goto eof;
 if (sscanf_s(line.c_str(),"%lg %lg %lg",&A,&B,&C)!=3)
  {error="Failed to read Antoine coefficients from \"";
   error+=path;
   error+='"';
   fclose(fin);
   return false;
  }
 pSatCorrelation=new Antoine(A,B,C);
 fclose(fin);
 return true;
}
