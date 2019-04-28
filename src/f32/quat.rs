use super::{Angle, Mat4, Quat, Vec3, Vec4};
use std::{
    fmt,
    ops::{Mul, MulAssign, Neg},
};

#[inline]
pub fn quat(x: f32, y: f32, z: f32, w: f32) -> Quat {
    Quat::new(x, y, z, w)
}

impl Quat {
    #[inline]
    /// Create quaterion for a normalized rotation axis and angle.
    pub fn from_axis_angle(axis: Vec3, angle: Angle) -> Quat {
        debug_assert!((axis.length_squared() - 1.0).abs() < 0.01);
        let (s, c) = (angle * 0.5).sin_cos();
        (axis * s).extend(c).into()
    }

    #[inline]
    pub fn from_rotation_x(angle: Angle) -> Quat {
        let (s, c) = (angle * 0.5).sin_cos();
        Quat::new(s, 0.0, 0.0, c)
    }

    #[inline]
    pub fn from_rotation_y(angle: Angle) -> Quat {
        let (s, c) = (angle * 0.5).sin_cos();
        Quat::new(0.0, s, 0.0, c)
    }

    #[inline]
    pub fn from_rotation_z(angle: Angle) -> Quat {
        let (s, c) = (angle * 0.5).sin_cos();
        Quat::new(0.0, 0.0, s, c)
    }

    #[inline]
    /// Create a quaternion from the given yaw (around y), pitch (around x) and roll (around z).
    pub fn from_rotation_ypr(yaw: Angle, pitch: Angle, roll: Angle) -> Quat {
        let (sy, cy) = (-0.5 * yaw).sin_cos();
        let (sr, cr) = (-0.5 * roll).sin_cos();
        let (sp, cp) = (-0.5 * pitch).sin_cos();
        let w = cy * cp * cr + sy * sp * sr;
        let x = -cy * sp * cr - sy * cp * sr;
        let y = cy * sp * sr - sy * cp * cr;
        let z = sy * sp * cr - cy * cp * sr;
        Quat::new(x, y, z, w)
    }

    #[inline]
    pub fn from_rotation_matrix(mat: &Mat4) -> Quat {
        // from DirectXMath XMQuaternionRotationMatrix
        // TODO: sse2 version
        let (m00, m01, m02, _m03) = mat.x_axis.into();
        let (m10, m11, m12, _m13) = mat.y_axis.into();
        let (m20, m21, m22, _m23) = mat.z_axis.into();
        if m22 <= 0.0 {
            // x^2 + y^2 >= z^2 + w^2
            let dif10 = m11 - m00;
            let omm22 = 1.0 - m22;
            if dif10 <= 0.0 {
                // x^2 >= y^2
                let four_xsq = omm22 - dif10;
                let inv4x = 0.5 / four_xsq.sqrt();
                Quat::new(
                    four_xsq * inv4x,
                    (m01 + m10) * inv4x,
                    (m02 + m20) * inv4x,
                    (m12 - m21) * inv4x,
                )
            } else {
                // y^2 >= x^2
                let four_ysq = omm22 + dif10;
                let inv4y = 0.5 / four_ysq.sqrt();
                Quat::new(
                    (m01 + m10) * inv4y,
                    four_ysq * inv4y,
                    (m12 + m21) * inv4y,
                    (m20 - m02) * inv4y,
                )
            }
        } else {
            // z^2 + w^2 >= x^2 + y^2
            let sum10 = m11 + m00;
            let opm22 = 1.0 + m22;
            if sum10 <= 0.0 {
                // z^2 >= w^2
                let four_zsq = opm22 - sum10;
                let inv4z = 0.5 / four_zsq.sqrt();
                Quat::new(
                    (m02 + m20) * inv4z,
                    (m12 + m21) * inv4z,
                    four_zsq * inv4z,
                    (m01 - m10) * inv4z,
                )
            } else {
                // w^2 >= z^2
                let four_wsq = opm22 + sum10;
                let inv4w = 0.5 / four_wsq.sqrt();
                Quat::new(
                    (m12 - m21) * inv4w,
                    (m20 - m02) * inv4w,
                    (m01 - m10) * inv4w,
                    four_wsq * inv4w,
                )
            }
        }
    }

    #[inline]
    pub fn get_rotation_angle(self) -> Angle {
        let w = self.get_w();
        Angle::acos(w) * 2.0
    }

    #[inline]
    pub fn get_rotation_axis(self) -> Vec3 {
        let (x, y, z, w) = self.into();
        let sin_theta_over_2sq = 1.0 - w * w;
        if sin_theta_over_2sq <= 0.0 {
            // panic?
            return Vec3::unit_x();
        }

        let inv_sin_theta_over_2 = 1.0 / sin_theta_over_2sq.sqrt();
        Vec3::new(
            x * inv_sin_theta_over_2,
            y * inv_sin_theta_over_2,
            z * inv_sin_theta_over_2,
        )
    }

