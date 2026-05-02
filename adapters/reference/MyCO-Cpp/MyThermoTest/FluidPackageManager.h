// FluidPackageManager.h: CFluidPackageManager 的声明

#pragma once
#include "resource.h"       // 主符号



#include "MyThermoTest_i.h"

// 添加物性包管理者头返回的物性包列表
#include "Variant.h"
// 添加物流对象模板
#include "MaterialObject.h"



#if defined(_WIN32_WCE) && !defined(_CE_DCOM) && !defined(_CE_ALLOW_SINGLE_THREADED_OBJECTS_IN_MTA)
#error "Windows CE 平台(如不提供完全 DCOM 支持的 Windows Mobile 平台)上无法正确支持单线程 COM 对象。定义 _CE_ALLOW_SINGLE_THREADED_OBJECTS_IN_MTA 可强制 ATL 支持创建单线程 COM 对象实现并允许使用其单线程 COM 对象实现。rgs 文件中的线程模型已被设置为“Free”，原因是该模型是非 DCOM Windows CE 平台支持的唯一线程模型。"
#endif

using namespace ATL;


// CFluidPackageManager

class ATL_NO_VTABLE CFluidPackageManager :
	public CComObjectRootEx<CComSingleThreadModel>,
	public CComCoClass<CFluidPackageManager, &CLSID_FluidPackageManager>,
	public IDispatchImpl<IFluidPackageManager, &IID_IFluidPackageManager, &LIBID_MyThermoTestLib, /*wMajor =*/ 1, /*wMinor =*/ 0>,
	public IDispatchImpl<ICapeThermoPropertyPackageManager, &__uuidof(ICapeThermoPropertyPackageManager), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 0>
{
public:
	CFluidPackageManager()
	{
	}

DECLARE_REGISTRY_RESOURCEID(106)


BEGIN_COM_MAP(CFluidPackageManager)
	COM_INTERFACE_ENTRY(IFluidPackageManager)
	COM_INTERFACE_ENTRY2(IDispatch, ICapeThermoPropertyPackageManager)
	COM_INTERFACE_ENTRY(ICapeThermoPropertyPackageManager)
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




// ICapeThermoPropertyPackageManager Methods
public:
	// 获取物性包列表
	STDMETHOD(GetPropertyPackageList)(VARIANT *GetPropertyPackageList)
	{
		// 手动创建一个物性集列表
		CVariant myThermoList;		// 创建一个数组
		myThermoList.MakeArray(2, VT_BSTR);	// 定义数组长度：2，类型：字符串
		myThermoList.AllocStringAt(0, L"PR set");	// 数组赋值，类型：宽字符串
		myThermoList.AllocStringAt(1, L"SRK set");

		// 返回上述创建好的列表
		*GetPropertyPackageList = myThermoList.ReturnValue();
		
		// 返回成功状态码
		//return E_NOTIMPL;
		return S_OK;
	}

	// 选择物性集列表中的一个
	STDMETHOD(GetPropertyPackage)(BSTR PackageName, LPDISPATCH *GetPropertyPackage)
	{
		// 为了方便理解和学习，上述我们强制用代码创建了一个列表
		// 故为了成功调用，需要对 PackageName 进行一个转换
		CBSTR myThermoName(PackageName);
		// 匹配调用
		if (myThermoName.Same(L"PR set"))	// 根据 BSTR.h 文件中的规则进行匹配，选择我们创建的 PR set
		{
			// 创建物流对象实例
			CComObject<CMaterialObject>* p;
			CComObject<CMaterialObject>::CreateInstance(&p);

			// 给实例写入对应的信息，信息列表模板在 MaterialObject.h 文件中已进行声明
			p->myEos = "PR";	// 使用什么状态方程
			// 组分列表是通过文件读取的，这里为了方便学习理解，直接使用代码指定
			p->myCompounds.push_back(L"111");	// H2O
			p->myCompounds.push_back(L"222");	// CO2

			// p 转换为 IDispatch 对象，MaterialObject（ATL对象）从此接口继承
			// 用 GetPropertyPackage 指针指向 p （状态方程以及组分信息）
			p->QueryInterface(IID_IDispatch, (LPVOID*)GetPropertyPackage);

			return S_OK;
		}
		else if (myThermoName.Same(L"SRK set"))
		{
			// 同上
			CComObject<CMaterialObject>* p;
			CComObject<CMaterialObject>::CreateInstance(&p);
			
			p->myEos = "SRK";
			p->myCompounds.push_back(L"333");
			p->myCompounds.push_back(L"444");
			p->myCompounds.push_back(L"555");

			p->QueryInterface(IID_IDispatch, (LPVOID*)GetPropertyPackage);

			return S_OK;
		}
		else return E_FAIL; // 如果都不匹配，返回调用失败

		// 在 if 语句中已返回状态码
		//return E_NOTIMPL;
	}

};

OBJECT_ENTRY_AUTO(__uuidof(FluidPackageManager), CFluidPackageManager)
