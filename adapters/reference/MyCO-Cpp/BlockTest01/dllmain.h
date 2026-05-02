// dllmain.h: 模块类的声明。

class CBlockTest01Module : public ATL::CAtlDllModuleT< CBlockTest01Module >
{
public :
	DECLARE_LIBID(LIBID_BlockTest01Lib)
	DECLARE_REGISTRY_APPID_RESOURCEID(IDR_BLOCKTEST01, "{5fe4e419-f915-44f5-9af0-789285807df1}")
};

extern class CBlockTest01Module _AtlModule;
