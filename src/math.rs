use crate::xr;
use cgmath::{self, SquareMatrix};

/// Two-component vector, byte-compatible with bytemuck, cgmath, and openxr.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vec2(pub [f32; 2]);
impl From<cgmath::Vector2<f32>> for Vec2 {
    fn from(value: cgmath::Vector2<f32>) -> Self {
        Self(value.into())
    }
}
impl From<Vec2> for cgmath::Vector2<f32> {
    fn from(value: Vec2) -> Self {
        value.0.into()
    }
}
impl From<xr::Vector2f> for Vec2 {
    fn from(value: xr::Vector2f) -> Self {
        Self([value.x, value.y])
    }
}
impl From<Vec2> for xr::Vector2f {
    fn from(value: Vec2) -> Self {
        Self {
            x: value.0[0],
            y: value.0[1],
        }
    }
}

/// Three-component vector, byte-compatible with bytemuck, cgmath, and openxr.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vec3(pub [f32; 3]);
impl From<cgmath::Vector3<f32>> for Vec3 {
    fn from(value: cgmath::Vector3<f32>) -> Self {
        Self(value.into())
    }
}
impl From<Vec3> for cgmath::Vector3<f32> {
    fn from(value: Vec3) -> Self {
        value.0.into()
    }
}
impl From<xr::Vector3f> for Vec3 {
    fn from(value: xr::Vector3f) -> Self {
        Self([value.x, value.y, value.z])
    }
}
impl From<Vec3> for xr::Vector3f {
    fn from(value: Vec3) -> Self {
        Self {
            x: value.0[0],
            y: value.0[1],
            z: value.0[2],
        }
    }
}

/// Quaternion rotation, byte-compatible with bytemuck, cgmath, and openxr.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Quat([f32; 4]);
impl From<cgmath::Quaternion<f32>> for Quat {
    fn from(value: cgmath::Quaternion<f32>) -> Self {
        Self(value.into())
    }
}
impl From<Quat> for cgmath::Quaternion<f32> {
    fn from(value: Quat) -> Self {
        value.0.into()
    }
}
impl From<xr::Quaternionf> for Quat {
    fn from(value: xr::Quaternionf) -> Self {
        Self([value.x, value.y, value.z, value.w])
    }
}
impl From<Quat> for xr::Quaternionf {
    fn from(value: Quat) -> Self {
        Self {
            x: value.0[0],
            y: value.0[1],
            z: value.0[2],
            w: value.0[3],
        }
    }
}

/// Non-scaled translation+rotation orientation, byte-compatible with bytemuck and openxr::Posef.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Pose {
    pub position: Vec3,
    pub orientation: Quat,
}
impl From<xr::Posef> for Pose {
    fn from(value: xr::Posef) -> Self {
        Self {
            position: value.position.into(),
            orientation: value.orientation.into(),
        }
    }
}


