//! Shell field identity shared by binding widgets and focus restoration.
use super::*;
use rf_model::equipment::SeparatorRatingInput;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum EquipmentInputField {
    Sizing(EquipmentField),
    Geometry(RatingGeometryField),
    Rating(SeparatorRatingInput),
}
impl From<EquipmentField> for EquipmentInputField {
    fn from(field: EquipmentField) -> Self {
        Self::Sizing(field)
    }
}
impl EquipmentInputField {
    pub(super) fn quantity(self) -> QuantityKind {
        match self {
            Self::Sizing(f) => f.quantity(),
            Self::Geometry(f) => f.quantity(),
            Self::Rating(f) => f.quantity(),
        }
    }
    pub(super) fn label(self, locale: StudioShellLocale) -> &'static str {
        match self {
            Self::Sizing(f) => field_label(locale, f),
            Self::Geometry(RatingGeometryField::Input(f)) => geometry_field_label(locale, f),
            Self::Geometry(RatingGeometryField::Axial(f)) => {
                field_label(locale, EquipmentField::Axial(f))
            }
            Self::Rating(f) => rating_field_label(locale, f),
        }
    }
}
