// BlockPort.h: CBlockPort 的声明

#pragma once
#include "resource.h"       // 主符号



#include "MyBlockTest_i.h"

// 为端口的名字和描述添加string支持和BSTR类型支持
#include <string>
#include "Variant.h"
// 添加string转const OLECCHAR*类型支持
#include <atlbase.h>
#include <atlconv.h>



#if defined(_WIN32_WCE) && !defined(_CE_DCOM) && !defined(_CE_ALLOW_SINGLE_THREADED_OBJECTS_IN_MTA)
#error "Windows CE 平台(如不提供完全 DCOM 支持的 Windows Mobile 平台)上无法正确支持单线程 COM 对象。定义 _CE_ALLOW_SINGLE_THREADED_OBJECTS_IN_MTA 可强制 ATL 支持创建单线程 COM 对象实现并允许使用其单线程 COM 对象实现。rgs 文件中的线程模型已被设置为“Free”，原因是该模型是非 DCOM Windows CE 平台支持的唯一线程模型。"
#endif

using namespace ATL;


// CBlockPort

class ATL_NO_VTABLE CBlockPort :
	public CComObjectRootEx<CComSingleThreadModel>,
	public CComCoClass<CBlockPort, &CLSID_BlockPort>,
	public IDispatchImpl<IBlockPort, &IID_IBlockPort, &LIBID_MyBlockTestLib, /*wMajor =*/ 1, /*wMinor =*/ 0>,
	public IDispatchImpl<ICapeUnitPort, &__uuidof(ICapeUnitPort), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 0>,
	public IDispatchImpl<ICapeIdentification, &__uuidof(ICapeIdentification), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 0>
{

private:
	// 创建一个物流对象实例
	//IDispatch* myMaterialObject;
	ICapeThermoMaterial* myMaterialObject;
	//CComPtr<ICapeThermoMaterial> myMaterialObject;
	// 创建一个端口方向的实例
	CapePortDirection direction;
	// 创建端口的名字和描述
	std::wstring name;
	std::wstring description;

public:
	CBlockPort()
	{
		// 给创建的物流对象一个初始值
		myMaterialObject = NULL;
	}

	// 获取物流对象，传给MyBlockPortsArray中的getInlet()和getOutlet()函数
	//IDispatch*& getMaterial()
	ICapeThermoMaterial*& getMaterial()
	//CComPtr<ICapeThermoMaterial>& getMaterial()
	{
		return myMaterialObject;
	}

	// 单独创建一个函数给myPortDirection传值
	void SetDirection(CapePortDirection direction)
	{
		// 给定创建的端口一个端口方向的值
		this->direction = direction;
	}

	// 创建一个函数给端口名字和描述传值
	void SetNamAndDes(std::wstring name, std::wstring description)
	{
		this->name = name;
		this->description = description;
	}


DECLARE_REGISTRY_RESOURCEID(110)


BEGIN_COM_MAP(CBlockPort)
	COM_INTERFACE_ENTRY(IBlockPort)
	COM_INTERFACE_ENTRY2(IDispatch, ICapeUnitPort)
	COM_INTERFACE_ENTRY(ICapeUnitPort)
	COM_INTERFACE_ENTRY(ICapeIdentification)
END_COM_MAP()



	DECLARE_PROTECT_FINAL_CONSTRUCT()

	HRESULT FinalConstruct()
	{
		return S_OK;
	}

	void FinalRelease()
	{
	}

public:




// ICapeUnitPort Methods
public:
	STDMETHOD(get_portType)(CapePortType *portType)
	{
		// 该端口1是一个物流对象端口
		*portType = CapePortType::CAPE_MATERIAL;

		return S_OK;
	}

	STDMETHOD(get_direction)(CapePortDirection *direction)
	{
		// 该端口1是一个单元模块入口
		//*direction = CapePortDirection::CAPE_INLET;
		// 获取单元模块方向参数
		*direction = this->direction;

		return S_OK;
	}

	STDMETHOD(get_connectedObject)(LPDISPATCH *connectedObject)
	{
		//*connectedObject = NULL;
		// 该端口连接到到创建的物流对象
		*connectedObject = myMaterialObject;
		//(*connectedObject)->AddRef();

		return S_OK;
	}

	STDMETHOD(Connect)(LPDISPATCH objectToConnect)
	{
		//myMaterialObject = objectToConnect;
		// 端口连接进行一个搜索，取到创建的物流对象
		//objectToConnect->QueryInterface(IID_IDispatch, (LPVOID*)&myMaterialObject);
		objectToConnect->QueryInterface(IID_ICapeThermoMaterial, (LPVOID*)&myMaterialObject);

		return S_OK;
	}

	STDMETHOD(Disconnect)()
	{
		// 断开状态就是获取的物流对象为空
		myMaterialObject = NULL;

		return S_OK;
	}


// ICapeIdentification Methods
public:
	STDMETHOD(get_ComponentName)(BSTR *componentName)
	{
		// 获取端口的名字
		//CBSTR nam(SysAllocString(CA2W(blockPortName.c_str())));	// string转const OLECHAR*类型
		//*ComponentName = nam;
		*componentName = SysAllocString(name.c_str());

		return S_OK;
	}

	STDMETHOD(put_ComponentName)(BSTR pszComponentName)
	{
		// 禁止自动设置，直接返回状态码
		return S_OK;
	}

	STDMETHOD(get_ComponentDescription)(BSTR *componentDescription)
	{
		// 获取端口的描述
		//CBSTR des(SysAllocString(CA2W(blockPortDescription.c_str())));	// string转const OLECHAR*类型
		//*ComponentDescription = des;
		*componentDescription = SysAllocString(description.c_str());

		return S_OK;
	}

	STDMETHOD(put_ComponentDescription)(BSTR pszComponentDescription)
	{
		// 禁止自动设置，直接返回状态码
		return S_OK;
	}

};

OBJECT_ENTRY_AUTO(__uuidof(BlockPort), CBlockPort)
