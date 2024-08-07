//! Module with various structures for capturing and querying different forms of input
//! e.g. simple controllers, oculus/touch_controller, hands

use crate::math::Pose;
use crate::shell::XrShell;
use crate::xr;
use crate::Result;

pub trait Controls {
    /// The type returned for information on controller state
    /// 
    /// stores e.g. which buttons were pressed, and the poses of various controllers
    type InputInfo;

    /// The type returned for feedback on controller output state
    /// i.e. haptics
    type OutputInfo;

    /// Getter for initialization
    /// 
    /// Return the single xr::ActionSet with all the actions for this control scheme.
    fn action_set(&self) -> &xr::ActionSet;

    /// Getter for initialization
    /// 
    /// Return the suggested_bindings for (potentially multiple) (interaction_profile, [(binding -> action)]),
    /// where each action exposed in [Controls::action_set] has a binding in each interaction profile.
    /// These may be combined with bindings for other control schemes before being passed to 
    /// [xrSuggestInteractionProfileBindings](https://registry.khronos.org/OpenXR/specs/1.0/man/html/xrSuggestInteractionProfileBindings.html),
    /// which "the application can call \[...\] **once** per interaction profile".
    /// 
    /// TODO ArrayVec here?
    fn suggested_bindings(&self, xr_instance: &xr::Instance) -> Result<Vec<(
        &str,
        Vec<xr::Binding<'_>>
    )>>;

    fn locate(&self, xr_shell: &XrShell, space: &xr::Space, time: xr::Time) -> Result<Self::InputInfo>;
    fn apply(&self, xr_shell: &XrShell, output: &Self::OutputInfo) -> Result<()>;
}

pub struct PointAndClickHand {
    pub grip: Pose,
    pub point: Pose,
    pub click: bool,
}

pub struct PointAndClickInput {
    pub lh: Option<PointAndClickHand>,
    pub rh: Option<PointAndClickHand>,
    pub menu_button: bool,
}

pub struct PointAndClickControls {
    lh_subpath: xr::Path,
    rh_subpath: xr::Path,
    
    action_set: xr::ActionSet,
    
    grip: xr::Action<xr::Posef>,
    lh_grip_space: xr::Space,
    rh_grip_space: xr::Space,

    point: xr::Action<xr::Posef>,
    lh_point_space: xr::Space,
    rh_point_space: xr::Space,

    click: xr::Action<bool>,
    menu_button: xr::Action<bool>,

}
impl PointAndClickControls {
    pub fn new(xr_shell: &XrShell, action_set_name: &'static str, localized_name: &'static str) -> Result<Self> {
        // Create an action set to encapsulate our actions
        // TODO: What is the unicode compatibility for OpenXR? Can we use UTF-8 &str everywhere?
        let action_set =
            xr_shell.xr_instance.create_action_set(action_set_name, localized_name, 0)?;

        let lh_subpath = xr_shell.xr_instance.string_to_path("/user/hand/left")?;
        let rh_subpath = xr_shell.xr_instance.string_to_path("/user/hand/right")?;

        // TODO localisation /shrug
        // We have four categories of input:
        // - the palm or "grip" orientation for each hand separately
        // - the pointing orientation for each hand separately
        // - the "click" input for each hand separately
        // - the "menu button" click input, we don't care which hand.
        // The inputs for each separate hand are registered as a single action for both with "subaction_paths" for each hand.
        // This means theoretically the inputs will be presented nicely on platforms which expose rebinding controls like Steam VR.
        let grip =
            action_set.create_action::<xr::Posef>("grip", "Palm Orientation", &[
                lh_subpath,
                rh_subpath,
            ])?;
        let point = 
            action_set.create_action::<xr::Posef>("point", "Pointing Direction", &[
                lh_subpath,
                rh_subpath,
            ])?;
        let click = 
            action_set.create_action::<bool>("click", "Click", &[
                lh_subpath,
                rh_subpath,
            ])?;
        let menu_button = 
            action_set.create_action::<bool>("menu_button", "Menu Button", &[])?;

        // Create an action space for each device we want to locate
        let lh_grip_space = grip.create_space(
            xr_shell.xr_session.clone(),
            lh_subpath,
            xr::Posef::IDENTITY,
        )?;
        let rh_grip_space = grip.create_space(
            xr_shell.xr_session.clone(),
            rh_subpath,
            xr::Posef::IDENTITY,
        )?;

        let lh_point_space = point.create_space(
            xr_shell.xr_session.clone(),
            lh_subpath,
            xr::Posef::IDENTITY,
        )?;
        let rh_point_space = point.create_space(
            xr_shell.xr_session.clone(),
            rh_subpath,
            xr::Posef::IDENTITY,
        )?;

        Ok(Self {
            lh_subpath,
            rh_subpath,

            action_set,
            
            grip,
            lh_grip_space,
            rh_grip_space,

            point,
            lh_point_space,
            rh_point_space,

            click,
            menu_button
        })
    }
}
impl Controls for PointAndClickControls {
    type InputInfo = PointAndClickInput;
    
