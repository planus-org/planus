use crate::analysis::{DeclarationAnalysis, WasChanged};
use planus_types::intermediate::{
    AssignMode, Declaration, DeclarationIndex, DeclarationKind, Declarations, SimpleType, TypeKind,
};

pub struct DefaultAnalysis;
impl DeclarationAnalysis for DefaultAnalysis {
    type State = bool;

    fn new_state(
        &mut self,
        _declarations: &Declarations,
        _decl_id: DeclarationIndex,
        declaration: &Declaration,
    ) -> Self::State {
        match declaration.kind {
            DeclarationKind::Table(_) | DeclarationKind::Struct(_) => true,
            DeclarationKind::Enum(_)
            | DeclarationKind::Union(_)
            | DeclarationKind::RpcService(_) => false,
        }
    }

    fn update_state(
        &mut self,
        _declarations: &Declarations,
        decl_id: DeclarationIndex,
        declaration: &Declaration,
        defaults_possible: &mut [Self::State],
    ) -> WasChanged {
        if !defaults_possible[decl_id.0] {
            return WasChanged::NoChange;
        }
        let mut cur_default_possible = defaults_possible[decl_id.0];

        match &declaration.kind {
            DeclarationKind::Table(decl) => {
                for field in decl.fields.values() {
                    if matches!(
                        field.assign_mode,
                        AssignMode::Optional | AssignMode::HasDefault(_)
                    ) {
                        continue;
                    }

                    match field.type_.kind {
                        TypeKind::Table(decl_id)
                        | TypeKind::SimpleType(SimpleType::Struct(decl_id)) => {
                            if !defaults_possible[decl_id.0] {
                                cur_default_possible = false;
                                break;
                            }
                        }
                        _ => {
                            // For other types, we can only do default if our assign_mode matches
                            cur_default_possible = false;
                            break;
                        }
                    }
                }
            }
            DeclarationKind::Struct(decl) => {
                for field in decl.fields.values() {
                    match &field.type_ {
                        SimpleType::Struct(decl_id) => {
                            if !defaults_possible[decl_id.0] {
                                cur_default_possible = false;
                                break;
                            }
                        }
                        SimpleType::Enum(_) => cur_default_possible = false,
                        SimpleType::Bool | SimpleType::Integer(_) | SimpleType::Float(_) => (),
                    }
                }
            }
            DeclarationKind::Enum(_)
            | DeclarationKind::Union(_)
            | DeclarationKind::RpcService(_) => {
                cur_default_possible = false;
            }
        }

        if cur_default_possible != defaults_possible[decl_id.0] {
            defaults_possible[decl_id.0] = cur_default_possible;
            WasChanged::Changed
        } else {
            WasChanged::NoChange
        }
    }
}

fn eq_possible_simple_type(eq_possible: &[bool], type_: &SimpleType) -> bool {
    match type_ {
        SimpleType::Struct(decl_id) => eq_possible[decl_id.0],
        SimpleType::Enum(_) | SimpleType::Bool | SimpleType::Integer(_) => true,
        SimpleType::Float(_) => false,
    }
}

fn eq_possible_type_kind(eq_possible: &[bool], type_: &TypeKind) -> bool {
    match type_ {
        TypeKind::Table(decl_id) | TypeKind::Union(decl_id) => eq_possible[decl_id.0],
        TypeKind::Vector(type_) => eq_possible_type_kind(eq_possible, &type_.kind),
        TypeKind::Array(type_, _) => eq_possible_type_kind(eq_possible, &type_.kind),
        TypeKind::SimpleType(type_) => eq_possible_simple_type(eq_possible, type_),
        TypeKind::String => true,
    }
}

pub struct EqAnalysis;
impl DeclarationAnalysis for EqAnalysis {
    type State = bool;

    fn new_state(
        &mut self,
        _declarations: &Declarations,
        _decl_id: DeclarationIndex,
        declaration: &Declaration,
    ) -> Self::State {
        match declaration.kind {
            DeclarationKind::Table(_)
            | DeclarationKind::Struct(_)
            | DeclarationKind::Enum(_)
            | DeclarationKind::Union(_) => true,
            DeclarationKind::RpcService(_) => false,
        }
    }

