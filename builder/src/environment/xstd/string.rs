use xelis_environment::Context;
use xelis_types::{path_as_ref, Type, Value, ValuePointer};
use super::{
    FnInstance,
    FnParams,
    FnReturnType,
    EnvironmentBuilder
};

pub fn register(env: &mut EnvironmentBuilder) {
    // String
    env.register_native_function("len", Some(Type::String), vec![], len, 1, Some(Type::U32));
    env.register_native_function("trim", Some(Type::String), vec![], trim, 1, Some(Type::String));
    env.register_native_function("contains", Some(Type::String), vec![Type::String], contains, 1, Some(Type::Bool));
    env.register_native_function("contains_ignore_case", Some(Type::String), vec![Type::String], contains_ignore_case, 1, Some(Type::Bool));
    env.register_native_function("to_uppercase", Some(Type::String), vec![], to_uppercase, 1, Some(Type::String));
    env.register_native_function("to_lowercase", Some(Type::String), vec![], to_lowercase, 1, Some(Type::String));
    env.register_native_function("to_bytes", Some(Type::String), vec![], to_bytes, 5, Some(Type::Array(Box::new(Type::U8))));
    env.register_native_function("index_of", Some(Type::String), vec![Type::String], index_of, 3, Some(Type::Optional(Box::new(Type::U32))));
    env.register_native_function("last_index_of", Some(Type::String), vec![Type::String], last_index_of, 3, Some(Type::Optional(Box::new(Type::U32))));
    env.register_native_function("replace", Some(Type::String), vec![Type::String, Type::String], replace, 5, Some(Type::String));
    env.register_native_function("starts_with", Some(Type::String), vec![Type::String], starts_with, 3, Some(Type::Bool));
    env.register_native_function("ends_with", Some(Type::String), vec![Type::String], ends_with, 3, Some(Type::Bool));
    env.register_native_function("split", Some(Type::String), vec![Type::String], split, 5, Some(Type::Array(Box::new(Type::String))));
    env.register_native_function("char_at", Some(Type::String), vec![Type::U32], char_at, 1, Some(Type::Optional(Box::new(Type::String))));

    env.register_native_function("is_empty", Some(Type::String), vec![], is_empty, 1, Some(Type::Bool));
    env.register_native_function("matches", Some(Type::String), vec![Type::String], string_matches, 50, Some(Type::Array(Box::new(Type::String))));
    env.register_native_function("substring", Some(Type::String), vec![Type::U32], string_substring, 3, Some(Type::Optional(Box::new(Type::String))));
    env.register_native_function("substring", Some(Type::String), vec![Type::U32, Type::U32], string_substring_range, 3, Some(Type::Optional(Box::new(Type::String))));
}

fn len(zelf: FnInstance, _: FnParams, _: &mut Context) -> FnReturnType {
    let s: &String = zelf?.as_string()?;
    Ok(Some(Value::U32(s.len() as u32)))
}

fn trim(zelf: FnInstance, _: FnParams, _: &mut Context) -> FnReturnType {
    let s = zelf?.as_string()?.trim().to_string();
    Ok(Some(Value::String(s)))
}

fn contains(zelf: FnInstance, mut parameters: FnParams, _: &mut Context) -> FnReturnType {
    let param = parameters.remove(0);
    path_as_ref!(param, p, {
        let value = p.as_string()?;
        let s = zelf?.as_string()?;
        Ok(Some(Value::Boolean(s.contains(value))))
    })
}

fn contains_ignore_case(zelf: FnInstance, mut parameters: FnParams, _: &mut Context) -> FnReturnType {
    let param = parameters.remove(0);

    let z = zelf?.as_string()?.to_lowercase();
    let p: String = path_as_ref!(param, p, p.as_string()?.to_lowercase());
    Ok(Some(Value::Boolean(z.contains(&p))))
}

fn to_uppercase(zelf: FnInstance, _: FnParams, _: &mut Context) -> FnReturnType {
    let s: String = zelf?.as_string()?.to_uppercase();
    Ok(Some(Value::String(s)))
}

fn to_lowercase(zelf: FnInstance, _: FnParams, _: &mut Context) -> FnReturnType {
    let s: String = zelf?.as_string()?.to_lowercase();
    Ok(Some(Value::String(s)))
}

fn to_bytes(zelf: FnInstance, _: FnParams, _: &mut Context) -> FnReturnType {
    let s: &String = zelf?.as_string()?;

    let mut bytes = Vec::new();
    for b in s.as_bytes() {
        bytes.push(ValuePointer::owned(Value::U8(*b)));
    }

    Ok(Some(Value::Array(bytes)))
}

