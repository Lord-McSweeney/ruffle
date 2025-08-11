use crate::avm2::error::argument_error;
use crate::avm2::globals::flash::geom::transform::matrix3d_to_object;
use crate::avm2::globals::slots::flash_geom_point as point_slots;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::{Activation, Error, TObject as _, Value};
use crate::avm2_stub_setter;
use ruffle_render::perspective_projection::PerspectiveProjection;

pub use crate::avm2::object::perspective_projection_allocator;

pub fn get_focal_length<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let this = this.as_perspective_projection().unwrap();

    let width = this.get_width(activation.context);
    let focal_length = this.perspective_projection().focal_length(width as f32);

    Ok(focal_length.into())
}

pub fn set_focal_length<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // FIXME: Render with the given PerspectiveProjection.
    avm2_stub_setter!(
        activation,
        "flash.geom.PerspectiveProjection",
        "focalLength"
    );
    let this = this.as_object().unwrap();
    let this = this.as_perspective_projection().unwrap();

    let focal_length = args.get_f64(0);
    if focal_length <= 0.0 {
        return Err(Error::avm_error(argument_error(
            activation,
            &format!("Error #2186: Invalid focalLength {focal_length}."),
            2186,
        )?));
    }

    this.sync_from_display_object();

    let width = this.get_width(activation.context);
    let fov = PerspectiveProjection::from_focal_length(focal_length, width).field_of_view;
    this.set_field_of_view(fov);

    this.sync_to_display_object();

    Ok(Value::Undefined)
}

pub fn get_field_of_view<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let this = this.as_perspective_projection().unwrap();

    let perspective_projection = this.perspective_projection();

    Ok(perspective_projection.field_of_view.into())
}

pub fn set_field_of_view<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // FIXME: Render with the given PerspectiveProjection.
    avm2_stub_setter!(
        activation,
        "flash.geom.PerspectiveProjection",
        "fieldOfView"
    );

    let this = this.as_object().unwrap();
    let this = this.as_perspective_projection().unwrap();

    let fov = args.get_f64(0);
    if fov <= 0.0 || 180.0 <= fov {
        return Err(Error::avm_error(argument_error(
            activation,
            "Error #2182: Invalid fieldOfView value.  The value must be greater than 0 and less than 180.",
            2182,
        )?));
    }

    this.sync_from_display_object();

    this.set_field_of_view(fov);

    this.sync_to_display_object();

    Ok(Value::Undefined)
}

pub fn get_projection_center<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let this = this.as_perspective_projection().unwrap();

    let perspective_projection = this.perspective_projection();
    let (x, y) = perspective_projection.center;

    activation
        .avm2()
        .classes()
        .point
        .construct(activation, &[x.into(), y.into()])
}

pub fn set_projection_center<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // FIXME: Render with the given PerspectiveProjection.
    avm2_stub_setter!(
        activation,
        "flash.geom.PerspectiveProjection",
        "projectionCenter"
    );

    let this = this.as_object().unwrap();
    let this = this.as_perspective_projection().unwrap();

    this.sync_from_display_object();

    let point = args.get_object(activation, 0, "point")?;
    let center_x = point.get_slot(point_slots::X).as_f64();
    let center_y = point.get_slot(point_slots::Y).as_f64();
    this.set_center((center_x, center_y));

    this.sync_to_display_object();

    Ok(Value::Undefined)
}

pub fn to_matrix_3d<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let this = this.as_perspective_projection().unwrap();

    let width = this.get_width(activation.context);
    let matrix3d = this.perspective_projection().to_matrix3d(width as f32);

    matrix3d_to_object(matrix3d, activation)
}
