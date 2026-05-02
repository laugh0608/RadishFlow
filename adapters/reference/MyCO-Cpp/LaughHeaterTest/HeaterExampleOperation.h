// HeaterExampleOperation.h: CHeaterExampleOperation 的声明

#pragma once
#include "resource.h"       // 主符号

#include "PortsArray.h"		// 添加对 PortsArray 的引用
#include "Variant.h"		// 添加对 Variant 的引用
#include "ParametersArray.h"	// 添加对 ParametersArray 的引用
#include <string>		// 添加对 wstring 的引用
#include <fstream>		// 添加对 ifstream 的引用
#include <cstdlib>		// 添加对 cstdlib 的引用
using namespace std;

#include "HeaterExample_i.h"



#if defined(_WIN32_WCE) && !defined(_CE_DCOM) && !defined(_CE_ALLOW_SINGLE_THREADED_OBJECTS_IN_MTA)
#error "Windows CE 平台(如不提供完全 DCOM 支持的 Windows Mobile 平台)上无法正确支持单线程 COM 对象。定义 _CE_ALLOW_SINGLE_THREADED_OBJECTS_IN_MTA 可强制 ATL 支持创建单线程 COM 对象实现并允许使用其单线程 COM 对象实现。rgs 文件中的线程模型已被设置为“Free”，原因是该模型是非 DCOM Windows CE 平台支持的唯一线程模型。"
#endif

using namespace ATL;


// CHeaterExampleOperation

