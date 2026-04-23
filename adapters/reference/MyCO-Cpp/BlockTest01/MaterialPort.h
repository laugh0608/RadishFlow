// MaterialPort.h: CMaterialPort 的声明

#pragma once
#include "resource.h"       // 主符号

#include <string>
using namespace std;
#include "Variant.h"
#include "atlbase.h"
#include "atlconv.h"

#include "BlockTest01_i.h"



#if defined(_WIN32_WCE) && !defined(_CE_DCOM) && !defined(_CE_ALLOW_SINGLE_THREADED_OBJECTS_IN_MTA)
#error "Windows CE 平台(如不提供完全 DCOM 支持的 Windows Mobile 平台)上无法正确支持单线程 COM 对象。定义 _CE_ALLOW_SINGLE_THREADED_OBJECTS_IN_MTA 可强制 ATL 支持创建单线程 COM 对象实现并允许使用其单线程 COM 对象实现。rgs 文件中的线程模型已被设置为“Free”，原因是该模型是非 DCOM Windows CE 平台支持的唯一线程模型。"
#endif

using namespace ATL;


// CMaterialPort

class ATL_NO_VTABLE CMaterialPort :
	public CComObjectRootEx<CComSingleThreadModel>,
	public CComCoClass<CMaterialPort, &CLSID_MaterialPort>,
	public IDispatchImpl<IMaterialPort, &IID_IMaterialPort, &LIBID_BlockTest01Lib, /*wMajor =*/ 1, /*wMinor =*/ 0>,
	public IDispatchImpl<ICapeUnitPort, &__uuidof(ICapeUnitPort), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>,
	public IDispatchImpl<ICapeIdentification, &__uuidof(ICapeIdentification), &LIBID_CAPEOPEN110, /* wMajor = */ 1, /* wMinor = */ 1>
{

private:
	// 创建一个物流对象实例
	//LPDISPATCH pMaterialObject;
	//IDispatch *pMaterialObject;
	// 传统方法，版本 1.1
	//ICapeThermoMaterial* pMaterialObject;
	// 变化 1，智能指针
	CComPtr<ICapeThermoMaterial> pMaterialObject;

	// 传入参数，为端口流股方向
	CapePortDirection pDirection;

	// 端口名称
	//string pName;
	wstring pName;
	// 端口描述
	//string pDesc;
	wstring pDesc;

public:
	//CMaterialPort(CapePortDirection pDirection)
	CMaterialPort()
	{
		// 给物流对象实例赋一个初始值
		pMaterialObject = NULL;

		// 将端口方向参数传入公有
		//this->pDirection = pDirection;
	}
	// 返回流股对象给 PortsArray 中的 getInlet 函数
	//IDispatch*& getMaterial() {
	// 传统方法，版本 1.1
	/*ICapeThermoMaterial* &getMaterial() {
		return pMaterialObject;
	}*/
	// 变化 2，智能指针
	CComPtr<ICapeThermoMaterial>& getMaterial() {
		return pMaterialObject;
	}

	// 设置端口流股方向
	void SetDirection(CapePortDirection pDirection) {
		// 将端口方向参数传入共有
		this->pDirection = pDirection;
	}

	// 设置端口名称和描述
	//void SetNameAndDesc(string pName, string pDesc) {
	void SetNameAndDesc(wstring pName, wstring pDesc) {
		this->pName = pName;
		this->pDesc = pDesc;
	}

DECLARE_REGISTRY_RESOURCEID(108)


BEGIN_COM_MAP(CMaterialPort)
	COM_INTERFACE_ENTRY(IMaterialPort)
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
		// 设置端口类型为流股类型
		*portType = CapePortType::CAPE_MATERIAL;

		return S_OK;
	}

	STDMETHOD(get_direction)(CapePortDirection *portDirection)
	{
		// 改为参数传入形式
		*portDirection = this->pDirection;
		// 设置端口流股方向为进口
		//*portDirection = CapePortDirection::CAPE_INLET;

		return S_OK;
	}

	STDMETHOD(get_connectedObject)(LPDISPATCH *connectedObject)
	{
		// 设置端口流股连接状态为未连接
		//*connectedObject = NULL;
		*connectedObject = pMaterialObject;
		// 变化 3，增加计数函数
		(*connectedObject)->AddRef();

		return S_OK;
	}

	STDMETHOD(Connect)(LPDISPATCH objectToConnect)
	{
		// 连接时的状态，强行连接到手动创建的物流对象
		//pMaterialObject = objectToConnect;
		//objectToConnect->QueryInterface(IID_IDispatch, (LPVOID*)&pMaterialObject);
		objectToConnect->QueryInterface(IID_ICapeThermoMaterial, (LPVOID*)&pMaterialObject);

		return S_OK;
	}

	STDMETHOD(Disconnect)()
	{
		// 断开时的状态，强行赋值
		pMaterialObject = NULL;

		return S_OK;
	}


// ICapeIdentification Methods
public:
	STDMETHOD(get_ComponentName)(BSTR *pComponentName)
	{
		// 获取端口的名字
		//CBSTR n(SysAllocString(CA2W(pName.c_str())));	// string 转 const OLECHAR* 类型
		//CBSTR n(L"INLET");
		//*pComponentName = n;
		//*pComponentName = SysAllocString(CA2W(pName.c_str()));
		*pComponentName = SysAllocString(pName.c_str());

		return S_OK;
	}

	STDMETHOD(put_ComponentName)(BSTR pszComponentName)
	{
		// 不做实现，返回空结果

		return S_OK;
	}

	STDMETHOD(get_ComponentDescription)(BSTR *pComponentDesc)
	{
		// 获取端口的描述
		//CBSTR d(SysAllocString(CA2W(pDesc.c_str())));	// string 转 const OLECHAR* 类型
		//*pComponentDesc = d;
		//*pComponentDesc = SysAllocString(CA2W(pDesc.c_str()));
		*pComponentDesc = SysAllocString(pDesc.c_str());

		return S_OK;
	}

	STDMETHOD(put_ComponentDescription)(BSTR pszComponentDesc)
	{
		// 不做实现，返回空结果

		return S_OK;
	}

};

OBJECT_ENTRY_AUTO(__uuidof(MaterialPort), CMaterialPort)
