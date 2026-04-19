#include "StdAfx.h"
#include "EditBox.h"
#include "resource.h"

extern HMODULE module;

//! Constructor
/*!
  Called upon construction of a EditBox instance
  \param title Title of the Edit box
  \param caption Text shown above the edit field
  \param defaultValue Default text, optional
  \sa InitDialog()
*/

EditBox::EditBox(const char *title,const char *caption,const char *defaultValue)
{this->title=title;
 this->caption=caption;
 if (defaultValue) value=defaultValue;
}

//! Constructor
/*!
  Called upon construction of a EditBox instance
  \param parent Parent window handle
  \return True in case the user has clicked OK, False in case of Cancel
*/

bool EditBox::Edit(HWND parent)
{return (DialogBoxParam(module,MAKEINTRESOURCE(IDD_EDITBOX),parent,EditBoxProc,(LPARAM)this)==IDOK);
}

//! Result
/*!
  Called to get the result of the dialog. Should be called only if Edit() returned true
  \return String entered by the user
*/

string EditBox::Result()
{return value;
}

//! Result
/*!
  Dialog box procedure for the EditBox dialog. Handles all the relevant window messages
  \param hwndDlg Dialog window handle
  \param uMsg Message
  \param wParam wParam
  \param lParam lParam
  \sa EditBox
*/

INT_PTR CALLBACK EditBoxProc(HWND hwndDlg,UINT uMsg,WPARAM wParam,LPARAM lParam)
{EditBox *editor;
 switch (uMsg)
  {case WM_INITDIALOG:
        //the lParam contains the pointer to the PackageEditor object
        editor=(EditBox *)lParam;
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
        editor=(EditBox *)GetWindowLongPtr(hwndDlg,GWLP_USERDATA);
        //handle command
        switch (LOWORD(wParam))
         {case IDOK: 
               editor->OnOK();
               return TRUE;
          case IDCANCEL: 
               editor->OnCancel();
               return TRUE;
         }
        break;
  }
 return FALSE;
}

//! Initialize the dialog

void EditBox::InitDialog()
{SetWindowText(hDlg,title.c_str());
 SetDlgItemText(hDlg,IDC_CAPTION,caption.c_str());
 SetDlgItemText(hDlg,IDC_EDIT,value.c_str());
}

//! OK has been clicked

void EditBox::OnOK()
{HWND edit=GetDlgItem(hDlg,IDC_EDIT);
 int len=GetWindowTextLength(edit);
 len++;
 char *buf=new char[len];
 GetWindowText(edit,buf,len);
 value=buf;
 delete []buf;
 EndDialog(hDlg,IDOK);
}

//! Cancel has been clicked

void EditBox::OnCancel()
{EndDialog(hDlg,IDCANCEL);
}

