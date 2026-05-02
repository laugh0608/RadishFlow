// dllmain.h: 模块类的声明。

class CHeaterExampleModule : public ATL::CAtlDllModuleT< CHeaterExampleModule >
{
public :
	DECLARE_LIBID(LIBID_HeaterExampleLib)
	DECLARE_REGISTRY_APPID_RESOURCEID(IDR_HEATEREXAMPLE, "{a488d911-5b85-497a-80a4-2a085146bdc6}")
};

extern class CHeaterExampleModule _AtlModule;
