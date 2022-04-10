use crate::*;

#[fp_export_signature]
pub fn dispatch(name: String, args: Vec<RawValue>) -> Option<RawValue>;
