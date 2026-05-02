#pragma once

//! ThermoSystemEditor class
/*!

	This class wraps the Edit dialog for the collection of Property Packages
	known to the Ideal Thermo Module. The edit dialog itself does not use any support 
	libraries, but an old-fashioned DialogBoxProc.
	
	The IDD_THERMOSYSTEMDIALOG dialog resource is used. Notice that it has the 
	Visible flag set so that also in hidden applications the window is forced to be 
	visible.
	
	This edit dialog is not associated with a Property Package, but rather to the 
	global collection of Property Packages available at the system. As such, it 
	can be called from a ICapeThermoSystem's or ICapePropertyPackageManager's 
	edit routine, or it can be called from a stand-alone application.
  
*/

class ThermoSystemEditor
{private:

 HWND hDlg; /*!< Window handle of the dialog */
 void InitDialog();
 void OnNew();
 void OnDelete();
 void OnEdit();
 void OnRename();
 void OnCancel();
 void OnSelChangePackage();
 void EnableButtons();
 string GetSelectedPackage();
 friend INT_PTR CALLBACK ThermoSystemEditorWindowProc(HWND hwndDlg,UINT uMsg,WPARAM wParam,LPARAM lParam);

 public:
 
 ThermoSystemEditor();
 void Edit();
 
};
