// EditDialog.h : Declaration of the CEditDialog

#pragma once
#include "resource.h"       // main symbols
#include <atlhost.h>

//! Simple edit dialog
/*!
  Implements a simple edit dialog for the Unit Operation. Shown from ICapeUtilities::Edit
*/

class CEditDialog : 
	public CAxDialogImpl<CEditDialog>
{
public:

	double *splitFactor; /*!< points to the location of the split factor being edited */
	double *heatInput; /*!< points to the location of the heat input being edited */
	OLECHAR buf[128]; /*!< text buffer for formatting values */

	//! Constructor.
    /*!
      Creates an Edit Dialog.
      \param splitFactor points to the location of the split factor being edited
      \param heatInput points to the location of the heat input being edited
    */

	CEditDialog(double *splitFactor,double *heatInput)
	{this->splitFactor=splitFactor;
	 this->heatInput=heatInput;
	}

	//! Resource identifier of the dialog
    /*!
      Resource identifier of the dialog; used by the CAxDialogImpl class to create the dialog
    */

	enum { IDD = IDD_EDITDIALOG };

	//! Message map

BEGIN_MSG_MAP(CEditDialog)
	MESSAGE_HANDLER(WM_INITDIALOG, OnInitDialog)
	COMMAND_HANDLER(IDOK, BN_CLICKED, OnClickedOK)
	COMMAND_HANDLER(IDCANCEL, BN_CLICKED, OnClickedCancel)
	COMMAND_HANDLER(IDC_SPLITFACTOR, EN_KILLFOCUS, OnEnKillfocusSplitfactor)
	COMMAND_HANDLER(IDC_HEATINPUT, EN_KILLFOCUS, OnEnKillfocusHeatinput)
	MESSAGE_HANDLER(WM_CLOSE, OnClose)
	CHAIN_MSG_MAP(CAxDialogImpl<CEditDialog>)
END_MSG_MAP()


	//! Called when the dialog needs to be initialized
    /*!
	  This function is called in response to WM_INITDIALOG. It initializes the values
	  in the dialog controls     
      \param uMsg WM_INITDIALOG
      \param wParam wParam as passed to WM_INITDIALOG
      \param lParam lParam as passed to WM_INITDIALOG
      \param bHandled set to TRUE if we process this message
      \return non-zero if the system will set the default focus
    */

	LRESULT OnInitDialog(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL& bHandled)
	{	CAxDialogImpl<CEditDialog>::OnInitDialog(uMsg, wParam, lParam, bHandled);
		bHandled = TRUE;
		//set split factor
		swprintf_s(buf,128,L"%lg",*splitFactor);
		SendDlgItemMessage(IDC_SPLITFACTOR,WM_SETTEXT,0,(LPARAM)buf);
		//set heat input
		swprintf_s(buf,128,L"%lg",*heatInput);
		SendDlgItemMessage(IDC_HEATINPUT,WM_SETTEXT,0,(LPARAM)buf);
		return 1;  // Let the system set the focus
	}

	//! Called when OK button is clicked or Enter is hit
    /*!
      Default implementation
	  \param wNotifyCode Notification code
      \param wID ID of control that triggered this call
      \param hWndCtl HWND of control that triggered this call
      \param bHandled set to TRUE if we process this message
      \return zero
    */

	LRESULT OnClickedOK(WORD wNotifyCode, WORD wID, HWND hWndCtl, BOOL& bHandled)
	{	EndDialog(wID);
		return 0;
	}

	//! Called when Escape is hit
    /*!
      Default implementation
	  \param wNotifyCode Notification code
      \param wID ID of control that triggered this call
      \param hWndCtl HWND of control that triggered this call
      \param bHandled set to TRUE if we process this message
      \return zero
    */

	LRESULT OnClickedCancel(WORD wNotifyCode, WORD wID, HWND hWndCtl, BOOL& bHandled)
	{	EndDialog(wID);
		return 0;
	}

	//! Called when split factor edit field loses focus
    /*!
      Used to interpret and update the data
	  \param wNotifyCode Notification code
      \param wID ID of control that triggered this call
      \param hWndCtl HWND of control that triggered this call
      \param bHandled set to TRUE if we process this message
      \return zero
    */

	LRESULT OnEnKillfocusSplitfactor(WORD wNotifyCode, WORD wID, HWND hWndCtl, BOOL& bHandled)
	{UpdateSplitFactor();
	 return 0;
	}

	//! Called when heat input edit field loses focus
    /*!
      Used to interpret and update the data
	  \param wNotifyCode Notification code
      \param wID ID of control that triggered this call
      \param hWndCtl HWND of control that triggered this call
      \param bHandled set to TRUE if we process this message
      \return zero
    */
	
	LRESULT OnEnKillfocusHeatinput(WORD wNotifyCode, WORD wID, HWND hWndCtl, BOOL& bHandled)
	{UpdateHeatInput();
	 return 0;
	}

	//! Called when dialog is about to close
    /*!
      Used to interpret and update the data
	  \param uMsg WM_CLOSE
      \param wParam wParam as passed to WM_CLOSE
      \param lParam lParam as passed to WM_CLOSE
      \param bHandled set to TRUE if we process this message
      \return zero
    */
	
	LRESULT OnClose(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL& bHandled)
	{UpdateSplitFactor();
	 UpdateHeatInput();
	 EndDialog(IDOK);
	 return 0;
	}

	//! Called to update split factor
    /*!
        Called in response to an update request. Will update the split factor data in the unit operation
        as well as in the edit field
    */
    
    void UpdateSplitFactor()
    {double newValue;
     //get the text from the control 
     SendDlgItemMessage(IDC_SPLITFACTOR,WM_GETTEXT,128,(LPARAM)buf);
     //intepret the text
     if (swscanf_s(buf,L"%lg",&newValue)!=1) newValue=0.5; //default value if not a number
     //put within limits
     if (newValue<0) newValue=0; else if (newValue>1.0) newValue=1.0;
     //update 
     *splitFactor=newValue;
     swprintf_s(buf,128,L"%lg",*splitFactor);
	 SendDlgItemMessage(IDC_SPLITFACTOR,WM_SETTEXT,0,(LPARAM)buf);
    }

	//! Called to update heat input
    /*!
        Called in response to an update request. Will update the heat input data in the unit operation
        as well as in the edit field
    */
    
    void UpdateHeatInput()
    {double newValue;
     //get the text from the control 
     SendDlgItemMessage(IDC_HEATINPUT,WM_GETTEXT,128,(LPARAM)buf);
     //intepret the text
     if (swscanf_s(buf,L"%lg",&newValue)!=1) newValue=0; //default value if not a number
     //update 
     *heatInput=newValue;
     swprintf_s(buf,128,L"%lg",*heatInput);
	 SendDlgItemMessage(IDC_HEATINPUT,WM_SETTEXT,0,(LPARAM)buf);
    }
	
};


