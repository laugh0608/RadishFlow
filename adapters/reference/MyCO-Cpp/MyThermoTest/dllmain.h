// dllmain.h: 模块类的声明。

class CMyThermoTestModule : public ATL::CAtlDllModuleT< CMyThermoTestModule >
{
public :
	DECLARE_LIBID(LIBID_MyThermoTestLib)
	DECLARE_REGISTRY_APPID_RESOURCEID(IDR_MYTHERMOTEST, "{b521d5a4-b34b-4c72-9b59-72a5e0d0139d}")
};

extern class CMyThermoTestModule _AtlModule;
