// dllmain.h: 模块类的声明。

class CMyBlockTestModule : public ATL::CAtlDllModuleT< CMyBlockTestModule >
{
public :
	DECLARE_LIBID(LIBID_MyBlockTestLib)
	DECLARE_REGISTRY_APPID_RESOURCEID(IDR_MYBLOCKTEST, "{abd5105a-8163-475f-adac-a66891f98448}")
};

extern class CMyBlockTestModule _AtlModule;