class ATL_NO_VTABLE CHeaterExampleOperation :
	public CComObjectRootEx<CComSingleThreadModel>,
	public CComCoClass<CHeaterExampleOperation, &CLSID_HeaterExampleOperation>,
	public IDispatchImpl<IHeaterExampleOperation, &IID_IHeaterExampleOperation, &LIBID_HeaterExampleLib, /*wMajor =*/ 1, /*wMinor =*/ 0>,
	public IDispatchImpl<ICapeUnit, &__uuidof(ICapeUnit), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>,
	public IDispatchImpl<ICapeUtilities, &__uuidof(ICapeUtilities), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>,
	public IDispatchImpl<ICapeIdentification, &__uuidof(ICapeIdentification), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>
{

private:
	// 创建端口数组
	CComObject<CPortsArray> *pPortArray;
	// 创建 Parameter 参数集数组
	CComObject<CParametersArray> *pParametersArray;

public:
	CHeaterExampleOperation()
	{
		// 实例化创建的端口数组
		CComObject<CPortsArray>::CreateInstance(&pPortArray);
		pPortArray->AddRef();
		// 实例化创建的 Parameters 参数集数组
		CComObject<CParametersArray>::CreateInstance(&pParametersArray);
		pParametersArray->AddRef();
	}

DECLARE_REGISTRY_RESOURCEID(106)


BEGIN_COM_MAP(CHeaterExampleOperation)
	COM_INTERFACE_ENTRY(IHeaterExampleOperation)
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
		// 获取端口为空时进行拦截
		if (ports == NULL) return E_FAIL;
		//if (*ports == NULL) return E_FAIL;
		
		// 创建端口数组
		//CComObject<CPortsArray> *pPortArray;
		// 实例化创建的端口数组
		//CComObject<CPortsArray>::CreateInstance(&pPortArray);
		// 返回获取的 ports 结果
		//pPortArray->QueryInterface(IID_IDispatch, (LPVOID*)ports);
		*ports = (ICapeCollection*)pPortArray;
		pPortArray->AddRef();

		return S_OK;
	}

	STDMETHOD(get_ValStatus)(CapeValidationStatus *pValStatus)
	{
		// 默认端口状态可用
		*pValStatus = CapeValidationStatus::CAPE_VALID;

		return S_OK;
	}

	// 获取进口流股物流对象中的参数，主要为温度、压力、摩尔流量、摩尔组成
	BOOL GetOverallTPFlowComposition(double& temperature, double& pressure, double& totalMoleFlow, CVariant& moleComposition)
	{
		// 定义临时变量
		HRESULT hr;
		wstring error;
		CVariant myValue;
		// PValue() 函数在 Variant.h 文件中定义返回 value 值
		// 获取温度
		hr = pPortArray->getInlet()->GetOverallProp(CBSTR(_T("temperature")), NULL, &myValue.Pvalue());
		myValue.CheckArray(VT_R8, error);
		temperature = myValue.GetDoubleAt(0);
		// 获取压力
		hr = pPortArray->getInlet()->GetOverallProp(CBSTR(_T("pressure")), NULL, &myValue.Pvalue());
		!myValue.CheckArray(VT_R8, error);
		pressure = myValue.GetDoubleAt(0);
		// 获取总摩尔流量
		hr = pPortArray->getInlet()->GetOverallProp(CBSTR(_T("totalFlow")), CBSTR(_T("mole")), &myValue.Pvalue());
		!myValue.CheckArray(VT_R8, error);
		totalMoleFlow = myValue.GetDoubleAt(0);
		// 获取组分的摩尔分率
		VARIANT pv;
		pv.vt = VT_EMPTY;
		hr = pPortArray->getInlet()->GetOverallProp(CBSTR(_T("fraction")), CBSTR(_T("mole")), &pv);
		myValue.CheckArray(VT_R8, error);
		moleComposition.Set(pv, TRUE);

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
		hr = pPortArray->getOutlet()->SetOverallProp(CBSTR(L"temperature"), NULL, myValue);
		// 设置压力
		myValue.MakeArray(1, VT_R8);
		myValue.SetDoubleAt(0, pressure);
		hr = pPortArray->getOutlet()->SetOverallProp(CBSTR(L"pressure"), NULL, myValue);
		// 设置总摩尔流量
		myValue.MakeArray(1, VT_R8);
		myValue.SetDoubleAt(0, totalMoleFlow);
		hr = pPortArray->getOutlet()->SetOverallProp(CBSTR(L"totalFlow"), CBSTR(L"mole"), myValue);
		// 设置组分摩尔分率
		hr = pPortArray->getOutlet()->SetOverallProp(CBSTR(L"fraction"), CBSTR(L"mole"), moleComposition);
		// 执行一次闪蒸，确定出口流股的相态
		CalcEquilibriumByTemperatureAndPressure();

		return 1;
	}
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
		pPortArray->getOutlet()->QueryInterface(IID_ICapeThermoEquilibriumRoutine, (LPVOID*)&capeThermoEquilibriumRoutine);
		// 执行闪蒸
		HRESULT hr = capeThermoEquilibriumRoutine->CalcEquilibrium(flashSpec1, flashSpec2, CBSTR(_T("unspecified")));

		return 1;
	}

	STDMETHOD(Calculate)()
	{
		// 实现计算，通过 PortsArray 中的热力学接口转化而来
		// 定义一个临时变量 v
		//CVariant v;
		// 从 pPortArray 获取到入口流股的热力学对象，并从其中获取到温度值，赋值给临时变量 v
		//pPortArray->getInlet()->GetOverallProp(L"temperature", L"empty", &v.Pvalue());
		// 重新定义一个临时变量
		//VARIANT v2;
		//v2.vt = VT_EMPTY;
		// 获取进口流股摩尔流量，赋值给 v2
		//HRESULT hr = pPortArray->getInlet()->GetOverallProp(L"totalFlow", L"mole", &v2);
		// 从 v2 中取值赋值给 v
		//CVariant v(v2, TRUE);
		// 定义一个临时变量 error 用来返回错误信息
		//wstring error;
		// 检查临时变量 v 是否为数组类型
		//v.CheckArray(VT_R8, error);
		// 读取临时变量 v 数组中的第一个值并赋值给临时变量 T，类型为双精度浮点
		//double T = v.GetDoubleAt(0);
		// 将临时变量 T 中的数值转换为长字符串并赋值给临时变量 sw
		//string s = to_string(T);
		//wstring stamp = wstring(s.begin(), s.end());
		//LPCWSTR sw = stamp.c_str();
		// 跳出一个弹窗，显示临时变量 sw 的值，也就是温度
		//MessageBox(NULL, sw, L"", MB_OK);

		// 实现闪蒸计算
		// 定义需要传入的参数
		double temperature, pressure, totalMoleFlow;
		CVariant moleComposition;
		// 调用获取入口流股物流对象参数
		GetOverallTPFlowComposition(temperature, pressure, totalMoleFlow, moleComposition);

		// 临时定义参数部分
		//temperature = 400;	// 默认单位为 K
		//pressure = 301325;	// 默认单位为 Pa

		// 读取UI界面程序输出的数据中转文件中的温度和压力
		ifstream file("C:/Users/laugh/Downloads/laughHeater_data.txt");
		file >> temperature >> pressure;
		file.close();

		// 设置出口流股物流对象参数
		SetOverallTPFlowCompositionAndFlash(temperature, pressure, totalMoleFlow, moleComposition);

		return S_OK;
	}

	STDMETHOD(Validate)(BSTR *message, VARIANT_BOOL *pValidateStatus)
	{
		// 带有检查功能的状态获取
		CBSTR msg(L"NO ERROR");
		*message = msg;
		// 状态：成功
		*pValidateStatus = TRUE;

		return S_OK;
	}