fn index_of(zelf: FnInstance, mut parameters: FnParams, _: &mut Context) -> FnReturnType {
    let s: &String = zelf?.as_string()?;
    let param = parameters.remove(0);
    let index = path_as_ref!(param, p, s.find(p.as_string()?));

    if let Some(index) = index {
        let inner = ValuePointer::owned(Value::U32(index as u32));
        Ok(Some(Value::Optional(Some(inner))))
    } else {
        Ok(Some(Value::Optional(None)))
    }
}

fn last_index_of(zelf: FnInstance, mut parameters: FnParams, _: &mut Context) -> FnReturnType {
    let s: &String = zelf?.as_string()?;
    let param = parameters.remove(0);
    let result = path_as_ref!(param, p, s.rfind(p.as_string()?));
    if let Some(index) = result {
        let inner = ValuePointer::owned(Value::U32(index as u32));
        Ok(Some(Value::Optional(Some(inner))))
    } else {
        Ok(Some(Value::Optional(None)))
    }
}

fn replace(zelf: FnInstance, mut parameters: FnParams, _: &mut Context) -> FnReturnType {
    let s: &String = zelf?.as_string()?;
    let param1 = parameters.remove(0);
    let param2 = parameters.remove(0);

    let s = path_as_ref!(param1, p1, path_as_ref!(param2, p2, s.replace(p1.as_string()?, p2.as_string()?)));
    Ok(Some(Value::String(s)))
}

fn starts_with(zelf: FnInstance, mut parameters: FnParams, _: &mut Context) -> FnReturnType {
    let s: &String = zelf?.as_string()?;
    let param = parameters.remove(0);
    let result = path_as_ref!(param, p, s.starts_with(p.as_string()?));
    Ok(Some(Value::Boolean(result)))
}

fn ends_with(zelf: FnInstance, mut parameters: FnParams, _: &mut Context) -> FnReturnType {
    let s: &String = zelf?.as_string()?;
    let param = parameters.remove(0);
    let result = path_as_ref!(param, p, s.ends_with(p.as_string()?));
    Ok(Some(Value::Boolean(result)))
}

fn split(zelf: FnInstance, mut parameters: FnParams, _: &mut Context) -> FnReturnType {
    let s: &String = zelf?.as_string()?;
    let param = parameters.remove(0);
    let values = path_as_ref!(param, handle, {
        let value = handle.as_string()?;
        s.split(value)
            .map(|s| ValuePointer::owned(Value::String(s.to_string())))
            .collect()
    });
    Ok(Some(Value::Array(values)))
}

fn char_at(zelf: FnInstance, mut parameters: FnParams, _: &mut Context) -> FnReturnType {
    let param =  parameters.remove(0);
    let index = path_as_ref!(param, p, p.as_u32()?) as usize;
    let s: &String = zelf?.as_string()?;
    if let Some(c) = s.chars().nth(index) {
        let inner = ValuePointer::owned(Value::String(c.to_string()));
        Ok(Some(Value::Optional(Some(inner))))
    } else {
        Ok(Some(Value::Optional(None)))
    }
}

fn is_empty(zelf: FnInstance, _: FnParams, _: &mut Context) -> FnReturnType {
    let s: &String = zelf?.as_string()?;
    Ok(Some(Value::Boolean(s.is_empty())))
}

fn string_matches(zelf: FnInstance, mut parameters: FnParams, _: &mut Context) -> FnReturnType {
    let s: &String = zelf?.as_string()?;
    let param = parameters.remove(0);
    let values = path_as_ref!(param, handle, {
        let value = handle.as_string()?;
        s.matches(value)
            .map(|s| ValuePointer::owned(Value::String(s.to_string())))
            .collect()
    });
    Ok(Some(Value::Array(values)))
}

fn string_substring(zelf: FnInstance, mut parameters: FnParams, _: &mut Context) -> FnReturnType {
    let s: &String = zelf?.as_string()?;
    let param = parameters.remove(0);
    let start = path_as_ref!(param, p, p.as_u32()?) as usize;
    if let Some(s) = s.get(start..) {
        let inner = ValuePointer::owned(Value::String(s.to_owned()));
        Ok(Some(Value::Optional(Some(inner))))
    } else {
        Ok(Some(Value::Optional(None)))
    }
}

fn string_substring_range(zelf: FnInstance, mut parameters: FnParams, _: &mut Context) -> FnReturnType {
    let s: &String = zelf?.as_string()?;
    let param1 = parameters.remove(0);
    let param2 = parameters.remove(0);
    let start = path_as_ref!(param1, p, p.as_u32()?) as usize;
    let end = path_as_ref!(param2, p, p.as_u32()?) as usize;
    if let Some(s) = s.get(start..end) {
        let inner = ValuePointer::owned(Value::String(s.to_owned()));
        Ok(Some(Value::Optional(Some(inner))))
    } else {
        Ok(Some(Value::Optional(None)))
    }
}