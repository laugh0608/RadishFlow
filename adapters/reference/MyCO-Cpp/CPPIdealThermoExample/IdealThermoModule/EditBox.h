#pragma once

//! EditBox class 
/*!

	This class wraps the Edit Box dialog and stores the information
	required during editing. The edit dialog itself does not use any support 
	libraries, but an old-fashioned DialogBoxProc.
	
	The IDD_EDITBOX dialog resource is used. 
	
	The purpose of the Edit box is merely to edit a string value.
	
	\sa PropertyPackage
  
*/

class EditBox
{	private:
    string value; /*!< Default value upon construction, user value upon OK */
    string title; /*!< Title of the dialog */
    string caption; /*!< Text above the edit field */
    HWND hDlg; /*!< Dialog window handle */
    friend INT_PTR CALLBACK EditBoxProc(HWND hwndDlg,UINT uMsg,WPARAM wParam,LPARAM lParam);
    void InitDialog();
    void OnOK();
    void OnCancel();

	public:
	EditBox(const char *title,const char *caption,const char *defaultValue=NULL);
	bool Edit(HWND parent);
	string Result();

};
