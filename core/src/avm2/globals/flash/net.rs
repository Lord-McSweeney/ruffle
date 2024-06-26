//! `flash.net` namespace

use crate::avm2::error::make_error_2007;
use crate::avm2::object::TObject;
use crate::avm2::{Activation, Error, Object, Value};
use crate::backend::navigator::NavigationMethod;
use indexmap::IndexMap;

pub mod file_reference;
pub mod local_connection;
pub mod net_connection;
pub mod net_stream;
pub mod object_encoding;
pub mod responder;
pub mod shared_object;
pub mod socket;
pub mod url_loader;
pub mod xml_socket;

fn object_to_index_map<'gc>(
    activation: &mut Activation<'_, 'gc>,
    obj: &Object<'gc>,
) -> Result<IndexMap<String, String>, Error<'gc>> {
    let mut map = IndexMap::new();
    let mut last_index = obj.get_next_enumerant(0, activation)?;
    while let Some(index) = last_index {
        let name = obj
            .get_enumerant_name(index, activation)?
            .coerce_to_string(activation)?;
        let value = obj
            .get_public_property(name, activation)?
            .coerce_to_string(activation)?
            .to_utf8_lossy()
            .to_string();

        let name = name.to_utf8_lossy().to_string();
        map.insert(name, value);
        last_index = obj.get_next_enumerant(index, activation)?;
    }
    Ok(map)
}

fn parse_data<'gc>(
    activation: &mut Activation<'_, 'gc>,
    url: &String,
    data: &Value<'gc>,
) -> Result<(String, IndexMap<String, String>), Error<'gc>> {
    let mut url = url.to_string();
    let mut vars = IndexMap::new();
    let urlvariables = activation
        .avm2()
        .classes()
        .urlvariables
        .inner_class_definition();
    if data.is_of_type(activation, urlvariables) {
        let obj = data.coerce_to_object(activation)?;
        vars = object_to_index_map(activation, &obj).unwrap_or_default();
    } else if *data != Value::Null {
        let str_data = data.coerce_to_string(activation)?.to_string();
        if !url.contains('?') {
            url.push('?');
        }
        url.push_str(&str_data);
    }
    Ok((url, vars))
}

/// Implements `flash.net.navigateToURL`
pub fn navigate_to_url<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let request = args
        .get(0)
        .ok_or("navigateToURL: not enough arguments")?
        .coerce_to_object(activation)?;

    let target = args
        .get(1)
        .ok_or("navigateToURL: not enough arguments")?
        .coerce_to_string(activation)?;

    match request.get_public_property("url", activation)? {
        Value::Null => Err(make_error_2007(activation, "url")),
        url => {
            let url = url.coerce_to_string(activation)?.to_string();
            let method = request
                .get_public_property("method", activation)?
                .coerce_to_string(activation)?;
            let method = NavigationMethod::from_method_str(&method).unwrap();
            let data: Value<'gc> = request.get_public_property("data", activation)?;
            let (url, vars) = parse_data(activation, &url, &data)?;
            activation.context.navigator.navigate_to_url(
                &url,
                &target.to_utf8_lossy(),
                Some((method, vars)),
            );
            Ok(Value::Undefined)
        }
    }
}
