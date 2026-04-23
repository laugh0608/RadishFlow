// MaterialObject.h: CMaterialObject 的声明

#pragma once
#include "resource.h"       // 主符号



#include "MyThermoTest_i.h"

// 创建物流对象实例，引入组分需要的类型库
#include <string>
#include <vector>
// GetCompoundList 函数示例，引入 Variant.h
#include "Variant.h"



#if defined(_WIN32_WCE) && !defined(_CE_DCOM) && !defined(_CE_ALLOW_SINGLE_THREADED_OBJECTS_IN_MTA)
#error "Windows CE 平台(如不提供完全 DCOM 支持的 Windows Mobile 平台)上无法正确支持单线程 COM 对象。定义 _CE_ALLOW_SINGLE_THREADED_OBJECTS_IN_MTA 可强制 ATL 支持创建单线程 COM 对象实现并允许使用其单线程 COM 对象实现。rgs 文件中的线程模型已被设置为“Free”，原因是该模型是非 DCOM Windows CE 平台支持的唯一线程模型。"
#endif

using namespace ATL;


// CMaterialObject

class ATL_NO_VTABLE CMaterialObject :
	public CComObjectRootEx<CComSingleThreadModel>,
	public CComCoClass<CMaterialObject, &CLSID_MaterialObject>,
	public IDispatchImpl<IMaterialObject, &IID_IMaterialObject, &LIBID_MyThermoTestLib, /*wMajor =*/ 1, /*wMinor =*/ 0>,
	public IDispatchImpl<ICapeThermoMaterial, &__uuidof(ICapeThermoMaterial), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 0>,
	public IDispatchImpl<ICapeThermoPhases, &__uuidof(ICapeThermoPhases), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 0>,
	public IDispatchImpl<ICapeThermoCompounds, &__uuidof(ICapeThermoCompounds), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 0>,
	public IDispatchImpl<ICapeThermoEquilibriumRoutine, &__uuidof(ICapeThermoEquilibriumRoutine), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 0>,
	public IDispatchImpl<ICapeThermoPropertyRoutine, &__uuidof(ICapeThermoPropertyRoutine), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 0>
{
public:
	// 创建物流对象实例，声明物流对象中需要什么信息
	std::string myEos;	// 使用什么状态方程
	std::vector<std::wstring> myCompounds;	// 含有什么组分，组分列表

	// 单相物性计算实例 ，声明计算需要传入的参数
	double T, P;	// 温度、压力，还应该传入流股的组成，这里为了学习理解暂时不写

	CMaterialObject()
	{
	}

DECLARE_REGISTRY_RESOURCEID(114)


BEGIN_COM_MAP(CMaterialObject)
	COM_INTERFACE_ENTRY(IMaterialObject)
	COM_INTERFACE_ENTRY2(IDispatch, ICapeThermoMaterial)
	COM_INTERFACE_ENTRY(ICapeThermoMaterial)
	COM_INTERFACE_ENTRY(ICapeThermoPhases)
	COM_INTERFACE_ENTRY(ICapeThermoCompounds)
	COM_INTERFACE_ENTRY(ICapeThermoEquilibriumRoutine)
	COM_INTERFACE_ENTRY(ICapeThermoPropertyRoutine)
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




// ICapeThermoMaterial Methods
public:
	STDMETHOD(ClearAllProps)()
	{
		 return E_NOTIMPL;
	}

	STDMETHOD(CopyFromMaterial)(LPDISPATCH * source)
	{
		 return E_NOTIMPL;
	}

	STDMETHOD(CreateMaterial)(LPDISPATCH *CreateMaterial)
	{
		 return E_NOTIMPL;
	}

	STDMETHOD(GetOverallProp)(BSTR property, BSTR basis, VARIANT * results)
	{
		 return E_NOTIMPL;
	}

	STDMETHOD(GetOverallTPFraction)(double * temperature, double * pressure, VARIANT * composition)
	{
		 return E_NOTIMPL;
	}

	STDMETHOD(GetPresentPhases)(VARIANT * phaseLabels, VARIANT * phaseStatus)
	{
		 return E_NOTIMPL;
	}

	STDMETHOD(GetSinglePhaseProp)(BSTR property, BSTR phaseLabel, BSTR basis, VARIANT * results)
	{
		 return E_NOTIMPL;
	}

	STDMETHOD(GetTPFraction)(BSTR phaseLabel, double * temperature, double * pressure, VARIANT * composition)
	{
		 return E_NOTIMPL;
	}

	STDMETHOD(GetTwoPhaseProp)(BSTR property, VARIANT phaseLabels, BSTR basis, VARIANT * results)
	{
		 return E_NOTIMPL;
	}

	STDMETHOD(SetOverallProp)(BSTR property, BSTR basis, VARIANT values)
	{
		// 下方 ICapeThermoPropertyRoutine 接口的 CalcTwoPhaseProp 函数参数从这里传入
		// 从 property 拿到值给 myPro
		CBSTR myPro(property);
		// 从 myPro 中取出温度值
		if (myPro.Same(L"Temperature"))
		{
			CVariant v(values, true);
			std::wstring error;
			v.CheckArray(VT_R8, error);	// 流股中含有不同的组分，所以这是个数组
			T = v.GetDoubleAt(0);	// 我们只示例读取温度数组中的第一个温度
		}
		// 从 myPro 中取出压力值
		if (myPro.Same(L"Pressure"))
		{
			CVariant v(values, true);
			std::wstring error;
			v.CheckArray(VT_R8, error);	// 同上
			P = v.GetDoubleAt(0);
		}

		return E_NOTIMPL;
	}

	STDMETHOD(SetPresentPhases)(VARIANT phaseLabels, VARIANT phaseStatus)
	{
		 return E_NOTIMPL;
	}

	STDMETHOD(SetSinglePhaseProp)(BSTR property, BSTR phaseLabel, BSTR basis, VARIANT values)
	{
		 return E_NOTIMPL;
	}

	STDMETHOD(SetTwoPhaseProp)(BSTR property, VARIANT phaseLabels, BSTR basis, VARIANT values)
	{
		 return E_NOTIMPL;
	}


// ICapeThermoPhases Methods
public:
	STDMETHOD(GetNumPhases)(long *GetNumPhases)
	{
		 return E_NOTIMPL;
	}

	STDMETHOD(GetPhaseInfo)(BSTR phaseLabel, BSTR phaseAttribute, VARIANT *GetPhaseInfo)
	{
		 return E_NOTIMPL;
	}

	STDMETHOD(GetPhaseList)(VARIANT * phaseLabels, VARIANT * stateOfAggregation, VARIANT * keyCompoundId)
	{
		 return E_NOTIMPL;
	}


// ICapeThermoCompounds Methods
public:
	STDMETHOD(GetCompoundConstant)(VARIANT props, VARIANT compIds, VARIANT *GetCompoundConstant)
	{
		 return E_NOTIMPL;
	}

	STDMETHOD(GetCompoundList)(VARIANT * compIds, VARIANT * formulae, VARIANT * names, VARIANT * boilTemps, VARIANT * molwts, VARIANT * casnos)
	{
		// 声明六个变量，以储存函数对应的参数，都是数组
		CVariant a, b, c, d, e, f;
		a.MakeArray(myCompounds.size(), VT_BSTR);	// 组分ID，字符串
		b.MakeArray(myCompounds.size(), VT_BSTR);	// 组分分子式，字符串
		c.MakeArray(myCompounds.size(), VT_BSTR);	// 组分名称，字符串
		d.MakeArray(myCompounds.size(), VT_R8);		// 组分沸点，浮点数
		e.MakeArray(myCompounds.size(), VT_R8);		// 组分分子量，浮点数
		f.MakeArray(myCompounds.size(), VT_BSTR);	// 组分CAS号，字符串
		// 等等等等，还可以自己添加其他参数

		// 给参数赋值
		for (size_t i = 0; i < myCompounds.size(); i++)
		{
			a.AllocStringAt(i, myCompounds[i].c_str());	// wstring类型转const OLECHAR*类型
			b.AllocStringAt(i, myCompounds[i].c_str());	// 这里应该是调用数据库直接输入组分的这些信息
			c.AllocStringAt(i, myCompounds[i].c_str());	// 为了简化学习，直接采用组分的数组序号进行了输入
			d.SetDoubleAt(i, 88 + i);	// 同样，这里只是简化输入，后面的+i只是为了差异化不同的组分数据
			e.SetDoubleAt(i, 99 + i);	// 同上
			f.AllocStringAt(i, myCompounds[i].c_str());
		}
		// 返回数组数据
		*compIds = a.ReturnValue();
		*formulae = b.ReturnValue();
		*names = c.ReturnValue();
		*boilTemps = d.ReturnValue();
		*molwts = e.ReturnValue();
		*casnos = f.ReturnValue();

		return S_OK;

		/*
		// 示例补充接口中的函数如何编写
		// 创建三个参数，类型数组，长度对应上文中获取到的组分列表
		// 分别对应函数参数中的：compIds、boilTemps
		CVariant myComListId, myComListBo;
		myComListId.MakeArray(myCompounds.size(), VT_BSTR);	// 组分的 ID，类型字符串
		myComListBo.MakeArray(myCompounds.size(), VT_R8);	// 组分的沸点，浮点数
		//......	其他的参数就不一一列举

		//给参数赋值
		for (size_t i = 0; i < myCompounds.size(); i++)
		{
			// 这里赋值的内容应该来自于数据库，而不是直接赋值
			myComListId.AllocStringAt(i, L"Compounds-i");	// 每个组分的ID都是Compounds-i，如果要启用该形式，需要在头文件下方更改myCompounds的类型为std::string
			myComListBo.SetDoubleAt(i, 80);	// 每个组分的沸点都是80
		}
		// 返回已经赋值的参数
		*compIds = myComListId.ReturnValue();
		*boilTemps = myComListBo.ReturnValue();
		*/
	}

	STDMETHOD(GetConstPropList)(VARIANT *GetConstPropList)
	{
		 return E_NOTIMPL;
	}

	STDMETHOD(GetNumCompounds)(long *GetNumCompounds)
	{
		// 返回组分数
		*GetNumCompounds = myCompounds.size();	// 根据组分列表的长度返回组分数
		return S_OK;
	}

	STDMETHOD(GetPDependentProperty)(VARIANT props, double pressure, VARIANT compIds, VARIANT * propVals)
	{
		 return E_NOTIMPL;
	}

	STDMETHOD(GetPDependentPropList)(VARIANT *GetPDependentPropList)
	{
		 return E_NOTIMPL;
	}

	STDMETHOD(GetTDependentProperty)(VARIANT props, double temperature, VARIANT compIds, VARIANT * propVals)
	{
		 return E_NOTIMPL;
	}

	STDMETHOD(GetTDependentPropList)(VARIANT *GetTDependentPropList)
	{
		 return E_NOTIMPL;
	}


// ICapeThermoEquilibriumRoutine Methods
public:
	STDMETHOD(CalcEquilibrium)(VARIANT specification1, VARIANT specification2, BSTR solutionType)
	{
		 return E_NOTIMPL;
	}

	STDMETHOD(CheckEquilibriumSpec)(VARIANT specification1, VARIANT specification2, BSTR solutionType, VARIANT_BOOL *CheckEquilibriumSpec)
	{
		 return E_NOTIMPL;
	}


// ICapeThermoPropertyRoutine Methods
public:
	// 修复 ICapeThermoPropertyRoutine 接口无法使用的bug，将 fFlags 的类型修改为 int
	//STDMETHOD(CalcAndGetLnPhi)(BSTR phaseLabel, double temperature, double pressure, VARIANT moleNumbers, long fFlags, VARIANT* lnPhi, VARIANT* lnPhiDT, VARIANT* lnPhiDP, VARIANT* lnPhiDn)
	STDMETHOD(CalcAndGetLnPhi)(BSTR phaseLabel, double temperature, double pressure, VARIANT moleNumbers, int fFlags, VARIANT* lnPhi, VARIANT* lnPhiDT, VARIANT* lnPhiDP, VARIANT* lnPhiDn)
	{
		 return E_NOTIMPL;
	}

	STDMETHOD(CalcSinglePhaseProp)(VARIANT props, BSTR phaseLabel)
	{
		 return E_NOTIMPL;
	}

	// 学习单相物性计算，更改了 props 的类型为 BSTR，物性应该是字符串类型
	//STDMETHOD(CalcTwoPhaseProp)(VARIANT props, VARIANT phaseLabels)
	
	//STDMETHOD(CalcTwoPhaseProp)(BSTR props, VARIANT phaseLabels)
	STDMETHOD(CalcTwoPhaseProp)(VARIANT props, VARIANT phaseLabels)	// 恢复 props 的类型为 VARIANT
	{
		// 创建一个物性
		//CBSTR property(props);
		//// 读取单元模块要求计算什么物性
		//if (property.Same(L"density"))
		//{
		//	double density = 7 * T * P;	// 温度，压力参数来自于单元模块传入，（这里的公式是瞎写的，示例用）
		//	// 这里计算完毕之后并不是直接返回计算结果，而是暂时储存在这里，通过GetOverallProp()这个函数来读取
		//}

		 return E_NOTIMPL;
	}

	STDMETHOD(CheckSinglePhasePropSpec)(BSTR property, BSTR phaseLabel, VARIANT_BOOL *CheckSinglePhasePropSpec)
	{
		 return E_NOTIMPL;
	}

	STDMETHOD(CheckTwoPhasePropSpec)(BSTR property, VARIANT phaseLabels, VARIANT_BOOL *CheckTwoPhasePropSpec)
	{
		 return E_NOTIMPL;
	}

	STDMETHOD(GetSinglePhasePropList)(VARIANT *GetSinglePhasePropList)
	{
		 return E_NOTIMPL;
	}

	STDMETHOD(GetTwoPhasePropList)(VARIANT *GetTwoPhasePropList)
	{
		 return E_NOTIMPL;
	}

};

OBJECT_ENTRY_AUTO(__uuidof(MaterialObject), CMaterialObject)