// ICapeUtilities Methods
public:
	STDMETHOD(get_parameters)(LPDISPATCH *pParameters)
	{
		// 暂时忽略这个接口，赋值为空（与工况分析、灵敏度分析等有关）
		//*pParameters = NULL;
		// 返回获取的 parameters 结果
		//pParametersArray->QueryInterface(IID_IDispatch, (LPVOID*)pParameters);
		*pParameters = (ICapeCollection*)pParametersArray;
		pParametersArray->AddRef();

		return S_OK;
	}

	STDMETHOD(put_simulationContext)(LPDISPATCH pSimulationContext)
	{
		// 该接口是当单元模块状态异常（如计算陷入死循环）时，单元模块与模拟软件通信，告诉模拟软件单元模块状态异常，需要强制结束
		// 这里暂时不做实现

		return S_OK;
	}

	STDMETHOD(Initialize)()
	{
		// 端口数组已在前文的构造函数 CHeaterExampleOperation() 中初始化完成，这里直接返回 OK 即可

		return S_OK;
	}

	STDMETHOD(Terminate)()
	{
		// 单元模块卸载，这里暂时不做实现，返回空结果

		return S_OK;
	}

	STDMETHOD(Edit)()
	{
		// 双击单元模块的逻辑，显示一个弹窗
		//MessageBox(NULL, L"Hello World", L"by laugh", MB_OK);

		// 调用写好的UI界面程序生成数据中转文件
		system("D:/SDK/Java/bin/java.exe -classpath D:/Code/Java-Vue/HeaterGUI/out/production/HeaterGUI/ HeaterGUI");

		return S_OK;
	}


// ICapeIdentification Methods
public:
	STDMETHOD(get_ComponentName)(BSTR *pComponentName)
	{
		// 获取单元模块名字
		CBSTR n(SysAllocString(L"LAUGH Heater"));	// string 转 const OLECHAR* 类型
		*pComponentName = n;

		return S_OK;
	}

	STDMETHOD(put_ComponentName)(BSTR pComponentName)
	{
		// 不做实现，返回空结果
		return S_OK;
	}

	STDMETHOD(get_ComponentDescription)(BSTR *pComponentDescription)
	{
		// 获取单元模块描述
		CBSTR d(SysAllocString(L"LAUGH Heater Desc"));	// string 转 const OLECHAR* 类型
		*pComponentDescription = d;

		return S_OK;
	}

	STDMETHOD(put_ComponentDescription)(BSTR pComponentDescription)
	{
		// 不做实现，返回空结果
		return S_OK;
	}

};

OBJECT_ENTRY_AUTO(__uuidof(HeaterExampleOperation), CHeaterExampleOperation)
