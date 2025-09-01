use tanitc_lexer::location::Location;
use tanitc_messages::Message;
use tanitc_ty::Type;

use crate::Analyzer;

impl Analyzer {
    pub fn compare_types(
        &self,
        lhs_type: &Type,
        rhs_type: &Type,
        location: Location,
    ) -> Result<(), Message> {
        let mut alias_to = self.find_alias_value(lhs_type);

        if lhs_type == rhs_type {
            alias_to = None;
        }

        if alias_to.is_none()
            && lhs_type != rhs_type
            && !self.try_coerce(rhs_type, lhs_type, location)
        {
            return Err(Message::from_string(
                    location,
                    format!(
                        "Cannot perform operation on objects with different types: {lhs_type} and {rhs_type}",
                    ),
                ));
        } else if alias_to.as_ref().is_some_and(|ty| rhs_type != ty)
            && !self.try_coerce(rhs_type, lhs_type, location)
        {
            return Err(Message::from_string(
                    location,
                    format!(
                        "Cannot perform operation on objects with different types: {lhs_type} (aka: {}) and {rhs_type}",
                        alias_to.unwrap()
                    ),
                ));
        }

        Ok(())
    }
}

impl Analyzer {
    fn check_ref_coerce_to_ptr(
        &self,
        src_type: &Type,
        dst_type: &Type,
        location: Location,
    ) -> Result<(), Message> {
        let Type::Ref { ref_to, .. } = src_type else {
            return Err(Message::unreachable(
                location,
                format!("dst_type expected to be reference, actually: {src_type}"),
            ));
        };

        let Type::Ptr(ptr_to) = dst_type else {
            return Err(Message::unreachable(
                location,
                format!("src_type expected to be pointer, actually: {dst_type}"),
            ));
        };

        self.compare_types(ref_to, ptr_to.as_ref(), location)
    }

    fn check_array_types(
        &self,
        src_type: &Type,
        dst_type: &Type,
        location: Location,
    ) -> Result<(), Message> {
        let Type::Array {
            value_type: src_type,
            ..
        } = src_type
        else {
            return Err(Message::unreachable(
                location,
                format!("dst_type expected to be reference, actually: {src_type}"),
            ));
        };

        let Type::Array {
            value_type: dst_type,
            ..
        } = dst_type
        else {
            return Err(Message::unreachable(
                location,
                format!("src_type expected to be pointer, actually: {dst_type}"),
            ));
        };

        self.compare_types(dst_type, src_type, location)
    }

    // Returns true if src_type can be coerced to dst_type, otherwise - false
    fn try_coerce(&self, src_type: &Type, dst_type: &Type, location: Location) -> bool {
        if src_type.is_reference() && dst_type.is_pointer() {
            return self
                .check_ref_coerce_to_ptr(src_type, dst_type, location)
                .is_ok();
        }

        if src_type.is_array() && dst_type.is_array() {
            return self.check_array_types(src_type, dst_type, location).is_ok();
        }

        false
    }
}
