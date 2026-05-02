#include "StdAfx.h"
#include "IdealThermoModule.h"
#include "Compound.h"
#include "PackageEditor.h"
#include "PropertyPackage.h"
#include "resource.h"

extern HMODULE module;

//! Constructor
/*!
  Called upon construction of a PackageEditor instance
  \param package The property package being edited
  \sa InitDialog
*/

PackageEditor::PackageEditor(PropertyPackage *package)
{this->package=package;
}

//! Window procedure for PropertyPackage Edit dialog
/*!
  Dialog box procedure for the edit dialog. Handles all the relevant window messages
  \param hwndDlg Dialog window handle
  \param uMsg Message
  \param wParam wParam
  \param lParam lParam
  \sa PackageEditor
*/

INT_PTR CALLBACK EditWindowProc(HWND hwndDlg,UINT uMsg,WPARAM wParam,LPARAM lParam)
{PackageEditor *editor;
 switch (uMsg)
  {case WM_INITDIALOG:
        //the lParam contains the pointer to the PackageEditor object
        editor=(PackageEditor *)lParam;
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
        editor=(PackageEditor *)GetWindowLongPtr(hwndDlg,GWLP_USERDATA);
        //handle command
        switch (LOWORD(wParam))
         {case IDC_UP: 
               editor->OnUp();
               return TRUE;
          case IDC_DOWN: 
               editor->OnDown();
               return TRUE;
          case IDC_ADD:
               editor->OnAdd();
               return TRUE;
          case IDC_DELETE:
               editor->OnDelete();
               return TRUE;
          case IDOK:
               editor->OnOk();
               return TRUE;
          case IDCANCEL:
               editor->OnCancel();
               return TRUE;
          case IDC_COMPOUNDLIST:
               if (HIWORD(wParam)==LBN_SELCHANGE) 
                {editor->OnSelChangeCompound();
                 return TRUE;
                }
               break;
          case IDC_AVAILCOMPOUNDLIST:
               if (HIWORD(wParam)==LBN_SELCHANGE) 
                {editor->OnSelChangeAvailCompound();
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
  Shows the edit dialog. Returns true in case the user accepts the changes, false in case the user cancels.
*/

bool PackageEditor::Edit()
{//get parent window 
 HWND parentWindow=GetActiveWindow();
 if (!IsWindow(parentWindow)) parentWindow=NULL; //do not show as child of invalid window
 if (parentWindow) if (!IsWindowVisible(parentWindow)) parentWindow=NULL; //do not show as child of invisible window
 //create and show dialog
 INT_PTR nRes=DialogBoxParam(module,MAKEINTRESOURCE(IDD_EDITDIALOG),parentWindow,EditWindowProc,(LPARAM)this);
 //return true in case the changes were accepted
 return (nRes==IDOK);
}

//! Initialize the dialog

void PackageEditor::InitDialog()
{int i,j;
 //fill present compound list
 presentCompounds.resize(package->compounds.size());
 for (i=0;i<(int)package->compounds.size();i++) 
  {presentCompounds[i]=package->compounds[i]->name;
   SendDlgItemMessage(hDlg,IDC_COMPOUNDLIST,LB_ADDSTRING,0,(LPARAM)presentCompounds[i].c_str());
  }
 //fill available compound list (compounds on the system, but not in the package)
 vector<string> allCompounds;
 ListFiles(GetDataPath().c_str(),"compound",allCompounds);
 for (i=0;i<(int)allCompounds.size();i++)
  {//check if not present
   for (j=0;j<(int)presentCompounds.size();j++)
    if (lstrcmpi(presentCompounds[j].c_str(),allCompounds[i].c_str())==0)
     break;
   if (j==(int)presentCompounds.size())
    {//not present, add
     SendDlgItemMessage(hDlg,IDC_AVAILCOMPOUNDLIST,LB_ADDSTRING,0,(LPARAM)allCompounds[i].c_str());
    }
  }
 //init button status
 EnableButtons();
}

//! Enable / disable buttons

void PackageEditor::EnableButtons()
{int index;
 index=(int)SendDlgItemMessage(hDlg,IDC_COMPOUNDLIST,LB_GETCURSEL,0,0);
 EnableWindow(GetDlgItem(hDlg,IDC_UP),index>0);
 EnableWindow(GetDlgItem(hDlg,IDC_DOWN),(index>=0)&&(index<(int)presentCompounds.size()-1));
 EnableWindow(GetDlgItem(hDlg,IDC_DELETE),index>=0);
 index=(int)SendDlgItemMessage(hDlg,IDC_AVAILCOMPOUNDLIST,LB_GETCURSEL,0,0);
 EnableWindow(GetDlgItem(hDlg,IDC_ADD),index>=0);
}

//! Up button was clicked

void PackageEditor::OnUp()
{int i=(int)SendDlgItemMessage(hDlg,IDC_COMPOUNDLIST,LB_GETCURSEL,0,0);
 if (i>0)
  {string s;
   s=presentCompounds[i-1];
   presentCompounds[i-1]=presentCompounds[i];
   presentCompounds[i]=s;
   SendDlgItemMessage(hDlg,IDC_COMPOUNDLIST,LB_DELETESTRING,i-1,0);
   SendDlgItemMessage(hDlg,IDC_COMPOUNDLIST,LB_INSERTSTRING,i,(LPARAM)presentCompounds[i].c_str());
   SendDlgItemMessage(hDlg,IDC_COMPOUNDLIST,LB_SETCURSEL,i-1,0);
   EnableButtons();
  }
}

//! Down button was clicked

void PackageEditor::OnDown()
{int i=(int)SendDlgItemMessage(hDlg,IDC_COMPOUNDLIST,LB_GETCURSEL,0,0);
 if ((i>=0)&&(i<(int)presentCompounds.size()-1))
  {string s;
   s=presentCompounds[i+1];
   presentCompounds[i+1]=presentCompounds[i];
   presentCompounds[i]=s;
   SendDlgItemMessage(hDlg,IDC_COMPOUNDLIST,LB_DELETESTRING,i,0);
   SendDlgItemMessage(hDlg,IDC_COMPOUNDLIST,LB_INSERTSTRING,i+1,(LPARAM)presentCompounds[i+1].c_str());
   SendDlgItemMessage(hDlg,IDC_COMPOUNDLIST,LB_SETCURSEL,i+1,0);
   EnableButtons();
  }
}

//! Add button was clicked

void PackageEditor::OnAdd()
{int i=(int)SendDlgItemMessage(hDlg,IDC_AVAILCOMPOUNDLIST,LB_GETCURSEL,0,0);
 if (i>=0)
  {char name[MAX_PATH]; //guaranteed long enough, as this is the file name of the compound
   SendDlgItemMessage(hDlg,IDC_AVAILCOMPOUNDLIST,LB_GETTEXT,i,(LPARAM)name);
   SendDlgItemMessage(hDlg,IDC_AVAILCOMPOUNDLIST,LB_DELETESTRING,i,0);
   presentCompounds.push_back(name);
   i=(int)SendDlgItemMessage(hDlg,IDC_COMPOUNDLIST,LB_ADDSTRING,0,(LPARAM)name);
   SendDlgItemMessage(hDlg,IDC_COMPOUNDLIST,LB_SETCURSEL,i,0);
   EnableButtons();
  }
}

//! Delete button was clicked

void PackageEditor::OnDelete()
{int i=(int)SendDlgItemMessage(hDlg,IDC_COMPOUNDLIST,LB_GETCURSEL,0,0);
 if (i>=0)
  {int j=(int)SendDlgItemMessage(hDlg,IDC_AVAILCOMPOUNDLIST,LB_ADDSTRING,0,(LPARAM)presentCompounds[i].c_str());
   SendDlgItemMessage(hDlg,IDC_AVAILCOMPOUNDLIST,LB_SETCURSEL,j,0);
   SendDlgItemMessage(hDlg,IDC_COMPOUNDLIST,LB_DELETESTRING,i,0);
   presentCompounds.erase(presentCompounds.begin()+i);
   EnableButtons();
  }
}

//! OK button was clicked

void PackageEditor::OnOk()
{//check the list of present compounds
 if (presentCompounds.size()==0)
  {MessageBox(hDlg,"At least one compound must be present","Error:",MB_ICONHAND);
   return;
  }
 //ok, load compounds and add to PP
 int i;
 vector<Compound *> newCompounds;
 for (i=0;i<(int)presentCompounds.size();i++)
  {Compound *comp=new Compound();
   newCompounds.push_back(comp);
   string error;
   if (!comp->Load(presentCompounds[i].c_str(),error))
    {//failed to load, do not proceed
     string s;
     s="Unable to load compound \"";
     s+=presentCompounds[i];
     s+="\": ";
     s+=error;
     MessageBox(hDlg,s.c_str(),"Error:",MB_ICONHAND);
     for (i=0;i<(int)newCompounds.size();i++) delete newCompounds[i];
     return;
    }
  }
 //all compounds loaded ok, replace compounds in package
 for (i=0;i<(int)package->compounds.size();i++) delete package->compounds[i];
 package->compounds=newCompounds;
 EndDialog(hDlg,IDOK);
}

//! Cancel button was clicked

void PackageEditor::OnCancel()
{EndDialog(hDlg,IDCANCEL);
}

//! Selection changed in present compound list

void PackageEditor::OnSelChangeCompound()
{EnableButtons();
}

//! Selection changed in available compound list

void PackageEditor::OnSelChangeAvailCompound()
{EnableButtons();
}
