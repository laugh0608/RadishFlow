// IdealThermoModule.cpp : Defines the entry point for the DLL application.
//

#include "stdafx.h"
#include "IdealThermoModule.h"
#include <shlobj.h>

/*! \mainpage Ideal Thermo Module
*
*(C) CO-LaN 2011: <a href="http://www.colan.org/">http://www.colan.org/</a>
*Implemented by AmsterCHEM 2011: <a href="http://www.amsterchem.com/">http://www.amsterchem.com/</a>
*
*This project (IdealThermoModule) implements a thermo server
*for a vapor liquid system of an ideal vapor phase and a liquid
*phase based on ideal activity. This implementation is merely
*meant to be serve for example CAPE-OPEN thermodynamic server
*implementations. The focus when writing the code was clarity,
*not performance or calculation stability.
*
*CO-LaN nor AmsterCHEM claim that this example is fit for purpose. 
*CO-LaN nor AmsterCHEM claim that this example implements
*the thermodynamic calculations in an efficient way; in the contrary, 
*in order to keep the example readible, no attempt is made to 
*cache allocated variables (such as strings, arrays, ...) or to provide 
* efficient lookup methods (e.g. for items in collections, ...)
*
*This implementation is intended for illustrative purposes only. Use
*this example as you please. Under no circumstance can CO-LaN or 
*AmsterCHEM be held liable for consequential or any other damages
*resulting from this code.
*
*This example implementation uses compound definitions stored in 
*.compound files in the data subfolder of the folder that contains
*IdealThermoModule.dll. In addition, Property Package definition 
*files are used, and stored as .propertypackage in the user's
*roaming data folder, in sub-folder CO-LaN_IdealThermoExample
*
*/

string userDataPath;   /*!< user data path (obtained via GetUserDataPath()) */
string systemDataPath; /*!< data path (obtained via GetDataPath()) */
HMODULE module;		   /*!< Handle of the current module */

//! DllMain DLL entry point
/*!
  This gets called when the DLL is loaded or unloaded, or 
  threads attach to or detach from the DLL
  \param hModule Module handle
  \param ul_reason_for_call Either one of DLL_PROCESS_ATTACH, DLL_THREAD_ATTACH,  DLL_THREAD_DETACH, DLL_PROCESS_DETACH
  \param lpReserved Unused
  \return True for success, false for error
*/

BOOL APIENTRY DllMain(HMODULE hModule,DWORD  ul_reason_for_call,LPVOID lpReserved)
{	switch (ul_reason_for_call)
	{case DLL_PROCESS_ATTACH:
	      module=hModule;
	      break;
	 case DLL_THREAD_ATTACH:
	 case DLL_THREAD_DETACH:
	 case DLL_PROCESS_DETACH:
		  break;
	}
    return TRUE;
}


//! GetUserDataPath
/*!
  Get the path for the storage of user data (property package configurations)
  \return user data path
  \sa GetDataPath()
*/

string GetUserDataPath()
{if (userDataPath.empty())
  {//get the user data path
   char *path;
   path=new char[MAX_PATH];
   SHGetSpecialFolderPath(GetActiveWindow(),path,CSIDL_APPDATA,TRUE);
   userDataPath=path;
   delete []path;
   if (userDataPath[userDataPath.length()-1]!='\\') userDataPath+='\\';
   userDataPath+="CO-LaN_IdealThermoExample";
   if (CreateDirectory(userDataPath.c_str(),NULL))
    {//directory was created newly, copy sample packages in there
     int i;
     string src,dest;
     vector<string> examplePackages;
     GetDataPath();
     ListFiles(systemDataPath.c_str(),"propertypackage",examplePackages);
     for (i=0;i<(int)examplePackages.size();i++)
      {src=systemDataPath;src+='\\';src+=examplePackages[i];src+=".propertypackage";
       dest=userDataPath;dest+='\\';dest+=examplePackages[i];dest+=".propertypackage";
       CopyFile(src.c_str(),dest.c_str(),TRUE);
      }
    }
  }
 return userDataPath.c_str();
}

//! GetDataPath
/*!
  Get the path for the storage of compounds, etc
  \return data path
  \sa GetUserDataPath()
*/

string GetDataPath()
{if (systemDataPath.empty())
  {//data is located in the data sub folder of the folder this DLL is in
   char *path;
   path=new char[MAX_PATH];
   GetModuleFileName(module,path,MAX_PATH);
   systemDataPath=path;
   delete []path;
   int index=(int)systemDataPath.size()-1;
   while (index>0)
    {if (systemDataPath[index]=='\\')
      {systemDataPath=systemDataPath.substr(0,index+1);
       break;
      }
     index--;
    }
   systemDataPath+="data";
  }
 return systemDataPath.c_str();
}

//! Helper function to read a line from a file
/*!
  Lines are stripped of leading and trailing white space (space, tab).
  Empty lines or lines starting with # are skipped.
  \param f File from which to read line. Must have been opened with read access
  \param line String returning the content of the line read
  \return False in case no lines are available anymore
*/

bool ReadLine(FILE *f,string &line)
{bool eof=false;
 line.clear();
 while ((line.size()==0)&&(!eof))
  {char c;
   while (true)
    {if (fread(&c,1,1,f)!=1) 
      {eof=true;
       break;
      }
     if (c)
      if (c!='\r')
       {if (c=='\n') break;
        line+=c;
       }
    }
   //strip white space
   while (line.size()) 
    {c=line[0];
     if ((c!=' ')&&(c!='\t')) break;
     line=line.substr(1);
    }
   while (line.size()) 
    {c=line[line.size()-1];
     if ((c!=' ')&&(c!='\t')) break;
     line=line.substr(0,line.size()-1);
    }
   //check comment line
   if (line.size()) if (line[0]=='#') line.clear();
  }
 return (line.size()>0);
}

//! Helper function to list files in a folder
/*!
  List all files with a given extension in a given folder 
  \param folder Folder to look in
  \param ext File extension to look for
  \param fileNames Will contain the names of the files (without folder or file extension) upon return
*/

void ListFiles(const char *folder,const char *ext,vector<string> &fileNames)
{WIN32_FIND_DATA FindFileData;
 HANDLE hFind=INVALID_HANDLE_VALUE;
 int i;
 string spec;
 string fileName;
 spec=folder;
 spec+="\\*.";
 spec+=ext;
 fileNames.clear();
 hFind=FindFirstFile(spec.c_str(),&FindFileData);
 if (hFind!=INVALID_HANDLE_VALUE) 
  {fileName=FindFileData.cFileName;
   i=(int)fileName.size()-1;
   while (i>=0)
    {if (fileName[i]=='.')
      {fileName=fileName.substr(0,i); 
       break;
      }
     i--;
    }
   fileNames.push_back(fileName);
   while (FindNextFile(hFind,&FindFileData)) 
    {fileName=FindFileData.cFileName;
     i=(int)fileName.size()-1;
     while (i>=0)
      {if (fileName[i]=='.')
        {fileName=fileName.substr(0,i); 
         break;
        }
       i--;
      }
     fileNames.push_back(fileName);
    }
   FindClose(hFind);
  }
}

//! Helper function for error string from errno error code
/*!
  \param errCode Error code
  \return Error descriptor
*/

string ErrorString(int errCode)
{string res;
 char *buf=new char[100];
 strerror_s(buf,100,errCode);
 char *ptr=buf;
 while (*ptr)
  {if ((*ptr=='\r')||(*ptr=='\n')) 
    {*ptr=0;
     break;
    }
   ptr++;
  }
 res=buf;
 delete []buf;  
 return res;
}
