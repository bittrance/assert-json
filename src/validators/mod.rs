use crate::{get_value_type_id, Error, Validator, Value};
use std::fmt::Debug;

mod array;
mod object;
mod primitive;

pub use array::*;
pub use object::*;
pub use primitive::*;

/// Match any value.
///
/// It will never return an error.
pub fn any() -> impl Validator {
    AnyValidator {}
}

struct AnyValidator {}

impl Validator for AnyValidator {
    fn validate<'a>(&self, _: &'a Value) -> Result<(), Error<'a>> {
        Ok(())
    }
}

/// Match a value equals the expected value.
pub fn eq<T>(expected: T) -> impl Validator
where
    T: Into<Value> + Clone + Debug + 'static,
{
    EqValidator { expected }
}

struct EqValidator<T>
where
    T: Into<Value> + Clone + Debug,
{
    expected: T,
}

impl<T> Validator for EqValidator<T>
where
    T: Into<Value> + Clone + Debug,
{
    fn validate<'a>(&self, value: &'a Value) -> Result<(), Error<'a>> {
        let expected_val = self.expected.clone().into();
        if get_value_type_id(&expected_val) != get_value_type_id(value) {
            return Err(Error::InvalidType(value, get_value_type_id(&expected_val)));
        }

        if value == &expected_val {
            Ok(())
        } else {
            Err(Error::InvalidValue(value, format!("{:?}", self.expected)))
        }
    }
}

#[doc(hidden)]
macro_rules! impl_from_validator_default {
    (
        $($ty:ty),*
    ) => {
        $(
            impl From<$ty> for Box<dyn Validator> {
                #[inline]
                fn from(u: $ty) -> Self {
                    Box::new(eq(u))
                }
            }
        )*
    };
}

impl_from_validator_default!(
    String, bool, u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, f32, f64
);

impl From<&str> for Box<dyn Validator> {
    fn from(str_input: &str) -> Self {
        Box::new(eq(String::from(str_input)))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Error, Validator, Value};

    #[test]
    fn any() {
        let validator = super::any();

        assert_eq!(Ok(()), validator.validate(&Value::Null));
    }

    #[test]
    fn eq_string() {
        let validator = super::eq("test");

        assert_eq!(Ok(()), validator.validate(&serde_json::json!("test")))
    }

    #[test]
    fn eq_string_fail() {
        let validator = super::eq(String::from("test"));

        assert!(matches!(
            validator.validate(&serde_json::json!("not expected")),
            Err(Error::InvalidValue(_, _))
        ));
    }

    #[test]
    fn primitive_type_validation() {
        let validator: Box<dyn Validator> = 4.into();
        assert_eq!(Ok(()), validator.validate(&serde_json::json!(4)))
    }
}
