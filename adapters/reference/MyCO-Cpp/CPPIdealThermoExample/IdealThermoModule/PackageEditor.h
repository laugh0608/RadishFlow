#pragma once

class PropertyPackage;

//! PackageEditor class
/*!

	This class wraps the Property Package Edit dialog and stores the information
	required during editing. The edit dialog itself does not use any support 
	libraries, but an old-fashioned DialogBoxProc.
	
	The IDD_EDITDIALOG dialog resource is used. Notice that it has the Visible
	flag set so that also in hidden applications the window is forced to be 
	visible.
	
	\sa PropertyPackage
  
*/


class PackageEditor
{private:

 vector<string> presentCompounds; /*!< List of present compounds */
 PropertyPackage *package; /*!< Reference to the property package being edited */
 HWND hDlg; /*!< Window handle of the dialog */
 void InitDialog();
 void OnUp();
 void OnDown();
 void OnAdd();
 void OnDelete();
 void OnOk();
 void OnCancel();
 void OnSelChangeCompound();
 void OnSelChangeAvailCompound();
 void EnableButtons();
 friend INT_PTR CALLBACK EditWindowProc(HWND hwndDlg,UINT uMsg,WPARAM wParam,LPARAM lParam);

 public:

 PackageEditor(PropertyPackage *package);
 bool Edit();

};