    #[inline]
    pub fn conjugate(self) -> Quat {
        let v: Vec4 = self.into();
        v.truncate().neg().extend(v.get_w()).into()
    }

    #[inline]
    pub fn dot(self, rhs: Quat) -> f32 {
        let v: Vec4 = self.into();
        v.dot(rhs.into())
    }

    #[inline]
    pub fn length(self) -> f32 {
        let v: Vec4 = self.into();
        v.length()
    }

    #[inline]
    pub fn length_squared(self) -> f32 {
        let v: Vec4 = self.into();
        v.length_squared()
    }

    #[inline]
    pub fn length_reciprocal(self) -> f32 {
        1.0 / self.length()
    }

    #[inline]
    pub fn normalize(self) -> Quat {
        let inv_len = self.length_reciprocal();
        let v: Vec4 = self.into();
        v.mul(inv_len).into()
    }

    #[inline]
    pub fn is_normalized(self) -> bool {
        const THRESHOLD: f32 = 0.00001;
        (self.length_squared() - 1.0).abs() < THRESHOLD
    }

    #[inline]
    pub fn lerp(self, end: Quat, t: f32) -> Quat {
        let start: Vec4 = self.into();
        let end: Vec4 = end.into();
        let dot = start.dot(end);
        let bias = if dot >= 0.0 { 1.0 } else { -1.0 };
        let interpolated = start + (t * ((end * bias) - start));
        let result: Quat = interpolated.into();
        result.normalize()
    }

    #[inline]
    /// Multiplies two quaternions.
    /// Note that due to floating point rounding the result may not be perfectly normalized.
    /// Multiplication order is as follows:
    /// `local_to_world = local_to_object * object_to_world`
    pub fn mul_quat(self, rhs: Quat) -> Quat {
        let (x0, y0, z0, w0) = self.into();
        let (x1, y1, z1, w1) = rhs.into();

        let x = (w1 * x0) + (x1 * w0) + (y1 * z0) - (z1 * y0);
        let y = (w1 * y0) - (x1 * z0) + (y1 * w0) + (z1 * x0);
        let z = (w1 * z0) + (x1 * y0) - (y1 * x0) + (z1 * w0);
        let w = (w1 * w0) - (x1 * x0) - (y1 * y0) - (z1 * z0);
        Quat::new(x, y, z, w)
    }
}

impl fmt::Debug for Quat {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let (x, y, z, w) = self.into();
        fmt.debug_tuple("Quat")
            .field(&x)
            .field(&y)
            .field(&z)
            .field(&w)
            .finish()
    }
}

impl fmt::Display for Quat {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let (x, y, z, w) = self.into();
        write!(fmt, "({}, {}, {}, {})", x, y, z, w)
    }
}

impl Mul<Quat> for Quat {
    type Output = Quat;
    #[inline]
    fn mul(self, rhs: Quat) -> Quat {
        self.mul_quat(rhs)
    }
}

impl MulAssign<Quat> for Quat {
    #[inline]
    fn mul_assign(&mut self, rhs: Quat) {
        *self = self.mul_quat(rhs);
    }
}

impl Neg for Quat {
    type Output = Quat;
    #[inline]
    fn neg(self) -> Quat {
        let v: Vec4 = self.into();
        (-1.0 * v).into()
    }
}

impl PartialEq for Quat {
    #[inline]
    fn eq(&self, rhs: &Quat) -> bool {
        let v: Vec4 = self.into();
        v.cmpeq(rhs.into()).all()
    }
}

impl AsRef<[f32; 4]> for Quat {
    #[inline]
    fn as_ref(&self) -> &[f32; 4] {
        unsafe { &*(self as *const Quat as *const [f32; 4]) }
    }
}

impl AsMut<[f32; 4]> for Quat {
    #[inline]
    fn as_mut(&mut self) -> &mut [f32; 4] {
        unsafe { &mut *(self as *mut Quat as *mut [f32; 4]) }
    }
}

impl From<(f32, f32, f32, f32)> for Quat {
    #[inline]
    fn from(t: (f32, f32, f32, f32)) -> Self {
        Quat::new(t.0, t.1, t.2, t.3)
    }
}

impl From<&(f32, f32, f32, f32)> for Quat {
    #[inline]
    fn from(t: &(f32, f32, f32, f32)) -> Self {
        Quat::new(t.0, t.1, t.2, t.3)
    }
}