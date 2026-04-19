#include "StdAfx.h"
#include "ThermoSystemEditor.h"
#include "IdealThermoModule.h"
#include "PackageEditor.h"
#include "PropertyPackage.h"
#include "resource.h"
#include "EditBox.h"
#include "PropertyPackageEnumerator.h"

extern HMODULE module;

//! Constructor
/*!
  Called upon construction of a ThermoSystemEditor instance
  \sa InitDialog()
*/

ThermoSystemEditor::ThermoSystemEditor()
{//does nothing
}

//! Window procedure for PropertyPackage Edit dialog
/*!
  Dialog box procedure for the edit dialog. Handles all the relevant window messages
  \param hwndDlg Dialog window handle
  \param uMsg Message
  \param wParam wParam
  \param lParam lParam
  \sa ThermoSystemEditor
*/

INT_PTR CALLBACK ThermoSystemEditorWindowProc(HWND hwndDlg,UINT uMsg,WPARAM wParam,LPARAM lParam)
{ThermoSystemEditor *editor;
 switch (uMsg)
  {case WM_INITDIALOG:
        //the lParam contains the pointer to the PackageEditor object
        editor=(ThermoSystemEditor *)lParam;
        editor->hDlg=hwndDlg;
        //store for subsequent calls
        #pragma warning ( suppress : 4244 ) //warning C4244 in VS2005 is a bug, can be ignored
        SetWindowLongPtr(hwndDlg,GWLP_USERDATA,(LONG_PTR)editor);
        //init
        editor->InitDialog();
        return TRUE;
   case WM_COMMAND:
        //get pointer to editor
        #pragma warning ( suppress : 4312 ) //warning C4312 in VS2005 is a bug, can be ignored
        editor=(ThermoSystemEditor *)GetWindowLongPtr(hwndDlg,GWLP_USERDATA); 
        //handle command
        switch (LOWORD(wParam))
         {case IDC_EDIT: 
               editor->OnEdit();
               return TRUE;
          case IDC_NEW: 
               editor->OnNew();
               return TRUE;
          case IDC_DELETE:
               editor->OnDelete();
               return TRUE;
          case IDC_RENAME:
               editor->OnRename();
               return TRUE;
          case IDCANCEL:
               editor->OnCancel();
               return TRUE;
          case IDC_PACKAGELIST:
               if (HIWORD(wParam)==LBN_SELCHANGE) 
                {editor->OnSelChangePackage();
                 return TRUE;
                }
               else if (HIWORD(wParam)==LBN_DBLCLK) 
                {editor->OnEdit();
                 return TRUE;
                }
               break;
         }
        break;
  }
 return FALSE;
}

//! Edit routine
/*!
  Shows the edit dialog. 
*/

void ThermoSystemEditor::Edit()
{//get parent window 
 HWND parentWindow=GetActiveWindow();
 if (!IsWindow(parentWindow)) parentWindow=NULL; //do not show as child of invalid window
 if (parentWindow) if (!IsWindowVisible(parentWindow)) parentWindow=NULL; //do not show as child of invisible window
 //create and show dialog
 DialogBoxParam(module,MAKEINTRESOURCE(IDD_THERMOSYSTEMDIALOG),parentWindow,ThermoSystemEditorWindowProc,(LPARAM)this);
}

//! Initialize the dialog

void ThermoSystemEditor::InitDialog()
{//fill the package list
 PropertyPackageEnumerator pEnum;
 for (int i=0;i<pEnum.Count();i++) SendDlgItemMessage(hDlg,IDC_PACKAGELIST,LB_ADDSTRING,0,(LPARAM)pEnum.PackageName(i));
 EnableButtons();
}

//! Create a new Property Package

void ThermoSystemEditor::OnNew()
{EditBox edit("New Property Package:","Enter package name:");
 if (edit.Edit(hDlg))
  {string newName=edit.Result();
   //check if name is unique
   int i;
   PropertyPackageEnumerator pEnum;
   for (i=0;i<pEnum.Count();i++)
    if (lstrcmpi(pEnum.PackageName(i),newName.c_str())==0)
     {MessageBox(hDlg,"A package with that name already exists","New package:",MB_ICONHAND);
      return;
     }
   PropertyPackage package;
   if (package.Edit())
    {string path=GetUserDataPath();
     path+='\\';
     path+=newName;
     if (package.Save(path.c_str()))
      {//add to list
       SendDlgItemMessage(hDlg,IDC_PACKAGELIST,LB_SETCURSEL,
         SendDlgItemMessage(hDlg,IDC_PACKAGELIST,LB_ADDSTRING,0,(LPARAM)newName.c_str()),
          0);
       EnableButtons();
      }
     else 
      {string s;
       s="Failed to save property package: ";
       s+=package.LastError();
       MessageBox(hDlg,s.c_str(),"New package:",MB_ICONHAND);
      }
    }
  }
}

//! Rename an existing Property Package

