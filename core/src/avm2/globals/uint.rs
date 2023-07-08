//! `uint` impl

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::error::{make_error_1002, make_error_1003, make_error_1004};
use crate::avm2::globals::number::{print_with_precision, print_with_radix};
use crate::avm2::method::{Method, NativeMethodImpl, ParamConfig};
use crate::avm2::object::{primitive_allocator, FunctionObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Multiname;
use crate::avm2::QName;
use crate::avm2::{AvmString, Error};
use gc_arena::GcCell;

/// Implements `uint`'s instance initializer.
fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut prim) = this.as_primitive_mut(activation.context.gc_context) {
        if matches!(*prim, Value::Undefined | Value::Null) {
            *prim = args
                .get(0)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_u32(activation)?
                .into();
        }
    }

    Ok(Value::Undefined)
}

/// Implements `uint`'s native instance initializer.
fn native_instance_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    activation.super_init(this, args)?;

    Ok(Value::Undefined)
}

/// Implements `uint`'s class initializer.
fn class_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let scope = activation.create_scopechain();
    let gc_context = activation.context.gc_context;
    let this_class = this.as_class_object().unwrap();
    let uint_proto = this_class.prototype();

    uint_proto.set_string_property_local(
        "toExponential",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(to_exponential, "toExponential", gc_context),
            scope,
            None,
            Some(this_class),
        )
        .into(),
        activation,
    )?;
    uint_proto.set_string_property_local(
        "toFixed",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(to_fixed, "toFixed", gc_context),
            scope,
            None,
            Some(this_class),
        )
        .into(),
        activation,
    )?;
    uint_proto.set_string_property_local(
        "toPrecision",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(to_precision, "toPrecision", gc_context),
            scope,
            None,
            Some(this_class),
        )
        .into(),
        activation,
    )?;
    uint_proto.set_string_property_local(
        "toLocaleString",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(to_string, "toLocaleString", gc_context),
            scope,
            None,
            Some(this_class),
        )
        .into(),
        activation,
    )?;
    uint_proto.set_string_property_local(
        "valueOf",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(value_of, "valueOf", gc_context),
            scope,
            None,
            Some(this_class),
        )
        .into(),
        activation,
    )?;

    uint_proto.set_local_property_is_enumerable(gc_context, "toExponential".into(), false);
    uint_proto.set_local_property_is_enumerable(gc_context, "toFixed".into(), false);
    uint_proto.set_local_property_is_enumerable(gc_context, "toPrecision".into(), false);
    uint_proto.set_local_property_is_enumerable(gc_context, "toLocaleString".into(), false);
    uint_proto.set_local_property_is_enumerable(gc_context, "valueOf".into(), false);

    Ok(Value::Undefined)
}

/// Implements `uint.toExponential`
fn to_exponential<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let number = Value::from(this).coerce_to_number(activation)?;

    let digits = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Integer(0))
        .coerce_to_u32(activation)? as usize;

    if digits > 20 {
        return Err(make_error_1002(activation));
    }

    Ok(AvmString::new_utf8(
        activation.context.gc_context,
        format!("{number:.digits$e}")
            .replace('e', "e+")
            .replace("e+-", "e-")
            .replace("e+0", ""),
    )
    .into())
}

/// Implements `uint.toFixed`
fn to_fixed<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let number = Value::from(this).coerce_to_number(activation)?;

    let digits = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Integer(0))
        .coerce_to_u32(activation)? as usize;

    if digits > 20 {
        return Err(make_error_1002(activation));
    }

    Ok(AvmString::new_utf8(
        activation.context.gc_context,
        format!("{0:.1$}", number as f64, digits),
    )
    .into())
}

/// Implements `uint.toPrecision`
fn to_precision<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let number = Value::from(this).coerce_to_number(activation)?;

    let wanted_digits = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Integer(0))
        .coerce_to_u32(activation)? as usize;

    if wanted_digits < 1 || wanted_digits > 21 {
        return Err(make_error_1002(activation));
    }

    Ok(print_with_precision(activation, number as f64, wanted_digits)?.into())
}

/// Implements `uint.toString`
fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let int_proto = activation.avm2().classes().uint.prototype();
    if Object::ptr_eq(int_proto, this) {
        return Ok("0".into());
    }

    if let Some(this) = this.as_primitive() {
        if let Value::Integer(number) = *this {
            let radix = args
                .get(0)
                .cloned()
                .unwrap_or(Value::Integer(10))
                .coerce_to_u32(activation)? as usize;

            if radix < 2 || radix > 36 {
                return Err(make_error_1003(activation, radix));
            }

            return Ok(print_with_radix(activation, number as f64, radix)?.into());
        }
    }

    Err(make_error_1004(activation, "uint.prototype.toString"))
}

/// Implements `uint.valueOf`
fn value_of<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let int_proto = activation.avm2().classes().int.prototype();
    if Object::ptr_eq(int_proto, this) {
        return Ok(0.into());
    }

    if let Some(this) = this.as_primitive() {
        return Ok(*this);
    }

    Err(make_error_1004(activation, "uint.prototype.valueOf"))
}

/// Construct `uint`'s class.
pub fn create_class<'gc>(activation: &mut Activation<'_, 'gc>) -> GcCell<'gc, Class<'gc>> {
    let mc = activation.context.gc_context;
    let class = Class::new(
        QName::new(activation.avm2().public_namespace, "uint"),
        Some(Multiname::new(activation.avm2().public_namespace, "Object")),
        Method::from_builtin_and_params(
            instance_init,
            "<uint instance initializer>",
            vec![ParamConfig {
                param_name: AvmString::new_utf8(activation.context.gc_context, "value"),
                param_type_name: Multiname::any(activation.context.gc_context),
                default_value: Some(Value::Integer(0)),
            }],
            false,
            mc,
        ),
        Method::from_builtin(class_init, "<uint class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);
    write.set_attributes(ClassAttributes::FINAL | ClassAttributes::SEALED);
    write.set_instance_allocator(primitive_allocator);
    write.set_native_instance_init(Method::from_builtin(
        native_instance_init,
        "<uint native instance initializer>",
        mc,
    ));

    const CLASS_CONSTANTS_UINT: &[(&str, u32)] =
        &[("MAX_VALUE", u32::MAX), ("MIN_VALUE", u32::MIN)];
    write.define_constant_uint_class_traits(
        activation.avm2().public_namespace,
        CLASS_CONSTANTS_UINT,
        activation,
    );

    const CLASS_CONSTANTS_INT: &[(&str, i32)] = &[("length", 1)];
    write.define_constant_int_class_traits(
        activation.avm2().public_namespace,
        CLASS_CONSTANTS_INT,
        activation,
    );

    const AS3_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("toExponential", to_exponential),
        ("toFixed", to_fixed),
        ("toPrecision", to_precision),
        ("toString", to_string),
        ("valueOf", value_of),
    ];
    write.define_builtin_instance_methods(
        mc,
        activation.avm2().as3_namespace,
        AS3_INSTANCE_METHODS,
    );

    class
}
