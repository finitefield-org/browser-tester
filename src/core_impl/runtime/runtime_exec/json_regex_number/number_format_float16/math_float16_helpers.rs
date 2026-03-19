use super::*;

impl Harness {
    pub(crate) fn sum_precise(values: &[Value]) -> f64 {
        let mut sum = 0.0f64;
        let mut compensation = 0.0f64;
        for value in values {
            let value = Self::coerce_number_for_global(value);
            let adjusted = value - compensation;
            let next = sum + adjusted;
            compensation = (next - sum) - adjusted;
            sum = next;
        }
        sum
    }

    pub(crate) fn js_math_round(value: f64) -> f64 {
        if !value.is_finite() || value == 0.0 {
            return value;
        }
        if (-0.5..0.0).contains(&value) {
            return -0.0;
        }
        let floor = value.floor();
        let frac = value - floor;
        if frac < 0.5 { floor } else { floor + 1.0 }
    }

    pub(crate) fn js_math_sign(value: f64) -> f64 {
        if value.is_nan() {
            f64::NAN
        } else if value == 0.0 {
            value
        } else if value > 0.0 {
            1.0
        } else {
            -1.0
        }
    }

    pub(crate) fn to_i32_for_math(value: &Value) -> i32 {
        let numeric = Self::coerce_number_for_global(value);
        if !numeric.is_finite() {
            return 0;
        }
        let unsigned = numeric.trunc().rem_euclid(4_294_967_296.0);
        if unsigned >= 2_147_483_648.0 {
            (unsigned - 4_294_967_296.0) as i32
        } else {
            unsigned as i32
        }
    }

    pub(crate) fn to_u32_for_math(value: &Value) -> u32 {
        let numeric = Self::coerce_number_for_global(value);
        if !numeric.is_finite() {
            return 0;
        }
        numeric.trunc().rem_euclid(4_294_967_296.0) as u32
    }

    pub(crate) fn math_f16round(value: f64) -> f64 {
        let half = Self::f32_to_f16_bits(value as f32);
        Self::f16_bits_to_f32(half) as f64
    }

    pub(crate) fn f32_to_f16_bits(value: f32) -> u16 {
        let bits = value.to_bits();
        let sign = ((bits >> 16) & 0x8000) as u16;
        let exp = ((bits >> 23) & 0xff) as i32;
        let mant = bits & 0x007f_ffff;

        if exp == 0xff {
            if mant == 0 {
                return sign | 0x7c00;
            }
            return sign | 0x7e00;
        }

        let exp16 = exp - 127 + 15;
        if exp16 >= 0x1f {
            return sign | 0x7c00;
        }

        if exp16 <= 0 {
            if exp16 < -10 {
                return sign;
            }
            let mantissa = mant | 0x0080_0000;
            let shift = (14 - exp16) as u32;
            let mut half_mant = mantissa >> shift;
            let round_bit = 1u32 << (shift - 1);
            if (mantissa & round_bit) != 0
                && ((mantissa & (round_bit - 1)) != 0 || (half_mant & 1) != 0)
            {
                half_mant += 1;
            }
            return sign | (half_mant as u16);
        }

        let mut half_exp = (exp16 as u16) << 10;
        let mut half_mant = (mant >> 13) as u16;
        let round_bits = mant & 0x1fff;
        if round_bits > 0x1000 || (round_bits == 0x1000 && (half_mant & 1) != 0) {
            half_mant = half_mant.wrapping_add(1);
            if half_mant == 0x0400 {
                half_mant = 0;
                half_exp = half_exp.wrapping_add(0x0400);
                if half_exp >= 0x7c00 {
                    return sign | 0x7c00;
                }
            }
        }
        sign | half_exp | half_mant
    }

    pub(crate) fn f16_bits_to_f32(bits: u16) -> f32 {
        let sign = ((bits & 0x8000) as u32) << 16;
        let exp = ((bits >> 10) & 0x1f) as u32;
        let mant = (bits & 0x03ff) as u32;

        let out_bits = if exp == 0 {
            if mant == 0 {
                sign
            } else {
                let mut mantissa = mant;
                let mut exp_val = -14i32;
                while (mantissa & 0x0400) == 0 {
                    mantissa <<= 1;
                    exp_val -= 1;
                }
                mantissa &= 0x03ff;
                let exp32 = ((exp_val + 127) as u32) << 23;
                sign | exp32 | (mantissa << 13)
            }
        } else if exp == 0x1f {
            sign | 0x7f80_0000 | (mant << 13)
        } else {
            let exp32 = (((exp as i32) - 15 + 127) as u32) << 23;
            sign | exp32 | (mant << 13)
        };

        f32::from_bits(out_bits)
    }
}