/// Four-by-four column-major matrix, byte-compatible with bytemuck, cgmath, and openxr.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Mat4(pub [[f32; 4]; 4]);
impl Mat4 {
    pub const fn zero() -> Self {
        Self([[0.0; 4]; 4])
    }
    pub const fn identity() -> Self {
        Self([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }
    pub fn from_translation(v: Vec3) -> Mat4 {
        cgmath::Matrix4::from_translation(v.into()).into()
    }
    pub fn from_translation_rotation(trans: Vec3, rot: Quat) -> Mat4 {
        let quat: cgmath::Quaternion<f32> = rot.into();

        let x2 = quat.v.x + quat.v.x;
        let y2 = quat.v.y + quat.v.y;
        let z2 = quat.v.z + quat.v.z;

        let xx2 = x2 * quat.v.x;
        let xy2 = x2 * quat.v.y;
        let xz2 = x2 * quat.v.z;

        let yy2 = y2 * quat.v.y;
        let yz2 = y2 * quat.v.z;
        let zz2 = z2 * quat.v.z;

        let sy2 = y2 * quat.s;
        let sz2 = z2 * quat.s;
        let sx2 = x2 * quat.s;

        #[cfg_attr(rustfmt, rustfmt_skip)]
        cgmath::Matrix4::new(
            1.0 - yy2 - zz2, xy2 + sz2, xz2 - sy2, 0.0,
            xy2 - sz2, 1.0 - xx2 - zz2, yz2 + sx2, 0.0,
            xz2 + sy2, yz2 - sx2, 1.0 - xx2 - yy2, 0.0,
            trans.0[0], trans.0[1], trans.0[2], 1.0
        ).into()
    }

    pub fn as_cg(self) -> cgmath::Matrix4<f32> {
        self.into()
    }

    pub fn inverse(self) -> Option<Mat4> {
        self.as_cg().invert().map(Into::into)
    }

    /// From https://github.com/KhronosGroup/OpenXR-SDK/blob/f90488c4fb1537f4256d09d4a4d3ad5543ebaf24/src/common/xr_linear.h#L623
    pub fn xr_projection_fov(fov: xr::Fovf, near_z: f32, far_z: f32) -> Mat4 {
        Self::xr_projection_tan(
            fov.angle_left.tan(),
            fov.angle_right.tan(),
            fov.angle_up.tan(),
            fov.angle_down.tan(),
            near_z, far_z,
        )
    }

    /// From https://github.com/KhronosGroup/OpenXR-SDK/blob/f90488c4fb1537f4256d09d4a4d3ad5543ebaf24/src/common/xr_linear.h#L564
    pub fn xr_projection_tan(tan_left: f32, tan_right: f32, tan_up: f32, tan_down: f32, near_z: f32, far_z: f32) -> Mat4 {
        let tan_width: f32 = tan_right - tan_left;

        // Set to tanAngleDown - tanAngleUp for a clip space with positive Y down (Vulkan).
        // Set to tanAngleUp - tanAngleDown for a clip space with positive Y up (OpenGL / D3D / Metal).
        let tan_height: f32 = tan_down - tan_up;

        // Set to nearZ for a [-1,1] Z clip space (OpenGL / OpenGL ES).
        // Set to zero for a [0,1] Z clip space (Vulkan / D3D / Metal).
        let offset_z: f32 = 0.0;

        if far_z <= near_z {
            // place the far plane at infinity
            cgmath::Matrix4::new(
                2.0/tan_width, 0.0, 0.0, 0.0,
                0.0, 2.0/tan_height, 0.0, 0.0,
                (tan_right + tan_left) / tan_width, (tan_up + tan_down) / tan_height, -1.0, -1.0,
                0.0, 0.0, -(near_z + offset_z), 0.0
            ).into()
        } else {
            // normal projection
            cgmath::Matrix4::new(
                2.0/tan_width, 0.0, 0.0, 0.0,
                0.0, 2.0/tan_height, 0.0, 0.0,
                (tan_right + tan_left) / tan_width, (tan_up + tan_down) / tan_height, -(far_z + offset_z) / (far_z - near_z), -1.0,
                0.0, 0.0, -(far_z * (near_z + offset_z)) / (far_z - near_z), 0.0
            ).into()
        }
    }
}
impl From<cgmath::Matrix4<f32>> for Mat4 {
    fn from(value: cgmath::Matrix4<f32>) -> Self {
        Self(value.into())
    }
}
impl From<Mat4> for cgmath::Matrix4<f32> {
    fn from(value: Mat4) -> Self {
        value.0.into()
    }
}
impl From<xr::Posef> for Mat4 {
    fn from(value: xr::Posef) -> Self {
        Self::from_translation_rotation(value.position.into(), value.orientation.into())
    }
}
impl From<Pose> for Mat4 {
    fn from(value: Pose) -> Self {
        Self::from_translation_rotation(value.position, value.orientation)
    }
}
impl From<Quat> for Mat4 {
    fn from(value: Quat) -> Self {
        let cgquat: cgmath::Quaternion<f32> = value.into();
        let cgmat: cgmath::Matrix4<f32> = cgquat.into();
        cgmat.into()
    }
}
impl std::ops::Mul<Mat4> for Mat4 {
    type Output = Mat4;
    
    fn mul(self, rhs: Mat4) -> Self::Output {
        (self.as_cg() * rhs.as_cg()).into()
    }
}