void ThermoSystemEditor::OnRename()
{string oldName=GetSelectedPackage();
 if (!oldName.empty())
  {//ask for a new name
   EditBox box("Rename package:","Enter new package name:",oldName.c_str());
   if (box.Edit(hDlg))
    {string newName=box.Result();
     if (lstrcmp(newName.c_str(),oldName.c_str())) //else no change
      {//check if name does not already exist 
       PropertyPackageEnumerator pEnum;
       for (int i=0;i<pEnum.Count();i++)
        if (lstrcmpi(newName.c_str(),pEnum.PackageName(i))==0) //same as existing, case insensitive
         if (lstrcmpi(oldName.c_str(),pEnum.PackageName(i))) //but not the current package
          {MessageBox(hDlg,"A package with that name already exist.","Rename package:",MB_ICONHAND);
           return;
          }
       //proceed
       string oldPath,newPath;
       oldPath=GetUserDataPath();
       oldPath+='\\';
       oldPath+=oldName;
       oldPath+=".propertypackage";       
       newPath=GetUserDataPath();
       newPath+='\\';
       newPath+=newName;
       newPath+=".propertypackage";       
       if (!MoveFile(oldPath.c_str(),newPath.c_str()))
        {MessageBox(hDlg,"Failed to rename property package","Rename package:",MB_ICONHAND);
        }
       else
        {//update list
         int index=(int)SendDlgItemMessage(hDlg,IDC_PACKAGELIST,LB_GETCURSEL,0,0);
         SendDlgItemMessage(hDlg,IDC_PACKAGELIST,LB_DELETESTRING,index,0);
         index=(int)SendDlgItemMessage(hDlg,IDC_PACKAGELIST,LB_ADDSTRING,0,(LPARAM)newName.c_str());
         SendDlgItemMessage(hDlg,IDC_PACKAGELIST,LB_SETCURSEL,index,0);
		 EnableButtons();
        }
      }
    }
  }
}

//! Delete an existing Property Package

void ThermoSystemEditor::OnDelete()
{string name=GetSelectedPackage();
 if (!name.empty())
  {string prompt="Delete property package \"";
   prompt+=name;
   prompt+="\"?";
   if (MessageBox(hDlg,prompt.c_str(),"Delete package:",MB_ICONQUESTION|MB_YESNO|MB_DEFBUTTON2)==IDYES)
    {//proceed
     string path=GetUserDataPath();
     path+='\\';
     path+=name;
     path+=".propertypackage";
     if (!DeleteFile(path.c_str()))
      {MessageBox(hDlg,"Failed to delete package.","Delete package:",MB_ICONHAND);
      }
     else
      {//update list
       int index=(int)SendDlgItemMessage(hDlg,IDC_PACKAGELIST,LB_GETCURSEL,0,0);
       SendDlgItemMessage(hDlg,IDC_PACKAGELIST,LB_DELETESTRING,index,0);
       EnableButtons();
      }
    }
  }
}

//! Edit an existing Property Package

void ThermoSystemEditor::OnEdit()
{string name=GetSelectedPackage();
 if (!name.empty())
  {PropertyPackage package;
   string path=GetUserDataPath();
   path+='\\';
   path+=name;
   path+=".propertypackage";
   if (!package.Load(path.c_str()))
    {string s;
     s="Failed to load package: ";
     s+=package.LastError();
     MessageBox(hDlg,s.c_str(),"Edit package:",MB_ICONHAND);
    }
   else
    {PackageEditor editor(&package);
     if (editor.Edit())
      {//save changes
       if (!package.Save(path.c_str()))
        {string s; 
         s="Failed to save changes: ";
         s+=package.LastError();
         MessageBox(hDlg,s.c_str(),"Edit package:",MB_ICONHAND);
        }
      }
    }
  }
}

//! Close the dialog

void ThermoSystemEditor::OnCancel()
{EndDialog(hDlg,IDCANCEL);
}

//! Package selection has changed

void ThermoSystemEditor::OnSelChangePackage()
{EnableButtons();
}

//! Enable / disable the buttons

void ThermoSystemEditor::EnableButtons()
{BOOL enabled=((int)SendDlgItemMessage(hDlg,IDC_PACKAGELIST,LB_GETCURSEL,0,0)>=0);
 EnableWindow(GetDlgItem(hDlg,IDC_EDIT),enabled); 
 EnableWindow(GetDlgItem(hDlg,IDC_RENAME),enabled); 
 EnableWindow(GetDlgItem(hDlg,IDC_DELETE),enabled); 
}

//! Get the selected package, returns empty string if nothing is selected

string ThermoSystemEditor::GetSelectedPackage()
{string res;
 int index=(int)SendDlgItemMessage(hDlg,IDC_PACKAGELIST,LB_GETCURSEL,0,0);
 if (index>=0)
  {int len=(int)SendDlgItemMessage(hDlg,IDC_PACKAGELIST,LB_GETTEXTLEN,index,0);
   len++;
   char *buf=new char[len];
   SendDlgItemMessage(hDlg,IDC_PACKAGELIST,LB_GETTEXT,index,(LPARAM)buf);
   res=buf;
   delete []buf;
  }
 return res;
}

