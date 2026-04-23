// MyBlockPortsArray.h: CMyBlockPortsArray 的声明

#pragma once
#include "resource.h"       // 主符号



#include "MyBlockTest_i.h"

// 引入单元模块端口文件
#include "BlockPort.h"



#if defined(_WIN32_WCE) && !defined(_CE_DCOM) && !defined(_CE_ALLOW_SINGLE_THREADED_OBJECTS_IN_MTA)
#error "Windows CE 平台(如不提供完全 DCOM 支持的 Windows Mobile 平台)上无法正确支持单线程 COM 对象。定义 _CE_ALLOW_SINGLE_THREADED_OBJECTS_IN_MTA 可强制 ATL 支持创建单线程 COM 对象实现并允许使用其单线程 COM 对象实现。rgs 文件中的线程模型已被设置为“Free”，原因是该模型是非 DCOM Windows CE 平台支持的唯一线程模型。"
#endif

using namespace ATL;


// CMyBlockPortsArray

class ATL_NO_VTABLE CMyBlockPortsArray :
	public CComObjectRootEx<CComSingleThreadModel>,
	public CComCoClass<CMyBlockPortsArray, &CLSID_MyBlockPortsArray>,
	public IDispatchImpl<IMyBlockPortsArray, &IID_IMyBlockPortsArray, &LIBID_MyBlockTestLib, /*wMajor =*/ 1, /*wMinor =*/ 0>,
	public IDispatchImpl<ICapeCollection, &__uuidof(ICapeCollection), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 0>,
	public IDispatchImpl<ICapeIdentification, &__uuidof(ICapeIdentification), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 0>
{

// 方便理解，简化学习Item()函数，直接在这里定义端口
private:
	CComObject<CBlockPort>* port1;	// 创建端口1实例
	CComObject<CBlockPort>* port2;	// 创建端口2实例


public:
	// 通过热力学接口ICapeThermoMaterial调用BlockPort中的物流对象
	ICapeThermoMaterial*& getInlet()
	{
		return (ICapeThermoMaterial*)port1->getMaterial();
	}
	ICapeThermoMaterial*& getOutlet()
	{
		return (ICapeThermoMaterial*)port2->getMaterial();
	}

	CMyBlockPortsArray()
	{
		// 实例化端口1，2
		CComObject<CBlockPort>::CreateInstance(&port1);
		port1->AddRef();
		CComObject<CBlockPort>::CreateInstance(&port2);
		port2->AddRef();
		// 定义端口方向并传给BlockPort中的SetDirection()函数
		port1->SetDirection(CapePortDirection::CAPE_INLET);		// 端口1为进口
		port2->SetDirection(CapePortDirection::CAPE_OUTLET);	// 端口2为出口
		// 定义端口的名字和描述并传给BlockPort中的SetNamAndDes()函数
		port1->SetNamAndDes(L"Inlet", L"Inlet Port");
		port2->SetNamAndDes(L"Outlet", L"Outlet Port");
	}


DECLARE_REGISTRY_RESOURCEID(109)


BEGIN_COM_MAP(CMyBlockPortsArray)
	COM_INTERFACE_ENTRY(IMyBlockPortsArray)
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
	STDMETHOD(Item)(VARIANT id, LPDISPATCH *item)
	{
		// 给端口1赋值
		//port1->QueryInterface(IID_IDispatch, (LPVOID*)Item);

		// 获取端口的逻辑，现在并不知道端口的id是什么，所以需要判断是整数还是端口的名字
		if(id.vt == VT_I4)	// 判断ID是个整数类型
		{
			if (id.lVal == 1)
			{
				// 端口1赋值
				port1->QueryInterface(IID_IDispatch, (LPVOID*)item);
				port1->AddRef();
			}
			else
			{
				// 端口2赋值
				port2->QueryInterface(IID_IDispatch, (LPVOID*)item);
				port2->AddRef();
			}
		}
		else if(id.vt == VT_BSTR)	// 判断ID是个字符串
		{
			if (CBSTR::Same(L"Inlet", id.bstrVal))	// 上面public部分设置的字段
			{
				port1->QueryInterface(IID_IDispatch, (LPVOID*)item);
				port1->AddRef();
			}
			else
			{
				port2->QueryInterface(IID_IDispatch, (LPVOID*)item);
				port2->AddRef();
			}
		}

		return S_OK;
	}

	STDMETHOD(Count)(long *myCountNum)
	{
		// 模块的端口数
		*myCountNum = 2;

		return S_OK;
	}


// ICapeIdentification Methods
public:
	STDMETHOD(get_ComponentName)(BSTR *componentName)
	{
		// 获取端口数组的名字
		//CBSTR nam(SysAllocString(CA2W("My Block Ports Array")));	// string转const OLECHAR*类型
		CBSTR nam(SysAllocString(L"My Block Ports Array"));
		*componentName = nam;

		return S_OK;
	}

	STDMETHOD(put_ComponentName)(BSTR pszComponentName)
	{
		// 禁止自动设置，直接返回状态码
		return S_OK;
	}

	STDMETHOD(get_ComponentDescription)(BSTR *componentDescription)
	{
		// 获取端口数组的描述
		//CBSTR des(SysAllocString(CA2W("My Block Ports Array Description")));	// string转const OLECHAR*类型
		CBSTR des(SysAllocString(L"My Block Ports Array Description"));
		*componentDescription = des;

		return S_OK;
	}

	STDMETHOD(put_ComponentDescription)(BSTR pszComponentDescription)
	{
		// 禁止自动设置，直接返回状态码
		return S_OK;
	}

};

OBJECT_ENTRY_AUTO(__uuidof(MyBlockPortsArray), CMyBlockPortsArray)
