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
                    vec![
                        parameter("unit_id", UnitIdentity, false),
                        parameter("kind", Text, false),
                    ],
                    vec![],
                    "使用受支持单元类型与未占用身份；Studio 放置入口分配身份。",
                    "新单元身份和文档修订",
                ),
                action(
                    ConnectPorts,
                    "连接端口",
                    vec![
                        parameter("stream_id", StreamIdentity, false),
                        parameter("from_unit_id", UnitIdentity, false),
                        parameter("from_port", PortName, false),
                        parameter("to_unit_id", UnitIdentity, true),
                        parameter("to_port", PortName, true),
                    ],
                    vec![],
                    "使用现有受控连接入口；对象存在、端口方向及物料类型匹配；候选唯一且不违反占用与拓扑规则。目标单元和目标端口成对指定。",
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
                    parameters: vec![],
                    fields: vec![],
                    preconditions: "无待提交或无效草稿；物性、组分与模型输入就绪；运行时执行 readiness 和求解校验。",
                    modifies_document: false,
                    undoable: false,
                    asynchronous: false,
                    returns: "当前求解快照或结构化失败；现有执行为同步，无任务取消承诺",
                },
            ],
        })
    }
}
