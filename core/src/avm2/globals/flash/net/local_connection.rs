use crate::avm2::error::{argument_error, type_error};
use crate::avm2::multiname::Multiname;
use crate::avm2::object::TObject;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::{Activation, Avm2, Error, Object, Value};
use crate::string::AvmString;

use crate::avm2_stub_method;

fn get_connection_list<'gc>(
    activation: &mut Activation<'_, 'gc>,
) -> Object<'gc> {
    activation
        .avm2()
        .classes()
        .localconnection
        .get_property(
            &Multiname::new(activation.avm2().flash_net_internal, "_connections"),
            activation,
        )
        .unwrap()
        .as_object()
        .unwrap()
}

fn does_connection_exist<'gc>(
    activation: &mut Activation<'_, 'gc>,
    name: AvmString<'gc>,
) -> bool {
    let connection_list = get_connection_list(activation);

    let array_storage = connection_list
        .as_array_storage_mut(activation.context.gc_context)
        .unwrap();

    let position = array_storage.iter().position(|a| a == Some(name.into()));

    position.is_some()
}

fn add_connection_lookup<'gc>(
    activation: &mut Activation<'_, 'gc>,
    name: AvmString<'gc>,
) -> i32 {
    let connection_list = get_connection_list(activation);

    let mut array_storage = connection_list
        .as_array_storage_mut(activation.context.gc_context)
        .unwrap();

    array_storage.push(name.into());

    (array_storage.length() - 1) as i32
}

fn remove_connection_lookup<'gc>(
    activation: &mut Activation<'_, 'gc>,
    index: i32,
) {
    let connection_list = get_connection_list(activation);

    let mut array_storage = connection_list
        .as_array_storage_mut(activation.context.gc_context)
        .unwrap();

    array_storage.remove(index);
}

/// Implements `domain` getter
pub fn get_domain<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let movie = activation.context.swf;

    let domain = if let Ok(url) = url::Url::parse(movie.url()) {
        if url.scheme() == "file" {
            "localhost".into()
        } else if let Some(domain) = url.domain() {
            AvmString::new_utf8(activation.context.gc_context, domain)
        } else {
            // no domain?
            "localhost".into()
        }
    } else {
        tracing::error!("LocalConnection::domain: Unable to parse movie URL");
        return Ok(Value::Null);
    };

    Ok(Value::String(domain))
}

/// Implements `LocalConnection.send`
pub fn send<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let connection_name = args.get_value(0);

    if matches!(connection_name, Value::Null) {
        return Err(Error::AvmError(type_error(
            activation,
            "Error #2007: Parameter connectionName must be non-null.",
            2007,
        )?));
    }

    if matches!(args.get_value(1), Value::Null) {
        return Err(Error::AvmError(type_error(
            activation,
            "Error #2007: Parameter methodName must be non-null.",
            2007,
        )?));
    }
    
    let connection_name = connection_name.coerce_to_string(activation)?;

    let event_name = if does_connection_exist(activation, connection_name) {
        "status"
    } else {
        "error"
    };

    let event = activation.avm2().classes().statusevent.construct(
        activation,
        &[
            "status".into(),
            false.into(),
            false.into(),
            Value::Null,
            event_name.into(),
        ],
    )?;

    // This must be asynchronous to match FP
    // NOTE: Currently broken due to lifetime issues
    let navigator = activation.context.navigator;
    let mut ctx = activation.context.reborrow();
    let future = Box::pin(async move {
        Ok(Avm2::dispatch_event(&mut ctx, event, this))
    });

    navigator.spawn_future(future);

    Ok(Value::Undefined)
}

/// Implements `LocalConnection.connect`
pub fn connect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    mut this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let connection_name = args.get_value(0);
    if matches!(connection_name, Value::Null) {
        return Err(Error::AvmError(type_error(
            activation,
            "Error #2007: Parameter connectionName must be non-null.",
            2007,
        )?));
    };

    let current_connection_id = this
        .get_property(
            &Multiname::new(activation.avm2().flash_net_internal, "_connectionIndex"),
            activation,
        )
        .unwrap()
        .coerce_to_i32(activation)?;

    if current_connection_id != -1 {
        return Err(Error::AvmError(argument_error(
            activation,
            "Error #2082: Connect failed because the object is already connected.",
            2082,
        )?));
    };

    avm2_stub_method!(activation, "flash.net.LocalConnection", "connect");

    let connection_name = connection_name.coerce_to_string(activation)?;

    let id = add_connection_lookup(activation, connection_name);

    this.set_property(
        &Multiname::new(activation.avm2().flash_net_internal, "_connectionIndex"),
        id.into(),
        activation,
    )?;

    Ok(Value::Undefined)
}

/// Implements `LocalConnection.close`
pub fn close<'gc>(
    activation: &mut Activation<'_, 'gc>,
    mut this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let connection = this
        .get_property(
            &Multiname::new(activation.avm2().flash_net_internal, "_connectionIndex"),
            activation,
        )
        .unwrap();

    // -1 means not connected
    if connection.coerce_to_i32(activation)? == -1 {
        return Err(Error::AvmError(argument_error(
            activation,
            "Error #2083: Close failed because the object is not connected.",
            2083,
        )?));
    };

    let connection_id = connection.coerce_to_i32(activation)?;

    this.set_property(
        &Multiname::new(activation.avm2().flash_net_internal, "_connectionIndex"),
        (-1).into(),
        activation,
    )?;
    
    remove_connection_lookup(activation, connection_id);

    Ok(Value::Undefined)
}
