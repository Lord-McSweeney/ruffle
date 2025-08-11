use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, TObject};
use crate::avm2::{Activation, ClassObject, Error};
use crate::context::UpdateContext;
use crate::display_object::{DisplayObject, TDisplayObject};
use crate::utils::HasPrefixField;
use gc_arena::{Collect, Gc, GcWeak};
use ruffle_render::perspective_projection::PerspectiveProjection;
use std::cell::Cell;
use std::fmt;

const DEFAULT_PP_WIDTH: f64 = 500.0;

/// A class instance allocator that allocates PerspectiveProjection objects.
pub fn perspective_projection_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    // Actionscript can observe that the field_of_view and center values are set before
    // the PerspectiveProjection constructor is called, so we need to set the
    // values in the allocator (rather than the constructor).
    let object = PerspectiveProjectionObject(Gc::new(
        activation.gc(),
        PerspectiveProjectionObjectData {
            base,
            display_object: None,
            field_of_view: Cell::new(55.0),
            center: Cell::new((250.0, 250.0)),
        },
    ));

    Ok(object.into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct PerspectiveProjectionObject<'gc>(pub Gc<'gc, PerspectiveProjectionObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct PerspectiveProjectionObjectWeak<'gc>(
    pub GcWeak<'gc, PerspectiveProjectionObjectData<'gc>>,
);

#[derive(Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct PerspectiveProjectionObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// The DisplayObject associated with this PerspectiveProjection, or None
    /// if this PerspectiveProjection is not associated with a DisplayObject
    display_object: Option<DisplayObject<'gc>>,

    field_of_view: Cell<f64>,

    center: Cell<(f64, f64)>,
}

impl<'gc> PerspectiveProjectionObject<'gc> {
    pub fn from_display_object(
        activation: &mut Activation<'_, 'gc>,
        display_object: DisplayObject<'gc>,
    ) -> Self {
        let class = activation.avm2().classes().perspectiveprojection;
        let base = ScriptObjectData::new(class);

        PerspectiveProjectionObject(Gc::new(
            activation.gc(),
            PerspectiveProjectionObjectData {
                base,
                display_object: Some(display_object),
                field_of_view: Cell::new(55.0),
                center: Cell::new((250.0, 250.0)),
            },
        ))
    }

    pub fn get_width(self, context: &mut UpdateContext<'gc>) -> f64 {
        match self.display_object() {
            // Not associated with any DO
            None => DEFAULT_PP_WIDTH,
            // Stage's PerspectiveProjection
            Some(dobj) if dobj.as_stage().is_some() => DEFAULT_PP_WIDTH,
            // Associated with other DO.
            Some(_dobj) => context.stage.stage_size().0 as f64,
        }
    }

    pub fn sync_from_display_object(self) {
        let Some(base) = self.display_object().map(|d| d.base()) else {
            // Not associated with DO. Unnecessary to sync.
            return;
        };

        let Some(perspective_projection) = base.perspective_projection() else {
            return;
        };

        self.set_field_of_view(perspective_projection.field_of_view);

        let center = perspective_projection.center;
        self.set_center(center);
    }

    pub fn sync_to_display_object(self) {
        let Some(base) = self.display_object().map(|d| d.base()) else {
            // Not associated with DO. Unnecessary to sync.
            return;
        };

        let Some(mut proj) = base.perspective_projection() else {
            return;
        };

        proj.field_of_view = self.field_of_view();
        proj.center = self.center();

        base.set_perspective_projection(Some(proj));
    }

    pub fn perspective_projection(self) -> PerspectiveProjection {
        if let Some(display_object) = self.display_object() {
            display_object
                .base()
                .perspective_projection()
                .unwrap_or_default()
        } else {
            PerspectiveProjection {
                field_of_view: self.field_of_view(),
                center: self.center(),
            }
        }
    }

    pub fn display_object(self) -> Option<DisplayObject<'gc>> {
        self.0.display_object
    }

    pub fn field_of_view(self) -> f64 {
        self.0.field_of_view.get()
    }

    pub fn set_field_of_view(self, field_of_view: f64) {
        self.0.field_of_view.set(field_of_view);
    }

    pub fn center(self) -> (f64, f64) {
        self.0.center.get()
    }

    pub fn set_center(self, center: (f64, f64)) {
        self.0.center.set(center);
    }
}

impl<'gc> TObject<'gc> for PerspectiveProjectionObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}

impl fmt::Debug for PerspectiveProjectionObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PerspectiveProjectionObject")
    }
}
