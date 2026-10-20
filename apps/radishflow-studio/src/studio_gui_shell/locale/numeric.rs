use super::StudioShellLocale;
use rf_types::units::{ContextRequirement, ConversionError};
use rf_ui::{NumericEditError, NumericFieldIssue, NumericParseError};

impl StudioShellLocale {
    pub(in super::super) fn numeric_issue(self, issue: &NumericFieldIssue) -> String {
        match issue {
            NumericFieldIssue::Incomplete => self.numeric_error(&NumericEditError::Incomplete),
            NumericFieldIssue::Conflict => self.numeric_error(&NumericEditError::FieldConflict),
            NumericFieldIssue::Rejected(error) => self.numeric_error(error),
        }
    }

    pub(in super::super) fn numeric_error(self, error: &NumericEditError) -> String {
        use NumericEditError::*;
        let zh = self == Self::ZhCn;
        match error {
            Incomplete => if zh { "输入未完成" } else { "Input incomplete" }.into(),
            FieldConflict => if zh {
                "已提交值或来源已改变，请选择如何保留本次编辑"
            } else {
                "Committed value or source changed; choose how to preserve this edit"
            }.into(),
            Invalid(NumericParseError::Syntax) => if zh {
                "数值格式无效。请输入单个数值，例如 1.25 或 1e-3；可附带匹配的单位。"
            } else {
                "Invalid number. Enter one value, such as 1.25 or 1e-3, optionally followed by a compatible unit."
            }.into(),
            Invalid(NumericParseError::UnitConflict { selected, suffix }) => {
                let selected = selected.definition().symbol;
                let suffix = suffix.definition().symbol;
                if zh {
                    format!("输入单位冲突：已选择 {selected}，文本后缀为 {suffix}。请修改后缀使其一致，或移除后缀后按 {selected} 输入数值。")
                } else {
                    format!("Unit conflict: selected {selected}, but the text uses {suffix}. Match the suffix to {selected}, or remove it and enter the value in {selected}.")
                }
            }
            Invalid(NumericParseError::Conversion(error)) | Conversion(error) => self.numeric_conversion_error(error),
            Lookup(_) => if zh {
                "此字段已不属于当前可用对象。请重新选择对象后编辑。"
            } else {
                "This field is no longer available in the current object. Select the object again before editing."
            }.into(),
            UnsupportedField => if zh {
                "此字段不支持数值编辑。"
            } else {
                "This field does not support numeric editing."
            }.into(),
            MissingSession | StaleGeneration { .. } => if zh {
                "本次编辑已过期。请重新选择字段后重试。"
            } else {
                "This edit is no longer current. Select the field again and retry."
            }.into(),
            Rejected(_) => if zh {
                "无法应用此值。请检查字段允许范围及关联对象条件；具体原因见诊断详情。"
            } else {
                "Cannot apply this value. Check the field limits and related objects; see Diagnostic details for the specific reason."
            }.into(),
        }
    }

    fn numeric_conversion_error(self, error: &ConversionError) -> String {
        let zh = self == Self::ZhCn;
        match error {
            ConversionError::UnknownUnit { token } => {
                if zh {
                    format!("无法识别单位“{token}”。请用单位菜单中列出的符号修改后缀，或删除后缀。")
                } else {
                    format!(
                        "Unknown unit “{token}”. Replace the suffix with a symbol listed in the unit menu, or remove the suffix."
                    )
                }
            }
            ConversionError::AmbiguousUnit { token } => {
                if zh {
                    format!(
                        "单位“{token}”的含义不明确。请改用明确的单位标识，或删除后缀后从菜单选择。"
                    )
                } else {
                    format!(
                        "Unit “{token}” is ambiguous. Use an explicit unit ID, or remove the suffix and choose a unit from the menu."
                    )
                }
            }
            ConversionError::IncompatibleUnit { quantity, unit } => {
                let quantity = self.quantity_name(*quantity);
                let unit = unit.definition().symbol;
                if zh {
                    format!("{unit} 不能用于{quantity}。请修改或删除单位后缀；可用单位见菜单。")
                } else {
                    format!(
                        "{unit} cannot be used for {quantity}. Correct or remove the unit suffix; compatible units are listed in the menu."
                    )
                }
            }
            ConversionError::ContextRequired { unit, requirement } => {
                let unit = unit.definition().symbol;
                let context = match (zh, requirement) {
                    (true, ContextRequirement::ReferencePressure) => "参考压力",
                    (true, ContextRequirement::MixtureMolarMass) => "混合物摩尔质量",
                    (true, ContextRequirement::StandardVolumeBasisAndStateRelation) => {
                        "标准体积基准和状态关系"
                    }
                    (false, ContextRequirement::ReferencePressure) => "a reference pressure",
                    (false, ContextRequirement::MixtureMolarMass) => "the mixture molar mass",
                    (false, ContextRequirement::StandardVolumeBasisAndStateRelation) => {
                        "a standard volume basis and state relation"
                    }
                };
                if zh {
                    format!(
                        "{unit} 的换算需要{context}，当前不支持。请改用菜单中支持的单位符号，并输入相应数值。"
                    )
                } else {
                    format!(
                        "Converting {unit} requires {context} and is not supported yet. Use a supported unit symbol from the menu and enter the value in that unit."
                    )
                }
            }
            ConversionError::NonFiniteValue => if zh {
                "请输入有限数值；不接受 NaN 或无穷大。"
            } else {
                "Enter a finite number; NaN and infinity are not accepted."
            }
            .into(),
            ConversionError::Overflow => if zh {
                "单位换算结果超出数值范围。请减小数值或指数。"
            } else {
                "The converted value is outside the numeric range. Reduce the value or exponent."
            }
            .into(),
        }
    }
}
