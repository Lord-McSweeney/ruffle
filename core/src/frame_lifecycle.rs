//! Frame events management
//!
//! This module aids in keeping track of which frame execution phase we are in.
//!
//! For AVM2 code, display objects execute a series of discrete phases, and
//! each object is notified about the current frame phase in rendering order.
//! When objects are created, they are 'caught up' to the current frame phase
//! to ensure correct order of operations.
//!
//! AVM1 code (presumably, either on an AVM1 stage or within an `AVM1Movie`)
//! runs in one phase, with timeline operations executing with all phases
//! inline in the order that clips were originally created.

use crate::avm2::Avm2;
use crate::context::UpdateContext;
use crate::display_object::{DisplayObject, TDisplayObject};
use tracing::instrument;

/// Which phase of the frame we're currently in.
///
/// AVM2 frames exist in one of four phases: `Enter`, `Construct`,
/// `FrameScripts`, or `Exit`. An additional `Idle` phase covers rendering and
/// event processing.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum FramePhase {
    /// We're entering the next frame.
    ///
    /// When movie clips enter a new frame, they must do two things:
    ///
    ///  - Remove all children that should not exist on the next frame.
    ///  - Increment their current frame number.
    ///
    /// Once this phase ends, we fire `enterFrame` on the broadcast list.
    Enter,

    /// We're constructing children of existing display objects.
    ///
    /// All `PlaceObject` tags should execute at this time.
    ///
    /// Once we construct the frame, we fire `frameConstructed` on the
    /// broadcast list.
    Construct,

    /// We're running all queued frame scripts.
    ///
    /// Frame scripts are the AS3 equivalent of old-style `DoAction` tags. They
    /// are queued in the `Update` phase if the current timeline frame number
    /// differs from the prior frame's one.
    FrameScripts,

    /// We're finishing frame processing.
    ///
    /// When we exit a completed frame, we fire `exitFrame` on the broadcast
    /// list.
    Exit,

    /// We're not currently executing any frame code.
    ///
    /// At this point in time, event handlers are expected to run. No frame
    /// catch-up work should execute.
    #[default]
    Idle,
}

/// Run one frame according to AVM2 frame order.
/// NOTE: The `each_orphan_movie` calls are in really odd places,
/// but this is needed to match Flash Player's output. There may
/// still be lurking bugs, but the current code matches Flash's
/// output exactly for two complex test cases (see `avm2/orphan_movie*`)
#[instrument(level = "debug", skip_all)]
pub fn run_all_phases_avm2(context: &mut UpdateContext<'_, '_>) {
    let stage = context.stage;

    if !stage.movie().is_action_script_3() {
        return;
    }

    *context.frame_phase = FramePhase::Enter;
    Avm2::each_orphan_obj(context, |orphan, context| {
        orphan.enter_frame(context);
    });
    stage.enter_frame(context);

    *context.frame_phase = FramePhase::Construct;
    Avm2::each_orphan_obj(context, |orphan, context| {
        orphan.construct_frame(context);
    });
    stage.construct_frame(context);
    stage.frame_constructed(context);

    *context.frame_phase = FramePhase::FrameScripts;
    Avm2::each_orphan_obj(context, |orphan, context| {
        orphan.run_frame_scripts(context);
    });
    stage.run_frame_scripts(context);

    *context.frame_phase = FramePhase::Exit;
    Avm2::each_orphan_obj(context, |orphan, context| {
        orphan.on_exit_frame(context);
    });
    stage.exit_frame(context);

    // We cannot easily remove dead `GcWeak` instances from the orphan list
    // inside `each_orphan_movie`, since the callback may modify the orphan list.
    // Instead, we do one cleanup at the end of the frame.
    // This performs special handling of clips which became orphaned as
    // a result of a RemoveObject tag - see `cleanup_dead_orphans` for details.
    Avm2::cleanup_dead_orphans(context);

    *context.frame_phase = FramePhase::Idle;
}

/// Run all previously-executed frame phases on a newly-constructed display
/// object.
///
/// This is a no-op on AVM1, which has it's own catch-up logic.
pub fn catchup_display_object_to_frame<'gc>(
    context: &mut UpdateContext<'_, 'gc>,
    is_content_as3: bool,
    dobj: DisplayObject<'gc>,
) {
    if !is_content_as3 {
        return;
    }

    match *context.frame_phase {
        FramePhase::Enter => {
            dobj.enter_frame(context);
        }
        FramePhase::Construct | FramePhase::FrameScripts | FramePhase::Exit | FramePhase::Idle => {
            dobj.enter_frame(context);
            dobj.construct_frame(context);
        }
    }
}
