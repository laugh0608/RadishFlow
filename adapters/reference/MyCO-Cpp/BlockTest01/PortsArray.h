// PortsArray.h: CPortsArray 的声明

#pragma once
#include "resource.h"       // 主符号

#include "MaterialPort.h"

#include "BlockTest01_i.h"



#if defined(_WIN32_WCE) && !defined(_CE_DCOM) && !defined(_CE_ALLOW_SINGLE_THREADED_OBJECTS_IN_MTA)
#error "Windows CE 平台(如不提供完全 DCOM 支持的 Windows Mobile 平台)上无法正确支持单线程 COM 对象。定义 _CE_ALLOW_SINGLE_THREADED_OBJECTS_IN_MTA 可强制 ATL 支持创建单线程 COM 对象实现并允许使用其单线程 COM 对象实现。rgs 文件中的线程模型已被设置为“Free”，原因是该模型是非 DCOM Windows CE 平台支持的唯一线程模型。"
#endif

using namespace ATL;


// CPortsArray

class ATL_NO_VTABLE CPortsArray :
	public CComObjectRootEx<CComSingleThreadModel>,
	public CComCoClass<CPortsArray, &CLSID_PortsArray>,
	public IDispatchImpl<IPortsArray, &IID_IPortsArray, &LIBID_BlockTest01Lib, /*wMajor =*/ 1, /*wMinor =*/ 0>,
	public IDispatchImpl<ICapeCollection, &__uuidof(ICapeCollection), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>,
	public IDispatchImpl<ICapeIdentification, &__uuidof(ICapeIdentification), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>
{

private:
	// 创建一个端口实例
	CComObject<CMaterialPort> *port1;
	// 增加一个端口实例
	CComObject<CMaterialPort> *port2;

public:
	// 通过热力学接口 ICapeThermoMaterial 调用端口连接的物流对象
	// 传统方法，版本 1.1
	//ICapeThermoMaterial* &getInlet() {
	// 变化 4，智能指针
	ICapeThermoMaterial* getInlet() {
		return (ICapeThermoMaterial*)port1->getMaterial();
	}

	/*void getInlet(ICapeThermoMaterial* &tm) {
		tm = (ICapeThermoMaterial*)port1->getMaterial();
	}*/

	// 传统方法，版本 1.1
	//ICapeThermoMaterial* &getOutlet() {
	// 变化 4，智能指针
	ICapeThermoMaterial* getOutlet() {
		return (ICapeThermoMaterial*)port2->getMaterial();
	}

	CPortsArray()
	{
		// 实例化创建的端口
		CComObject<CMaterialPort>::CreateInstance(&port1);
		port1->AddRef();
		// 设置端口方向
		port1->SetDirection(CapePortDirection::CAPE_INLET);
		// 设置端口名字和描述
		//port1->SetNameAndDesc("INLET", "PORT1");
		port1->SetNameAndDesc(L"INLET", L"PORT1");
		// 实例化创建的端口
		CComObject<CMaterialPort>::CreateInstance(&port2);
		port2->AddRef();
		// 设置端口方向
		port2->SetDirection(CapePortDirection::CAPE_OUTLET);
		// 设置端口名字和描述
		//port2->SetNameAndDesc("OUTLET", "PORT2");
		port2->SetNameAndDesc(L"OUTLET", L"PORT2");
	}

DECLARE_REGISTRY_RESOURCEID(107)


BEGIN_COM_MAP(CPortsArray)
	COM_INTERFACE_ENTRY(IPortsArray)
	COM_INTERFACE_ENTRY2(IDispatch, ICapeCollection)
	COM_INTERFACE_ENTRY(ICapeCollection)
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




// ICapeCollection Methods
public:
	STDMETHOD(Item)(VARIANT id, LPDISPATCH *pItem)
	{
		//// 获取 id
		//CVariant v(id, TRUE);
		//wstring error;
		//// 如果 id 是整数数组，则给端口 port 赋值
		//if (v.CheckArray(VT_I4, error)) {
		//	if (v.GetLongAt(0) == 0) {
		//		// 给实例化好的端口进行赋值
		//		port1->QueryInterface(IID_IDispatch, (LPVOID*)pItem);
		//	} else {
		//		port2->QueryInterface(IID_IDispatch, (LPVOID*)pItem);
		//	}
		//// 如果 id 是字符串数组，则给端口 port 赋值
		//} else if (v.CheckArray(VT_BSTR,error)) {
		//	CBSTR name = v.GetStringAt(0);
		//	if (CBSTR::Same(L"INLET", name)) {
		//		port1->QueryInterface(IID_IDispatch, (LPVOID*)pItem);
		//	}
		//	else {
		//		port2->QueryInterface(IID_IDispatch, (LPVOID*)pItem);
		//	}
		//}
		
		// 获取 id
		if (id.vt == VT_I4)	// 判断ID是个整数类型
		{
			if (id.lVal == 1)
			{
				// 端口1赋值
				port1->QueryInterface(IID_IDispatch, (LPVOID*)pItem);
				port1->AddRef();
			}
			else
			{
				// 端口2赋值
				port2->QueryInterface(IID_IDispatch, (LPVOID*)pItem);
				port2->AddRef();
			}
		}
		
		// 给实例化好的端口进行赋值
		//port1->QueryInterface(IID_IDispatch, (LPVOID*)pItem);

		return S_OK;
	}

	STDMETHOD(Count)(long *portNum)
	{
		// 端口数
		*portNum = 2;
		//*portNum = 1;

		return S_OK;
	}


// ICapeIdentification Methods
public:
	STDMETHOD(get_ComponentName)(BSTR *pComponentName)
	{
		// 获取端口数组的名字
		//CBSTR n(SysAllocString(CA2W("Ports Array Name")));	// string 转 const OLECHAR* 类型
		CBSTR n(SysAllocString(L"Ports Array Name"));	// string 转 const OLECHAR* 类型
		*pComponentName = n;

		return S_OK;
	}

	STDMETHOD(put_ComponentName)(BSTR pszComponentName)
	{
		// 不做实现，返回空结果

		return S_OK;
	}

	STDMETHOD(get_ComponentDescription)(BSTR *pComponentDesc)
	{
		// 获取端口数组的描述
		//CBSTR d(SysAllocString(CA2W("Ports Array Desc")));	// string 转 const OLECHAR* 类型
		CBSTR d(SysAllocString(L"Ports Array Desc"));	// string 转 const OLECHAR* 类型
		*pComponentDesc = d;

		return S_OK;
	}

	STDMETHOD(put_ComponentDescription)(BSTR pszComponentDesc)
	{
		// 不做实现，返回空结果

		return S_OK;
	}

};

OBJECT_ENTRY_AUTO(__uuidof(PortsArray), CPortsArray)
