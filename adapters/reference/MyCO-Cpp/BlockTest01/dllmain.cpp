// dllmain.cpp: DllMain 的实现。

#include "pch.h"
#include "framework.h"
#include "resource.h"
#include "BlockTest01_i.h"
#include "dllmain.h"
#include "xdlldata.h"

CBlockTest01Module _AtlModule;

class CBlockTest01App : public CWinApp
{
public:

// 重写
	virtual BOOL InitInstance();
	virtual int ExitInstance();

	DECLARE_MESSAGE_MAP()
};

BEGIN_MESSAGE_MAP(CBlockTest01App, CWinApp)
END_MESSAGE_MAP()

CBlockTest01App theApp;

BOOL CBlockTest01App::InitInstance()
{
#ifdef _MERGE_PROXYSTUB
	if (!PrxDllMain(m_hInstance, DLL_PROCESS_ATTACH, nullptr))
		return FALSE;
#endif
	return CWinApp::InitInstance();
}

int CBlockTest01App::ExitInstance()
{
	return CWinApp::ExitInstance();
}
