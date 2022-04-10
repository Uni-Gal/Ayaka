use gal_bindings::*;
use rt_format::*;
use std::collections::HashMap;

pub struct Guest;

impl Export for Guest {
    fn dispatch(name: String, args: Vec<RawValue>) -> Option<RawValue> {
        match name.as_str() {
            "fmt" => Some(fmt(&args)),
            _ => None,
        }
    }
}

decl_dispatch!(Guest);

struct ValueWrap<'a>(&'a RawValue);

impl FormatArgument for ValueWrap<'_> {
    fn supports_format(&self, specifier: &Specifier) -> bool {
        match self.0 {
            RawValue::Unit | RawValue::Bool(_) | RawValue::Str(_) => match specifier.format {
                Format::Debug | Format::Display => true,
                _ => false,
            },
            RawValue::Num(_) => true,
        }
    }

    fn fmt_display(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::fmt::Display;
        match self.0 {
            RawValue::Unit => Ok(()),
            RawValue::Bool(b) => b.fmt(f),
            RawValue::Num(n) => n.fmt(f),
            RawValue::Str(s) => s.fmt(f),
        }
    }

    fn fmt_debug(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::fmt::Debug;
        self.0.fmt(f)
    }

    fn fmt_octal(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            RawValue::Num(n) => std::fmt::Octal::fmt(n, f),
            _ => Err(std::fmt::Error),
        }
    }

    fn fmt_lower_hex(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            RawValue::Num(n) => std::fmt::LowerHex::fmt(n, f),
            _ => Err(std::fmt::Error),
        }
    }

    fn fmt_upper_hex(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            RawValue::Num(n) => std::fmt::UpperHex::fmt(n, f),
            _ => Err(std::fmt::Error),
        }
    }

    fn fmt_binary(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            RawValue::Num(n) => std::fmt::Binary::fmt(n, f),
            _ => Err(std::fmt::Error),
        }
    }

    fn fmt_lower_exp(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            RawValue::Num(n) => std::fmt::LowerExp::fmt(n, f),
            _ => Err(std::fmt::Error),
        }
    }

    fn fmt_upper_exp(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            RawValue::Num(n) => std::fmt::UpperExp::fmt(n, f),
            _ => Err(std::fmt::Error),
        }
    }
}

fn fmt(args: &[RawValue]) -> RawValue {
    if args.is_empty() {
        RawValue::Unit
    } else {
        ParsedFormat::parse(
            &args[0].get_str(),
            &args[1..].iter().map(|v| ValueWrap(v)).collect::<Vec<_>>(),
            &HashMap::<String, ValueWrap>::new(),
        )
        .map(|r| RawValue::Str(r.to_string()))
        .unwrap_or_default()
    }
}