    type OutputInfo = ();

    fn action_set(&self) -> &xr::ActionSet {
        &self.action_set
    }
    fn suggested_bindings(&self, xr_instance: &xr::Instance) -> Result<Vec<(
        &str,
        Vec<xr::Binding<'_>>
    )>> {
        Ok(vec![
            (
                "/interaction_profiles/khr/simple_controller",
                vec![
                    xr::Binding::new(
                        &self.grip,
                        xr_instance.string_to_path("/user/hand/left/input/grip/pose")?
                    ),
                    xr::Binding::new(
                        &self.grip,
                        xr_instance.string_to_path("/user/hand/right/input/grip/pose")?
                    ),

                    xr::Binding::new(
                        &self.point,
                        xr_instance.string_to_path("/user/hand/left/input/aim/pose")?
                    ),
                    xr::Binding::new(
                        &self.point,
                        xr_instance.string_to_path("/user/hand/right/input/aim/pose")?
                    ),

                    xr::Binding::new(
                        &self.click,
                        xr_instance.string_to_path("/user/hand/left/input/select/click")?
                    ),
                    xr::Binding::new(
                        &self.click,
                        xr_instance.string_to_path("/user/hand/right/input/select/click")?
                    ),

                    xr::Binding::new(
                        &self.menu_button,
                        xr_instance.string_to_path("/user/hand/left/input/menu/click")?
                    ),
                    xr::Binding::new(
                        &self.menu_button,
                        xr_instance.string_to_path("/user/hand/right/input/menu/click")?
                    ),
                ]
            )
        ])
    }
    
    fn locate(&self, xr_shell: &XrShell, space: &xr::Space, time: xr::Time) -> Result<Self::InputInfo> {
        // Find where our controllers are located in the Stage space
        let lh_grip = self
            .lh_grip_space
            .locate(space, time)?;
        let lh_point = self
            .lh_point_space
            .locate(space, time)?;

        let lh_active = 
            self
                .grip
                .is_active(&xr_shell.xr_session, self.lh_subpath)?
            &&
            self
                .point
                .is_active(&xr_shell.xr_session, self.lh_subpath)?;

        let rh_grip = self
            .rh_grip_space
            .locate(space, time)?;
        let rh_point = self
            .rh_point_space
            .locate(space, time)?;
            
        let rh_active = 
            self
                .grip
                .is_active(&xr_shell.xr_session, self.rh_subpath)?
            &&
            self
                .point
                .is_active(&xr_shell.xr_session, self.rh_subpath)?;

        let lh_click = self.click.state(&xr_shell.xr_session, self.lh_subpath)?;
        let rh_click = self.click.state(&xr_shell.xr_session, self.rh_subpath)?;

        let menu_click = self.menu_button.state(&xr_shell.xr_session, xr::Path::NULL)?;

        // TODO look at the flags for {lh,rh}_{grip,point}

        Ok(PointAndClickInput {
            lh: if lh_active {
                Some(PointAndClickHand {
                    grip: lh_grip.pose.into(),
                    point: lh_point.pose.into(),
                    click: lh_click.current_state
                })
            } else {
                None
            },
            rh: if rh_active {
                Some(PointAndClickHand {
                    grip: rh_grip.pose.into(),
                    point: rh_point.pose.into(),
                    click: rh_click.current_state
                })
            } else {
                None
            },
            menu_button: menu_click.is_active && menu_click.current_state
        })
    }
    
    fn apply(&self, _xr_shell: &XrShell, _output: &Self::OutputInfo) -> Result<()> {
        Ok(())    
    }

    
}

/// Simple controllers
/// 
/// "/interaction_profiles/khr/simple_controller"
struct SimpleControllers();

/// Oculus/Meta Quest 3 controllers
/// 
/// "/interaction_profiles/oculus/touch_controller"?
/// https://community.khronos.org/t/quest-3-controllers-with-steamvr/111048
/// https://en.wikipedia.org/wiki/Oculus_Touch
struct OculusTouchControllers();

