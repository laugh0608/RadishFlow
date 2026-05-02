// MyBlock.h: CMyBlock 的声明

#pragma once
#include "resource.h"       // 主符号



#include "MyBlockTest_i.h"

// 添加单元模块端口数组
#include "MyBlockPortsArray.h"
// 添加BSTR、Variant类型支持
#include "Variant.h"
// 添加ParameterArray数组，给ICapeUnit接口的get_ports函数返回参数
#include "MyParameterArray.h"
#include <string>



#if defined(_WIN32_WCE) && !defined(_CE_DCOM) && !defined(_CE_ALLOW_SINGLE_THREADED_OBJECTS_IN_MTA)
#error "Windows CE 平台(如不提供完全 DCOM 支持的 Windows Mobile 平台)上无法正确支持单线程 COM 对象。定义 _CE_ALLOW_SINGLE_THREADED_OBJECTS_IN_MTA 可强制 ATL 支持创建单线程 COM 对象实现并允许使用其单线程 COM 对象实现。rgs 文件中的线程模型已被设置为“Free”，原因是该模型是非 DCOM Windows CE 平台支持的唯一线程模型。"
#endif

using namespace ATL;


// CMyBlock

class ATL_NO_VTABLE CMyBlock :
	public CComObjectRootEx<CComSingleThreadModel>,
	public CComCoClass<CMyBlock, &CLSID_MyBlock>,
	public IDispatchImpl<IMyBlock, &IID_IMyBlock, &LIBID_MyBlockTestLib, /*wMajor =*/ 1, /*wMinor =*/ 0>,
	public IDispatchImpl<ICapeUnit, &__uuidof(ICapeUnit), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 0>,
	public IDispatchImpl<ICapeUtilities, &__uuidof(ICapeUtilities), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 0>,
	public IDispatchImpl<ICapeIdentification, &__uuidof(ICapeIdentification), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 0>
{

// 创建一个全局端口数的实例
private:
	CComObject<CMyBlockPortsArray> *myPortArray;	// 创建一个端口数组的实例
	CComObject<CMyParameterArray> *myParameterArray;	// 创建一个parameters数组的实例

public:
	CMyBlock()
	{
		// Constructor步骤
		//MessageBox(NULL, L"Constructor, OK", L"", MB_OK);

		// 实例化头部创建的myPortArray，获取到ports参数数组
		CComObject<CMyBlockPortsArray>::CreateInstance(&myPortArray);
		myPortArray->AddRef();	// 添加引用计数函数

		// 实例化头部创建的parameterArray，获取到parameters参数数组
		CComObject<CMyParameterArray>::CreateInstance(&myParameterArray);
		myParameterArray->AddRef();	// 添加引用计数函数
	}

DECLARE_REGISTRY_RESOURCEID(106)


BEGIN_COM_MAP(CMyBlock)
	COM_INTERFACE_ENTRY(IMyBlock)
	COM_INTERFACE_ENTRY2(IDispatch, ICapeUnit)
	COM_INTERFACE_ENTRY(ICapeUnit)
	COM_INTERFACE_ENTRY(ICapeUtilities)
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




// ICapeUnit Methods
public:
	STDMETHOD(get_ports)(LPDISPATCH *ports)
	{
		// 创建数组
		//CComObject<CMyBlockPortsArray> *p;
		// 实例化头部创建的myPortArray，获取到端口数组
		//CComObject<CMyBlockPortsArray>::CreateInstance(&p);
		// 将pIDispatch指针指向ports参数，给myPortArray赋值
		//p->QueryInterface(IID_IDispatch, (LPVOID*)ports);

		// Get_ports步骤开始
		//MessageBox(NULL, L"Get_ports, Start", L"", MB_OK);

		// 如果是空值，则拦截传过来的*ports
		if (ports == NULL) return E_FAIL;
		
		//CComObject<CMyBlockPortsArray>::CreateInstance(&myPortArray);
		//myPortArray->QueryInterface(IID_IDispatch, (LPVOID*)ports);
		// 更改传值的方式，直接强制转化为ICapeCollection指针
		*ports = (ICapeCollection*)myPortArray;
		myPortArray->AddRef();

		// Get_ports步骤结束
		//MessageBox(NULL, L"Get_ports, End", L"", MB_OK);

		return S_OK;
	}

	STDMETHOD(get_ValStatus)(CapeValidationStatus *status)
	{
		// Get_Val_Status步骤
		//MessageBox(NULL, L"Get_Val_Status, OK", L"", MB_OK);

		// 获取当前端口的状态
		*status = CapeValidationStatus::CAPE_VALID;	// 默认端口可用

		return S_OK;
	}

	// 获取进口流股物流对象中的参数，主要为温度、压力、摩尔流量、摩尔组成
	BOOL GetOverallTPFlowComposition(double& temperature, double& pressure, double& totalMoleFlow, CVariant& moleComposition)
	{
		// 定义临时变量
		HRESULT hr;
		std::wstring error;
		CVariant myValue;
		//myVariantValue()函数在Variant.h文件中定义返回value值
		// 获取温度
		hr = myPortArray->getInlet()->GetOverallProp(CBSTR(_T("temperature")), NULL, &myValue.myVariantValue());
		myValue.CheckArray(VT_R8, error);
		temperature = myValue.GetDoubleAt(0);
		// 获取压力
		hr = myPortArray->getInlet()->GetOverallProp(CBSTR(_T("pressure")), NULL, &myValue.myVariantValue());
		myValue.CheckArray(VT_R8, error);
		pressure = myValue.GetDoubleAt(0);
		// 获取总摩尔流量
		hr = myPortArray->getInlet()->GetOverallProp(CBSTR(_T("totalFlow")), CBSTR(_T("mole")), &myValue.myVariantValue());
		myValue.CheckArray(VT_R8, error);
		totalMoleFlow = myValue.GetDoubleAt(0);
		// 获取组分的摩尔分率
		VARIANT v;
		v.vt = VT_EMPTY;
		hr = myPortArray->getInlet()->GetOverallProp(CBSTR(_T("fraction")), CBSTR(_T("mole")), &v);
		myValue.CheckArray(VT_R8, error);
		moleComposition.Set(v, true);
				
		/* // 测试是否获取到流股的值
		std::string s = std::to_string(temperature);
		std::wstring stamp = std::wstring(s.begin(), s.end());
		LPCWSTR sw = stamp.c_str();
		MessageBox(NULL, sw, L"", MB_OK);
		*/

		return 1;
	}

	// 将计算完毕的参数赋值给流股并执行一次闪蒸
	BOOL SetOverallTPFlowCompositionAndFlash(double temperature, double pressure, double totalMoleFlow, CVariant& moleComposition)
	{
		// 定义临时变量
		HRESULT hr;
		CVariant myValue;
		// 设置温度
		myValue.MakeArray(1, VT_R8);
		myValue.SetDoubleAt(0, temperature);
		hr = myPortArray->getOutlet()->SetOverallProp(CBSTR(L"temperature"), NULL, myValue);
		// 设置压力
		myValue.MakeArray(1, VT_R8);
		myValue.SetDoubleAt(0, pressure);
		hr = myPortArray->getOutlet()->SetOverallProp(CBSTR(L"pressure"), NULL, myValue);
		// 设置总摩尔流量
		myValue.MakeArray(1, VT_R8);
		myValue.SetDoubleAt(0, totalMoleFlow);
		hr = myPortArray->getOutlet()->SetOverallProp(CBSTR(L"totalFlow"), CBSTR(L"mole"), myValue);
		// 设置组分摩尔分率
		hr = myPortArray->getOutlet()->SetOverallProp(CBSTR(L"fraction"), CBSTR(L"mole"), moleComposition);
		// 执行一次闪蒸，确定出口流股的相态
		CalcEquilibriumByTemperatureAndPressure();

		return 1;
	}

	// 获取组分列表函数
	/*
	BOOL GetCompoundsList(CVariant& aliasNameList)
	{
		CComPtr<ICapeThermoCompounds> capeThermoCompounds;
		myPortArray->getInlet()->QueryInterface(IID_ICapeThermoCompounds, (LPVOID*)&capeThermoCompounds);
		CVariant formulaList, nameList, boilingPointList, molecularWeightList, casList;
		HRESULT hr = capeThermoCompounds->GetCompoundList(&aliasNameList.value, &formulaList.value, &nameList.value, &boilingPointList.value, &molecularWeightList.value, &casList.value);
		std::wstring error;
		aliasNameList.CheckArray(VT_BSTR, error);

		return 1;
	}
	*/

	// 闪蒸函数
	BOOL CalcEquilibriumByTemperatureAndPressure()
	{
		// 定义临时变量
		CVariant flashSpec1, flashSpec2;
		CBSTR overall(L"overall");
		// 温度闪蒸
		flashSpec1.MakeArray(3, VT_BSTR);
		flashSpec1.AllocStringAt(0, L"temperature");
		flashSpec1.SetStringAt(1, NULL);
		flashSpec1.SetStringAt(2, overall);
		// 压力闪蒸
		flashSpec2.MakeArray(3, VT_BSTR);
		flashSpec2.AllocStringAt(0, L"pressure");
		flashSpec2.SetStringAt(1, NULL);
		flashSpec2.SetStringAt(2, overall);
		// 创建一个闪蒸计算的实例
		CComPtr<ICapeThermoEquilibriumRoutine> capeThermoEquilibriumRoutine;
		// 获取赋值完毕的出口流股信息
		myPortArray->getOutlet()->QueryInterface(IID_ICapeThermoEquilibriumRoutine, (LPVOID*)&capeThermoEquilibriumRoutine);
		// 执行闪蒸
		HRESULT hr = capeThermoEquilibriumRoutine->CalcEquilibrium(flashSpec1, flashSpec2, CBSTR(_T("unspecified")));

		return 1;
	}

	STDMETHOD(Calculate)()
	{
		/*
		VARIANT v1;
		v1.vt = VT_EMPTY;
		HRESULT hr = myPortArray->getInlet()->GetOverallProp(L"totalFlow", L"mole", &v1);
		std::wstring error;
		CVariant v(v1, true);
		v.CheckArray(VT_R8, error);
		double myFlow = v.GetDoubleAt(0);
		std::string s = std::to_string(myFlow);
		std::wstring stamp = std::wstring(s.begin(), s.end());
		LPCWSTR sw = stamp.c_str();
		MessageBox(NULL, sw, L"", MB_OK);
		*/
		// 定义需要传入的参数
		double temperature, pressure, totalMoleFlow;
		CVariant moleComposition;
		// 调用获取入口流股物流对象参数
		GetOverallTPFlowComposition(temperature, pressure, totalMoleFlow, moleComposition);

		// 临时定义参数部分
		temperature = 400;	// 默认单位为K
		pressure = 301325;	// 默认单位为Pa

		// 设置出口流股物流对象参数
		SetOverallTPFlowCompositionAndFlash(temperature, pressure, totalMoleFlow, moleComposition);

		return S_OK;
	}

	STDMETHOD(Validate)(BSTR *message, VARIANT_BOOL *status)
	{
		// Validate步骤
		//MessageBox(NULL, L"Validate, OK", L"", MB_OK);

		// 和get_ValStatus函数比，Validate是带有检查功能的获取端口状态
		CBSTR myMsg(L"NO ERROR");
		*message = myMsg;
		*status = TRUE;

		return S_OK;
	}


// ICapeUtilities Methods
public:
	STDMETHOD(get_parameters)(LPDISPATCH *parameters)
	{
		// 该函数是与灵敏度分析、工况分析和aspen下的代码调用参数功能相关
		
		// Get_parameters步骤
		//MessageBox(NULL, L"Get_parameters, OK", L"", MB_OK);
		
		// 实例化头部创建的parameterArray，获取到parameters参数数组
		//CComObject<CMyParameterArray>::CreateInstance(&parameterArray);

		//*parameters = NULL;
		// 将pIDispatch指针指向parameters参数，给myParameterArray赋值
		//myParameterArray->QueryInterface(IID_IDispatch, (LPVOID*)parameters);
		// 更改传值的方式，直接强制转化为ICapeCollection指针
		*parameters = (ICapeCollection*)myParameterArray;
		myParameterArray->AddRef();

		return S_OK;
	}

	STDMETHOD(put_simulationContext)(LPDISPATCH pszsimulationContext)
	{
		// 该函数是与流程模拟软件交互的，举例来说就是当模块计算失败或陷入死循环时
		// 可以通过该函数告诉流程模拟软件，模块计算失败
		// 这里也暂时不做实现

		// Put_simulationContext步骤
		//MessageBox(NULL, L"Put_simulationContext, OK", L"", MB_OK);

		return S_OK;
	}

	STDMETHOD(Initialize)()
	{
		// 该函数是初始化函数
		// 已在文件头部的构造函数中初始化实例，这里只返回初始化成功状态码

		// Initialize步骤
		//MessageBox(NULL, L"Initialize, OK", L"", MB_OK);

		return S_OK;
	}

	STDMETHOD(Terminate)()
	{
		// 卸载函数，不做实现

		// Terminate步骤
		//MessageBox(NULL, L"Terminate, OK", L"", MB_OK);

		return S_OK;
	}

	STDMETHOD(Edit)()
	{
		// 该函数就是双击之后的逻辑，编辑该单元模块
		// 但这里不做实现，仅使用消息弹窗来测试双击编辑的功能
		MessageBox(NULL, L"Hello World!", L"By laugh", MB_OK);

		return S_OK;
	}


// ICapeIdentification Methods
public:
	STDMETHOD(get_ComponentName)(BSTR *componentName)
	{
		// 获取单元模块的名字
		//CBSTR nam(SysAllocString(CA2W("Test Block")));	// string转const OLECHAR*类型
		CBSTR nam(SysAllocString(L"Test Block"));
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
		// 获取单元模块的描述
		//CBSTR des(SysAllocString(CA2W("Test Block Description")));	// string转const OLECHAR*类型
		CBSTR des(SysAllocString(L"Test Block Description"));	// string转const OLECHAR*类型
		*componentDescription = des;

		return S_OK;
	}

	STDMETHOD(put_ComponentDescription)(BSTR pszComponentDescription)
	{
		// 禁止自动设置，直接返回状态码
		return S_OK;
	}

};

OBJECT_ENTRY_AUTO(__uuidof(MyBlock), CMyBlock)