    fn update_state(
        &mut self,
        _declarations: &Declarations,
        decl_id: DeclarationIndex,
        declaration: &Declaration,
        eq_possible: &mut [Self::State],
    ) -> WasChanged {
        if !eq_possible[decl_id.0] {
            return WasChanged::NoChange;
        }
        let mut cur_eq_possible = eq_possible[decl_id.0];

        match &declaration.kind {
            DeclarationKind::Table(decl) => {
                for field in decl.fields.values() {
                    if !eq_possible_type_kind(eq_possible, &field.type_.kind) {
                        cur_eq_possible = false;
                        break;
                    }
                }
            }
            DeclarationKind::Struct(decl) => {
                for field in decl.fields.values() {
                    if !eq_possible_simple_type(eq_possible, &field.type_) {
                        cur_eq_possible = false;
                        break;
                    }
                }
            }
            DeclarationKind::Union(decl) => {
                for variant in decl.variants.values() {
                    if !eq_possible_type_kind(eq_possible, &variant.type_.kind) {
                        cur_eq_possible = false;
                        break;
                    }
                }
            }
            DeclarationKind::Enum(_) => (),
            DeclarationKind::RpcService(_) => cur_eq_possible = false,
        }

        if cur_eq_possible != eq_possible[decl_id.0] {
            eq_possible[decl_id.0] = cur_eq_possible;
            WasChanged::Changed
        } else {
            WasChanged::NoChange
        }
    }
}

fn infallible_conversion_simple_type(infallible_conversion: &[bool], type_: &SimpleType) -> bool {
    match type_ {
        SimpleType::Struct(decl_id) => infallible_conversion[decl_id.0],
        SimpleType::Bool | SimpleType::Integer(_) | SimpleType::Float(_) => true,
        SimpleType::Enum(_) => false,
    }
}

fn infallible_conversion_type_kind(infallible_conversion: &[bool], type_: &TypeKind) -> bool {
    match type_ {
        TypeKind::Union(decl_id) => infallible_conversion[decl_id.0],
        TypeKind::SimpleType(type_) => {
            infallible_conversion_simple_type(infallible_conversion, type_)
        }
        TypeKind::Table(_) | TypeKind::Array(_, _) | TypeKind::Vector(_) | TypeKind::String => {
            false
        }
    }
}
pub struct InfallibleConversionAnalysis;
impl DeclarationAnalysis for InfallibleConversionAnalysis {
    type State = bool;

    fn new_state(
        &mut self,
        _declarations: &Declarations,
        _decl_id: DeclarationIndex,
        declaration: &Declaration,
    ) -> Self::State {
        match declaration.kind {
            DeclarationKind::Struct(_) | DeclarationKind::Union(_) => true,
            DeclarationKind::Enum(_)
            | DeclarationKind::Table(_)
            | DeclarationKind::RpcService(_) => false,
        }
    }

    fn update_state(
        &mut self,
        _declarations: &Declarations,
        decl_id: DeclarationIndex,
        declaration: &Declaration,
        infallible_conversion: &mut [Self::State],
    ) -> WasChanged {
        if !infallible_conversion[decl_id.0] {
            return WasChanged::NoChange;
        }
        let mut cur_conversion_possible = infallible_conversion[decl_id.0];

        match &declaration.kind {
            DeclarationKind::Struct(decl) => {
                for field in decl.fields.values() {
                    if !infallible_conversion_simple_type(infallible_conversion, &field.type_) {
                        cur_conversion_possible = false;
                        break;
                    }
                }
            }
            DeclarationKind::Union(decl) => {
                for variant in decl.variants.values() {
                    if !infallible_conversion_type_kind(infallible_conversion, &variant.type_.kind)
                    {
                        cur_conversion_possible = false;
                        break;
                    }
                }
            }
            DeclarationKind::Table(_)
            | DeclarationKind::Enum(_)
            | DeclarationKind::RpcService(_) => {
                cur_conversion_possible = false;
            }
        }

        if cur_conversion_possible != infallible_conversion[decl_id.0] {
            infallible_conversion[decl_id.0] = cur_conversion_possible;
            WasChanged::Changed
        } else {
            WasChanged::NoChange
        }
    }
}
