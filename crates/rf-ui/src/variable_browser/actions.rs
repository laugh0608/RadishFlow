use super::*;
use crate::UnitInspectorDraftField;
use crate::state::unit_inspector::unit_inspector_draft_fields;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionKind {
    SetUnitParameter,
    RenameUnit,
    SetStreamSpecification,
    CreateUnit,
    ConnectPorts,
    DisconnectPorts,
    Run,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterType {
    Text,
    Number,
    UnitIdentity,
    StreamIdentity,
    PortName,
    UnitKind,
    MaterialPort,
    PackageSelection,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionParameter {
    pub name: &'static str,
    pub kind: ParameterType,
    pub optional: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionDescriptor {
    pub kind: ActionKind,
    pub label: &'static str,
    pub parameters: Vec<ActionParameter>,
    pub fields: Vec<VariableField>,
    pub preconditions: &'static str,
    pub modifies_document: bool,
    pub undoable: bool,
    pub asynchronous: bool,
    pub returns: &'static str,
}

impl VariableBrowser<'_> {
    /// Describes existing actions, without providing invocation or claiming current availability.
    pub fn actions(&self, object: Option<&ObjectId>) -> Result<Vec<ActionDescriptor>, BrowseError> {
        use ActionKind::*;
        use ParameterType::*;
        let parameter = |name, kind, optional| ActionParameter {
            name,
            kind,
            optional,
        };
        let action = |kind, label, parameters, fields, preconditions, returns| ActionDescriptor {
            kind,
            label,
            parameters,
            fields,
            preconditions,
            returns,
            modifies_document: true,
            undoable: true,
            asynchronous: false,
        };
        if let Some(object) = object {
            self.object(object)?;
        }
        Ok(match object {
            Some(ObjectId::Unit(id)) => {
                let fields = unit_inspector_draft_fields(&self.document.flowsheet.units[id])
                    .into_iter()
                    .filter_map(|f| match f {
                        UnitInspectorDraftField::Name => None,
                        UnitInspectorDraftField::OutletTemperatureK => {
                            Some(VariableField::OutletTemperature)
                        }
                        UnitInspectorDraftField::OutletPressurePa => {
                            Some(VariableField::OutletPressure)
                        }
                    })
                    .collect::<Vec<_>>();
                let mut actions = vec![action(
                    RenameUnit,
                    "重命名单元",
                    vec![parameter("new_name", Text, false)],
                    vec![VariableField::Name],
                    "对象必须存在；名称非空白，允许重名。",
                    "文档修订；对象身份不变",
                )];
                if !fields.is_empty() {
                    actions.push(action(
                        SetUnitParameter,
                        "设置单元参数",
                        vec![parameter("parameter", Text, false), parameter("value", Number, false)],
                        fields,
                        "对象和参数必须受支持；SI 值满足变量约束；经正式 Inspector / 文档命令提交。",
                        "单次文档修订；同步出口模板并旧化结果",
                    ));
                }
                actions
            }
            Some(object @ ObjectId::Stream(_)) => vec![
                action(
                    SetStreamSpecification,
                    "设置流股输入",
                    vec![
                        parameter("field", Text, false),
                        parameter("value", Number, false),
                    ],
                    self.variables(object)?
                        .into_iter()
                        .filter(|v| {
                            v.id.section == VariableSection::Inputs
                                && v.id.field != VariableField::Name
                        })
                        .map(|v| v.id.field)
                        .collect(),
                    "流股必须存在；数值有限并符合 SI 约束；组成项须属于项目组分。名称通过文本值提交。",
                    "单次文档修订；旧化结果",
                ),
                action(
                    SetStreamSpecification,
                    "重命名流股",
                    vec![
                        parameter("field", Text, false),
                        parameter("value", Text, false),
                    ],
                    vec![VariableField::Name],
                    "字段为 name；名称不能空白，允许重名。",
                    "文档修订；对象身份不变",
                ),
            ],
            None => vec![
                action(
                    CreateUnit,
                    "创建单元",
                    vec![parameter("kind", UnitKind, false)],
                    vec![],
                    "使用六类内置单元；创建入口分配未占用身份和标准端口；应用调用需匹配文档 / 修订且无未处理编辑。",
                    "新单元身份和文档修订",
                ),
                action(
                    ConnectPorts,
                    "连接端口",
                    vec![
                        parameter("stream_id", StreamIdentity, false),
                        parameter("source", MaterialPort, false),
                        parameter("sink", MaterialPort, true),
                    ],
                    vec![],
                    "执行时匹配当前本地规则候选，并校验端口方向、类型和占用；可创建受控出口流股。完整拓扑仍在运行前校验。",
                    "文档修订；连接关系",
                ),
                action(
                    DisconnectPorts,
                    "断开端口",
                    vec![
                        parameter("unit_id", UnitIdentity, false),
                        parameter("port", PortName, false),
                    ],
                    vec![],
                    "端口必须存在且已绑定；经现有断开入口提交，保留流股和另一端绑定。",
                    "文档修订；断开结果",
                ),
                ActionDescriptor {
                    kind: Run,
                    label: "运行当前流程",
                    parameters: vec![parameter("package", PackageSelection, false)],
                    fields: vec![],
                    preconditions: "物性包可显式选择或使用 Preferred；无待提交或无效草稿；物性、组分与模型输入就绪；运行时执行 readiness 和求解校验。",
                    modifies_document: false,
                    undoable: false,
                    asynchronous: false,
                    returns: "运行 outcome 与控制状态（含阻塞 / 失败）；同步执行，无任务取消承诺",
                },
            ],
        })
    }
}
